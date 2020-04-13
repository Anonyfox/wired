use super::Backend;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub trait StaticBlock
where
    Self: Serialize,
    for<'de> Self: Deserialize<'de>,
{
    fn start(&self) -> usize;

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn load(backend: &dyn Backend, start: usize) -> Result<Self, Box<dyn Error>> {
        let range = std::ops::Range {
            start: start,
            end: start + Self::size(),
        };
        Ok(bincode::deserialize_from(backend.read(range))?)
    }

    fn save(&self, backend: &mut dyn Backend) -> Result<usize, Box<dyn Error>> {
        let range = std::ops::Range {
            start: self.start(),
            end: self.start() + Self::size(),
        };
        let bytes: Vec<u8> = bincode::serialize(&self)?;
        backend.write(range, &bytes)?;
        Ok(bytes.len())
    }
}
