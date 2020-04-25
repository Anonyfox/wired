use super::Backend;
use memmap2::{MmapMut, MmapOptions};
use std::error::Error;
use std::fs::File;

impl Backend {
    pub fn open_file(file: &File) -> Result<(usize, MmapMut), Box<dyn Error>> {
        let size = ensure_minimum_file_size(&file)?;
        let mapped_file = create_file_mapping(&file, size)?;
        Ok((size, mapped_file))
    }

    pub fn resize_file(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.size * 2;
        self.file.set_len(new_size as u64)?;
        self.size = new_size;
        let new_mapped_file = create_file_mapping(&self.file, new_size)?;
        self.mapped_file = new_mapped_file;
        Ok(())
    }

    pub fn flush(&self) -> Result<(), Box<dyn Error>> {
        let result = self.mapped_file.flush()?;
        Ok(result)
    }
}

fn create_file_mapping(file: &File, size: usize) -> Result<MmapMut, Box<dyn Error>> {
    let mapped_file = unsafe { MmapOptions::new().len(size).map_mut(&file)? };
    Ok(mapped_file)
}

fn ensure_minimum_file_size(file: &File) -> Result<usize, Box<dyn Error>> {
    let current_size: usize = file.metadata()?.len() as usize;
    if current_size == 0 {
        let min_size: usize = page_size::get();
        file.set_len(min_size as u64)?;
        Ok(min_size)
    } else {
        Ok(current_size)
    }
}
