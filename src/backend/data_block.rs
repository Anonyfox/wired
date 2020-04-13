use super::Backend;
use super::StaticBlock;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub trait DataBlock<T>: StaticBlock
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    fn data_size(&self) -> usize;
    fn set_data_size(&mut self, size: usize);

    fn data_fetch(&self, backend: &dyn Backend) -> Result<T, Box<dyn Error>> {
        let range = self.data_range();
        Ok(bincode::deserialize_from(backend.read(range))?)
    }

    fn data_store(&mut self, backend: &mut dyn Backend, data: &T) -> Result<usize, Box<dyn Error>> {
        let bytes: Vec<u8> = bincode::serialize(&data)?;
        self.set_data_size(bytes.len());
        self.save(&mut *backend)?;
        let range = self.data_range();
        backend.write(range, &bytes)?;
        Ok(bytes.len())
    }

    fn data_position(&self) -> usize {
        self.start() + Self::size()
    }

    fn data_range(&self) -> std::ops::Range<usize> {
        let data_position = self.data_position();
        std::ops::Range {
            start: data_position,
            end: data_position + self.data_size(),
        }
    }
}
