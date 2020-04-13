mod header;
mod node;

use crate::backend::{Backend, DataBlock, Mmap, StaticBlock};
use header::Header;
use node::Node;
use std::error::Error;

pub struct LinkedList {
    header: Header,
    backend: Box<dyn Backend>,
}

impl LinkedList {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let backend = Self::connect_backend(&path)?;
        Self::initialize_state(backend)
    }

    #[cfg(test)]
    fn connect_backend(_path: &str) -> Result<Box<dyn Backend>, Box<dyn Error>> {
        let file = tempfile::tempfile()?;
        let backend = Mmap::new(file)?;
        Ok(Box::new(backend))
    }

    #[cfg(not(test))]
    fn connect_backend(path: &str) -> Result<Box<dyn Backend>, Box<dyn Error>> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let backend = Mmap::new(file)?;
        Ok(Box::new(backend))
    }

    fn initialize_state(mut backend: Box<dyn Backend>) -> Result<Self, Box<dyn Error>> {
        let header = Header::load(&*backend, 0)?;
        header.save(&mut *backend)?;
        Ok(Self {
            header,
            backend,
            // data_type: PhantomData,
        })
    }

    pub fn count(&self) -> usize {
        self.header.element_count()
    }

    pub fn first_node(&self) -> Result<Option<Node>, Box<dyn Error>> {
        let position = self.header.get_first_node_ptr();
        if position == 0 {
            Ok(None)
        } else {
            let node = Node::load(&*self.backend, position)?;
            Ok(Some(node))
        }
    }

    pub fn last_node(&self) -> Result<Option<Node>, Box<dyn Error>> {
        let position = self.header.get_last_node_ptr();
        if position == 0 {
            Ok(None)
        } else {
            let node = Node::load(&*self.backend, position)?;
            Ok(Some(node))
        }
    }

    pub fn push(&mut self) -> Result<Node, Box<dyn Error>> {
        let position = self.next_free_position()?;
        let mut new_node = Node::new(position);
        if let Some(mut last_node) = self.last_node()? {
            last_node.set_next(&new_node);
            last_node.save(&mut *self.backend)?;
            new_node.set_prev(&last_node);
        } else {
            self.header.set_first_node_ptr(new_node.start());
        };
        new_node.save(&mut *self.backend)?;
        self.header.set_last_node_ptr(new_node.start());
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.backend.persist()?;
        Ok(new_node)
    }

    pub fn pop(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        if let Some(last_node) = self.last_node()? {
            dbg!(&last_node);
            if let Some(mut prev_node) = last_node.prev() {
                prev_node.init(&*self.backend)?;
                self.header.set_last_node_ptr(prev_node.start());
                prev_node.set_next_empty();
                prev_node.save(&mut *self.backend)?;
            } else {
                self.header.set_last_node_ptr(0);
                self.header.set_first_node_ptr(0);
            }
            self.header.dec_counter();
            self.header.save(&mut *self.backend)?;
            self.backend.persist()?;
            Ok(Some(last_node))
        } else {
            Ok(None)
        }
    }

    pub fn unshift(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        // adds one item in front
        // impl after space management is done
        Ok(None)
    }

    pub fn shift(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        // removes one item from front
        // impl after space management is done
        Ok(None)
    }

    fn next_free_position(&self) -> Result<usize, Box<dyn Error>> {
        if let Some(last_node) = self.last_node()? {
            // TODO: this is VERY naive for now
            let position = last_node.start() + Node::size();
            Ok(position)
        } else {
            Ok(Header::size())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop() {
        let mut list = LinkedList::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        list.push().expect("couldn't push");
        assert_eq!(list.count(), 1);

        list.push().expect("couldn't push");
        assert_eq!(list.count(), 2);

        list.pop().expect("couldn't pop");
        assert_eq!(list.count(), 1);

        list.pop().expect("couldn't pop");
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn shift_and_unshift() {
        let mut list = LinkedList::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);
    }
}
