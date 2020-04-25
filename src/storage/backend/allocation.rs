use super::frames::Frame;
use super::header::Header;
use super::Backend;
use std::error::Error;

impl Backend {
    /// runtime: O(n)
    pub fn next_free_frame_position(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut result: Option<usize> = None;
        let mut position = Header::first_frame_position();
        let max_position = self.header.frame_count * Frame::total_size();
        while result.is_none() && position < max_position {
            let frame = self.read_frame(position)?;
            if frame.deleted == true {
                result = Some(position);
            } else {
                position += Frame::total_size();
            }
        }
        if let Some(next_free_position) = result {
            Ok(next_free_position)
        } else {
            let next_free_position = Header::size() + self.header.frame_count * Frame::total_size();
            self.header.frame_count += 1;
            self.header.update(&mut self.mapped_file)?;
            if (next_free_position + Frame::total_size()) > self.size {
                self.resize_file()?;
            }
            Ok(next_free_position)
        }
    }
}
