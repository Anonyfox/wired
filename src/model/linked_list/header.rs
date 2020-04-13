use crate::backend::StaticBlock;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    first_node_ptr: usize,
    last_node_ptr: usize,
    element_count: usize,
    allocate_ptr: usize,
}

impl StaticBlock for Header {
    fn start(&self) -> usize {
        0
    }
}

impl Header {
    pub fn get_first_node_ptr(&self) -> usize {
        self.first_node_ptr
    }

    pub fn set_first_node_ptr(&mut self, ptr: usize) {
        self.first_node_ptr = ptr;
    }

    pub fn get_last_node_ptr(&self) -> usize {
        self.last_node_ptr
    }

    pub fn set_last_node_ptr(&mut self, ptr: usize) {
        self.last_node_ptr = ptr;
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

    pub fn get_allocator(&self) -> usize {
        self.allocate_ptr
    }

    pub fn set_allocator(&mut self, ptr: usize) {
        self.allocate_ptr = ptr;
    }
}
