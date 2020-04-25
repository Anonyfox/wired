mod allocation;
mod file_mapping;
mod frames;
mod header;

use memmap2::MmapMut;
use std::error::Error;
use std::fs::File;

pub struct Backend {
    size: usize,
    mapped_file: MmapMut,
    file: File,
    header: header::Header,
}

impl Backend {
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let (size, mut mapped_file) = Self::open_file(&file)?;
        let header = Self::initialize_header(&mut mapped_file)?;
        let backend = Self {
            header,
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
            let frame = self.read_frame(cursor)?;
            if frame.deleted == false {
                let body = self.read_frame_body(cursor)?;
                bytes.extend_from_slice(body);
            }
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
            let mut frame = self.read_frame(cursor)?;
            cursor = frame.next;
            frame.deleted = true;
            frame.next = 0;
            self.update_frame(frame)?;
        }
        self.flush()?;
        Ok(())
    }

    pub fn offset() -> usize {
        header::Header::size()
    }

    pub fn block_size() -> usize {
        frames::Frame::total_size()
    }

    pub fn is_empty(&self) -> bool {
        self.header.frame_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        // prepare
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut backend = Backend::new(file).expect("could not create mmap");

        // insert simple element
        let position = backend.create(b"hello").expect("could not create");
        assert_eq!(position, 16);

        // confirm by reading back
        let data = backend.read(16).expect("could not read");
        assert_eq!(data, b"hello");

        // insert multi-frame element
        let long_data = (0..1025).map(|_| 1 as u8).collect::<Vec<u8>>();
        let position = backend.create(&long_data).expect("could not create");
        assert_eq!(position, 16 + 1024);

        // confirm by reading back
        let long_data = backend.read(position).expect("could not read");
        assert_eq!(long_data.len(), 1025);
    }

    #[test]
    fn update() {
        // prepare
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut backend = Backend::new(file).expect("could not create mmap");

        // insert multi-frame element
        let long_data = (0..1025).map(|_| 1 as u8).collect::<Vec<u8>>();
        let position = backend.create(&long_data).expect("could not create");
        assert_eq!(position, 16);

        // confirm by reading back
        let long_data = backend.read(position).expect("could not read");
        assert_eq!(long_data.len(), 1025);

        // update with simple element
        let data = (0..10).map(|_| 1 as u8).collect::<Vec<u8>>();
        backend.update(position, &data).expect("could not create");
        assert_eq!(position, 16);

        // confirm by reading back
        let data = backend.read(position).expect("could not read");
        assert_eq!(data.len(), 10);

        // confirm "next" frame got "deleted"
        let data = backend.read(position + 1024).expect("could not read");
        assert_eq!(data.len(), 0);
    }
}
