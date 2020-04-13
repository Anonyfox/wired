use crate::backend::{Backend, DataBlock, StaticBlock};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node<T> {
    position: usize,
    next_ptr: usize,
    prev_ptr: usize,
    data_type: PhantomData<T>,
    data_size: usize,
}

impl<T> Node<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    pub fn new(position: usize) -> Self {
        Self {
            position,
            next_ptr: 0,
            prev_ptr: 0,
            data_type: PhantomData,
            data_size: 0,
        }
    }

    pub fn create(backend: &mut dyn Backend, position: usize) -> Result<Self, Box<dyn Error>> {
        let node = Self::new(position);
        node.save(&mut *backend)?;
        Ok(node)
    }

    pub fn is_first(&self) -> bool {
        self.prev_ptr == 0
    }

    pub fn is_last(&self) -> bool {
        self.next_ptr == 0
    }

    pub fn next(&self, backend: &dyn Backend) -> Result<Option<Self>, Box<dyn Error>> {
        if self.next_ptr == 0 {
            Ok(None)
        } else {
            let node = Self::load(&*backend, self.next_ptr)?;
            Ok(Some(node))
        }
    }

    pub fn prev(&self, backend: &dyn Backend) -> Result<Option<Self>, Box<dyn Error>> {
        if self.prev_ptr == 0 {
            Ok(None)
        } else {
            let node = Self::load(&*backend, self.prev_ptr)?;
            Ok(Some(node))
        }
    }

    pub fn set_next(
        &mut self,
        backend: &mut dyn Backend,
        other: &Self,
    ) -> Result<(), Box<dyn Error>> {
        self.next_ptr = other.position;
        self.save(&mut *backend)?;
        Ok(())
    }

    pub fn set_next_empty(&mut self, backend: &mut dyn Backend) -> Result<(), Box<dyn Error>> {
        self.next_ptr = 0;
        self.save(&mut *backend)?;
        Ok(())
    }

    pub fn set_prev(
        &mut self,
        backend: &mut dyn Backend,
        other: &Self,
    ) -> Result<(), Box<dyn Error>> {
        self.prev_ptr = other.position;
        self.save(&mut *backend)?;
        Ok(())
    }

    pub fn set_prev_empty(&mut self, backend: &mut dyn Backend) -> Result<(), Box<dyn Error>> {
        self.prev_ptr = 0;
        self.save(&mut *backend)?;
        Ok(())
    }
}

impl<T> StaticBlock for Node<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    fn start(&self) -> usize {
        self.position
    }
}

impl<T> DataBlock<T> for Node<T>
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
