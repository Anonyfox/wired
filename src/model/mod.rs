use super::backend::{Backend, Mmap};
use std::error::Error;

mod linked_list;
pub use linked_list::LinkedList;

pub trait Model {
    #[cfg(test)]
    fn connect_backend(_path: &str) -> Result<Box<dyn Backend>, Box<dyn Error>> {
        let file = tempfile::tempfile()?;
        let backend = Mmap::new(file)?;
        Ok(Box::new(backend))
    }

    #[cfg(not(test))]
    fn connect_backend(path: &str) -> Result<Box<dyn Backend>, Box<dyn Error>> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let backend = Mmap::new(file)?;
        Ok(Box::new(backend))
    }
}
