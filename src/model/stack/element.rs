use crate::backend::{DataBlock, StaticBlock};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug)]
pub struct Element<T> {
    position: usize,
    prev: usize,
    data_size: usize,
    data_type: PhantomData<T>,
}

impl<T> StaticBlock for Element<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    fn start(&self) -> usize {
        self.position
    }
}

impl<T> DataBlock<T> for Element<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    fn data_size(&self) -> usize {
        self.data_size
    }

    fn set_data_size(&mut self, size: usize) {
        self.data_size = size;
    }
}

impl<T> Element<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    pub fn new(position: usize) -> Self {
        Self {
            position,
            prev: 0,
            data_size: 0,
            data_type: PhantomData,
        }
    }

    pub fn get_prev_ptr(&self) -> usize {
        self.prev
    }

    pub fn set_prev_ptr(&mut self, ptr: usize) {
        self.prev = ptr;
    }

    pub fn get_ptr(&self) -> usize {
        self.position
    }
}
