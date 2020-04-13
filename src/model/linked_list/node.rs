use crate::backend::{Backend, DataBlock, StaticBlock};
use serde::{Deserialize, Serialize};
use std::error::Error;
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

    pub fn create(backend: &mut dyn Backend, position: usize) -> Result<Self, Box<dyn Error>> {
        let node = Self::new(position);
        node.save(&mut *backend)?;
        Ok(node)
    }

    pub fn next(&self, backend: &dyn Backend) -> Result<Option<Self>, Box<dyn Error>> {
        if self.next_ptr == 0 {
            Ok(None)
        } else {
            let node = Self::load(&*backend, self.next_ptr)?;
            Ok(Some(node))
        }
    }

    pub fn prev(&self) -> Option<Self> {
        if self.prev_ptr == 0 {
            None
        } else {
            Some(Self::new(self.prev_ptr))
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

impl StaticBlock for Node {
    fn start(&self) -> usize {
        self.position
    }
}
