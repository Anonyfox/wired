use super::Backend;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::ops::Range;

const FRAME_SIZE: usize = 1024;
// const FRAME_SIZE: usize = 32 * 1024;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Frame {
    // first byte position in file of this frame
    pub position: usize,
    // length of the body
    pub body_size: usize,
    // is this block deleted?
    pub deleted: bool,
    // if not 0, read the next block in addition to this one and treat them as one logical unit
    pub next: usize,
}

impl Frame {
    pub fn header_size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn capacity() -> usize {
        FRAME_SIZE - Self::header_size()
    }

    pub fn free_bytes(&self) -> usize {
        Self::capacity() - self.body_size
    }

    pub fn total_size() -> usize {
        FRAME_SIZE
    }
}

impl Backend {
    pub fn create_frame(&mut self, position: usize) -> Result<Frame, Box<dyn Error>> {
        let frame = Frame {
            position: position,
            deleted: false,
            next: 0,
            body_size: 0,
        };
        self.update_frame(frame)?;
        self.read_frame(position)
    }

    pub fn read_frame(&self, position: usize) -> Result<Frame, Box<dyn Error>> {
        let start = position;
        let end = Frame::header_size() + position;
        let range = Range { start, end };
        let bytes = &self.mapped_file[range];
        Ok(bincode::deserialize_from(bytes)?)
    }

    pub fn update_frame(&mut self, frame: Frame) -> Result<(), Box<dyn Error>> {
        let start = frame.position;
        let end = Frame::header_size() + frame.position;
        let range = Range { start, end };
        let bytes: Vec<u8> = bincode::serialize(&frame)?;
        (&mut self.mapped_file[range]).write_all(&bytes)?;
        Ok(())
    }

    pub fn read_frame_body(&self, position: usize) -> Result<&[u8], Box<dyn Error>> {
        let frame = self.read_frame(position)?;
        let start = Frame::header_size() + position;
        let end = start + frame.body_size;
        let range = Range { start, end };
        Ok(&self.mapped_file[range])
    }

    pub fn write_frame_body(
        &mut self,
        position: usize,
        bytes: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let body_size = bytes.len();
        let mut frame = self.read_frame(position)?;
        let start = frame.position + Frame::header_size();
        let end = start + body_size;
        let range = Range { start, end };
        (&mut self.mapped_file[range]).write_all(&bytes)?;
        frame.body_size = body_size;
        self.update_frame(frame)?;
        Ok(())
    }
}
