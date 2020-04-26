mod backend;

use backend::Backend;
use std::error::Error;
use std::fs::File;

pub struct BlockStorage {
    backend: Backend,
}

impl BlockStorage {
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let backend = Backend::new(file)?;
        Ok(Self { backend })
    }

    pub fn create(&mut self, bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
        let position = self.backend.create(bytes)?;
        let index = position_to_index(position);
        Ok(index)
    }

    pub fn read(&self, index: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let position = index_to_position(index);
        self.backend.read(position)
    }

    pub fn update(&mut self, index: usize, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        let position = index_to_position(index);
        self.backend.update(position, bytes)
    }

    pub fn delete(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        let position = index_to_position(index);
        self.backend.delete(position)
    }

    pub fn is_empty(&self) -> bool {
        self.backend.is_empty()
    }

    // pub fn list_indices(&self) -> Result<Vec<usize>, Box<dyn Error>> {
    //     let positions = self.backend.collect_head_nodes()?;
    //     let indexes = positions
    //         .iter()
    //         .map(|pos| position_to_index(*pos))
    //         .collect();
    //     Ok(indexes)
    // }
}

fn position_to_index(position: usize) -> usize {
    (position - Backend::offset()) / Backend::block_size()
}

fn index_to_position(index: usize) -> usize {
    Backend::offset() + Backend::block_size() * index
}
