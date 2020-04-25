use super::Backend;
use memmap2::MmapMut;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::ops::RangeTo;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub frame_count: usize,
    pub version: usize,
}

impl Header {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn first_frame_position() -> usize {
        Self::size()
    }

    pub fn read(mmap: &MmapMut) -> Result<Self, Box<dyn Error>> {
        let end = Header::size();
        let range = RangeTo { end };
        let bytes = &mmap[range];
        Ok(bincode::deserialize_from(bytes)?)
    }

    pub fn update(&self, mmap: &mut MmapMut) -> Result<(), Box<dyn Error>> {
        let end = Header::size();
        let range = RangeTo { end };
        let bytes: Vec<u8> = bincode::serialize(&self)?;
        (&mut mmap[range]).write_all(&bytes)?;
        Ok(())
    }
}

impl Backend {
    pub fn initialize_header(mapped_file: &mut MmapMut) -> Result<Header, Box<dyn Error>> {
        let end = Header::size();
        let range = RangeTo { end };
        let bytes = &mapped_file[range];
        let mut header: Header = bincode::deserialize_from(bytes)?;
        if header.version == 0 {
            header.version = 1;
            header.update(mapped_file)?;
        }
        Ok(header)
    }
}
