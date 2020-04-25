mod file_mapping;
mod frames;
mod header_region;

use memmap2::MmapMut;
use std::error::Error;
use std::fs::File;

pub struct Backend {
    size: usize,
    mapped_file: MmapMut,
    file: File,
    header_region: header_region::HeaderRegion,
}

impl Backend {
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let (size, mut mapped_file) = Self::open_file(&file)?;
        let mut header_region = Self::initialize_header_region(&mapped_file)?;
        if header_region.version == 0 {
            header_region.version = 1;
            header_region.update(&mut mapped_file)?;
        }
        let backend = Self {
            header_region,
            file,
            mapped_file,
            size,
        };
        Ok(backend)
    }

    /// runtime: O(n)
    pub fn create(&mut self, bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
        let start = self.next_free_frame_position()?;
        self.write_bytes_starting_at(start, bytes)?;
        self.flush()?;
        Ok(start)
    }

    fn write_bytes_starting_at(
        &mut self,
        start: usize,
        bytes: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        // prepare for looping
        let chunk_size = frames::Frame::capacity();
        let mut last_frame_position: Option<usize> = None;
        for (index, byte_chunk) in bytes.chunks(chunk_size).enumerate() {
            // use the given position on first iteration
            let position = if index == 0 {
                start
            } else {
                self.next_free_frame_position()?
            };

            // persist the chunk into a frame
            self.create_frame(position)?;
            self.write_frame_body(position, byte_chunk)?;

            // set the "next" pointer of the last frame to this frame
            if let Some(last_position) = last_frame_position {
                let mut last_frame = self.read_frame(last_position)?;
                last_frame.next = position;
                self.update_frame(last_frame)?;
            }
            last_frame_position = Some(position);
        }
        Ok(())
    }

    /// runtime: O(1)
    pub fn read(&self, position: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes: Vec<u8> = vec![];
        let mut cursor: usize = position;
        while cursor != 0 {
            let frame = self.read_frame(position)?;
            let body = self.read_frame_body(position)?;
            bytes.extend_from_slice(body);
            cursor = frame.next;
        }
        Ok(bytes)
    }

    // runtime: O(n) - is delete + create
    pub fn update(&mut self, position: usize, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        self.delete(position)?;
        self.write_bytes_starting_at(position, bytes)?;
        self.flush()?;
        Ok(())
    }

    /// runtime: O(1)
    pub fn delete(&mut self, position: usize) -> Result<(), Box<dyn Error>> {
        let mut cursor: usize = position;
        while cursor != 0 {
            let mut frame = self.read_frame(position)?;
            frame.deleted = true;
            cursor = frame.next;
            self.update_frame(frame)?;
        }
        self.flush()?;
        Ok(())
    }

    /// runtime: O(n)
    fn next_free_frame_position(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut result: Option<usize> = None;
        let mut position = header_region::HeaderRegion::first_frame_position();
        let max_position = self.header_region.frame_count * frames::Frame::total_size();
        while result.is_none() && position < max_position {
            let frame = self.read_frame(position)?;
            if frame.deleted == true {
                result = Some(position);
            } else {
                position += frames::Frame::total_size();
            }
        }
        if let Some(next_free_position) = result {
            Ok(next_free_position)
        } else {
            let next_free_position = self.header_region.frame_count * frames::Frame::total_size();
            self.header_region.frame_count += 1;
            self.header_region.update(&mut self.mapped_file)?;
            if (next_free_position + frames::Frame::total_size()) > self.size {
                self.resize_file()?;
            }
            Ok(next_free_position)
        }
    }
}
