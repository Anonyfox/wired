use std::error::Error;
use std::ops::Range;

mod data_block;
mod mmap;
mod static_block;

pub use data_block::DataBlock;
pub use mmap::Mmap;
pub use static_block::StaticBlock;

pub trait Backend {
    fn read(&self, range: Range<usize>) -> &[u8];
    fn write(&mut self, range: Range<usize>, bytes: &[u8]) -> Result<(), Box<dyn Error>>;
    fn persist(&mut self) -> Result<(), Box<dyn Error>>;
}

// pub fn get_file_handle(path: &Path) -> Result<fs::File, Box<dyn Error>> {
//     let file = fs::OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open(&path)?;
//     Ok(file)
// }
