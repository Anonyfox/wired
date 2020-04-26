use super::frames::Frame;
use super::header::Header;
use super::Backend;
use std::error::Error;

impl Backend {
    /// allocator with runtime O(1)
    pub fn next_free_frame(&mut self) -> Result<usize, Box<dyn Error>> {
        // try to use an existing frame that got "deleted"
        if self.header.first_free_frame != 0 {
            let frame = self.read_frame(self.header.first_free_frame)?;
            self.header.first_free_frame = frame.next;
            self.header.update(&mut self.mapped_file)?;
            Ok(frame.position)
        // or allocate more memory
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

    /// remove a frame from the list of deleted frames, making it an orphan.
    ///
    /// this should be used with great care, since this memory frame will never
    /// be reclaimed again if not used immediately.
    pub fn unlink_free_frame(&mut self, position: usize) -> Result<usize, Box<dyn Error>> {
        let mut cursor: usize = self.header.first_free_frame;
        let mut prev: Option<Frame> = None;
        while cursor != 0 {
            let frame = self.read_frame(cursor)?;
            if frame.position == position {
                if let Some(mut prev) = prev {
                    prev.next = frame.next;
                    self.update_frame(prev)?;
                    break;
                } else {
                    self.header.first_free_frame = frame.next;
                    self.header.update(&mut self.mapped_file)?;
                    break;
                }
            } else {
                cursor = frame.next;
                prev = Some(frame);
            }
        }
        Ok(position)
    }
}
