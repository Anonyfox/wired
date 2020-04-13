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
        let mut header = Header::load(&*backend, 0)?;
        if header.element_count() == 0 {
            header.set_allocator(Header::size());
        }
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
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut last_node) = self.last_node()? {
            last_node.set_next(&mut *self.backend, &new_node)?;
            new_node.set_prev(&mut *self.backend, &last_node)?;
        } else {
            new_node.save(&mut *self.backend)?;
            self.header.set_first_node_ptr(new_node.start());
        };
        self.header.set_last_node_ptr(new_node.start());
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.update_allocator(&new_node)?;
        self.backend.persist()?;
        Ok(new_node)
    }

    pub fn pop(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        if let Some(last_node) = self.last_node()? {
            if let Some(mut prev_node) = last_node.prev() {
                prev_node.init(&*self.backend)?;
                self.header.set_last_node_ptr(prev_node.start());
                prev_node.set_next_empty(&mut *self.backend)?;
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

    // adds one item in front
    pub fn unshift(&mut self) -> Result<Node, Box<dyn Error>> {
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut first_node) = self.first_node()? {
            first_node.set_prev(&mut *self.backend, &new_node)?;
            new_node.set_next(&mut *self.backend, &first_node)?;
        } else {
            self.header.set_last_node_ptr(new_node.start());
        }
        self.header.set_first_node_ptr(new_node.start());
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.update_allocator(&new_node)?;
        self.backend.persist()?;
        Ok(new_node)
    }

    // removes one item from front
    pub fn shift(&mut self) -> Result<Option<Node>, Box<dyn Error>> {
        if let Some(first_node) = self.first_node()? {
            if let Some(mut next_node) = first_node.next(&*self.backend)? {
                self.header.set_first_node_ptr(next_node.start());
                next_node.set_prev_empty(&mut *self.backend)?;
            } else {
                self.header.set_last_node_ptr(0);
                self.header.set_first_node_ptr(0);
            }
            self.header.dec_counter();
            self.header.save(&mut *self.backend)?;
            self.backend.persist()?;
            Ok(Some(first_node))
        } else {
            Ok(None)
        }
    }

    fn update_allocator(&mut self, node: &Node) -> Result<(), Box<dyn Error>> {
        let position = node.start() + Node::size();
        if position > self.header.get_allocator() {
            self.header.set_allocator(position);
            self.header.save(&mut *self.backend)?;
            self.backend.persist()?;
        }
        Ok(())
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

        list.unshift().expect("couldn't unshift");
        assert_eq!(list.count(), 1);

        list.unshift().expect("couldn't unshift");
        assert_eq!(list.count(), 2);

        list.shift().expect("couldn't shift");
        assert_eq!(list.count(), 1);

        list.shift().expect("couldn't shift");
        assert_eq!(list.count(), 0);
    }

    #[test]
    #[ignore]
    fn generic_data() {
        let list = LinkedList::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);
    }
}
