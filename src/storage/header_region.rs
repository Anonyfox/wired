use super::Backend;
use memmap2::MmapMut;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::ops::RangeTo;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HeaderRegion {
    pub frame_count: usize,
    pub version: usize,
}

impl HeaderRegion {
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn first_frame_position() -> usize {
        Self::size()
    }

    pub fn read(mmap: &MmapMut) -> Result<Self, Box<dyn Error>> {
        let end = HeaderRegion::size();
        let range = RangeTo { end };
        let bytes = &mmap[range];
        Ok(bincode::deserialize_from(bytes)?)
    }

    pub fn update(&self, mmap: &mut MmapMut) -> Result<(), Box<dyn Error>> {
        let end = HeaderRegion::size();
        let range = RangeTo { end };
        let bytes: Vec<u8> = bincode::serialize(&self)?;
        (&mut mmap[range]).write_all(&bytes)?;
        Ok(())
    }
}

impl Backend {
    pub fn initialize_header_region(
        mapped_file: &MmapMut,
    ) -> Result<HeaderRegion, Box<dyn std::error::Error>> {
        let end = HeaderRegion::size();
        let range = RangeTo { end };
        let bytes = &mapped_file[range];
        Ok(bincode::deserialize_from(bytes)?)
    }

    pub fn update_header(&mut self, header: &HeaderRegion) -> Result<(), Box<dyn Error>> {
        let end = HeaderRegion::size();
        let range = RangeTo { end };
        let bytes: Vec<u8> = bincode::serialize(header)?;
        (&mut self.mapped_file[range]).write_all(&bytes)?;
        Ok(())
    }
}
