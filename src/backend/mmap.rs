use super::Backend;
use memmap2::{MmapMut, MmapOptions};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Range;

pub struct Mmap {
    size: usize,
    mapped_file: MmapMut,
    file: File,
}

impl Mmap {
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let min_size: usize = 1024;
        let size: usize = file.metadata()?.len() as usize;
        if size == 0 {
            file.set_len(min_size as u64)?;
        }
        let size = std::cmp::max(size, min_size);
        let mapped_file = Self::create_file_mapping(&file, size)?;
        let mmap = Self {
            file,
            mapped_file,
            size,
        };
        Ok(mmap)
    }

    fn create_file_mapping(file: &File, size: usize) -> Result<MmapMut, Box<dyn Error>> {
        let mapped_file = unsafe { MmapOptions::new().len(size).map_mut(&file)? };
        Ok(mapped_file)
    }

    fn resize_mapped_file(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.size * 2;
        self.file.set_len(new_size as u64)?;
        self.size = new_size;
        let new_mapped_file = Self::create_file_mapping(&self.file, new_size)?;
        self.mapped_file = new_mapped_file;
        Ok(())
    }
}

impl Backend for Mmap {
    fn read(&self, range: Range<usize>) -> &[u8] {
        &self.mapped_file[range]
    }

    fn write(&mut self, range: Range<usize>, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        if range.end > self.size {
            self.resize_mapped_file()?;
        }
        (&mut self.mapped_file[range]).write_all(&bytes)?;
        Ok(())
    }

    fn persist(&mut self) -> Result<(), Box<dyn Error>> {
        let result = self.mapped_file.flush()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn works() {
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut mmap = Mmap::new(file).expect("could not create mmap");
        let example_range = Range {
            start: 0 as usize,
            end: 5 as usize,
        };
        mmap.write(example_range, b"hello")
            .expect("could not write");
        mmap.persist().expect("could not flush");
        let example_range = Range {
            start: 0 as usize,
            end: 5 as usize,
        };
        let content = mmap.read(example_range);
        assert_eq!(content, b"hello");
    }

    #[test]
    fn does_resize_if_needed() {
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut mmap = Mmap::new(file).expect("could not create mmap");
        for i in 0..1023 as usize {
            let example_range = Range {
                start: i as usize,
                end: i + 1 as usize,
            };
            mmap.write(example_range, b"1")
                .expect("could not write byte");
        }
        let example_range = Range {
            start: 1024 as usize,
            end: 1030 as usize,
        };
        mmap.write(example_range, b"hello!")
            .expect("could not write additional bytes");
    }
}
