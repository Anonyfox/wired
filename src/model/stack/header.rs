use crate::backend::StaticBlock;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    current_next_element: usize,
    element_count: usize,
}

impl StaticBlock for Header {
    fn start(&self) -> usize {
        0
    }
}

impl Header {
    pub fn get_current_ptr(&self) -> usize {
        self.current_next_element
    }

    pub fn set_current_ptr(&mut self, ptr: usize) {
        self.current_next_element = ptr;
    }

    pub fn element_count(&self) -> usize {
        self.element_count
    }

    pub fn inc_counter(&mut self) {
        self.element_count += 1
    }

    pub fn dec_counter(&mut self) {
        self.element_count -= 1
    }
}
