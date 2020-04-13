use crate::backend::{DataBlock, StaticBlock};
use serde::{Deserialize, Serialize};
// use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    position: usize,
    next_ptr: usize,
    prev_ptr: usize,
}

impl Node {
    pub fn new(position: usize) -> Self {
        Self {
            position,
            next_ptr: 0,
            prev_ptr: 0,
        }
    }

    pub fn ptr(&self) -> usize {
        self.position
    }

    pub fn next(&self) -> Option<Self> {
        if self.next_ptr == 0 {
            None
        } else {
            Some(Self::new(self.next_ptr))
        }
    }

    pub fn prev(&self) -> Option<Self> {
        if self.prev_ptr == 0 {
            None
        } else {
            Some(Self::new(self.prev_ptr))
        }
    }

    pub fn set_next(&mut self, other: &Self) {
        self.next_ptr = other.position;
    }

    pub fn set_next_empty(&mut self) {
        self.next_ptr = 0;
    }

    pub fn set_prev(&mut self, other: &Self) {
        self.prev_ptr = other.position;
    }

    pub fn set_prev_empty(&mut self) {
        self.next_ptr = 0;
    }
}

impl StaticBlock for Node {
    fn start(&self) -> usize {
        self.position
    }
}
