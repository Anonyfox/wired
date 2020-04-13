mod header;
mod node;

use super::Model;
use crate::backend::{Backend, DataBlock, StaticBlock};
use header::Header;
use node::Node;
use std::error::Error;
use std::marker::PhantomData;

pub struct LinkedList<T> {
    header: Header,
    backend: Box<dyn Backend>,
    data_type: PhantomData<T>,
}

impl<T> Model for LinkedList<T> {}

impl<T> LinkedList<T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
{
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let backend = Self::connect_backend(&path)?;
        Self::initialize_state(backend)
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
            data_type: PhantomData,
            // data_type: PhantomData,
        })
    }

    pub fn count(&self) -> usize {
        self.header.element_count()
    }

    pub fn first_node(&self) -> Result<Option<Node<T>>, Box<dyn Error>> {
        let position = self.header.get_first_node_ptr();
        if position == 0 {
            Ok(None)
        } else {
            let node = Node::load(&*self.backend, position)?;
            Ok(Some(node))
        }
    }

    pub fn last_node(&self) -> Result<Option<Node<T>>, Box<dyn Error>> {
        let position = self.header.get_last_node_ptr();
        if position == 0 {
            Ok(None)
        } else {
            let node = Node::load(&*self.backend, position)?;
            Ok(Some(node))
        }
    }

    pub fn push(&mut self, data: &T) -> Result<Node<T>, Box<dyn Error>> {
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut last_node) = self.last_node()? {
            last_node.set_next(&mut *self.backend, &new_node)?;
            new_node.set_prev(&mut *self.backend, &last_node)?;
        } else {
            new_node.save(&mut *self.backend)?;
            self.header.set_first_node_ptr(new_node.start());
        };
        new_node.data_store(&mut *self.backend, &data)?;
        self.header.set_last_node_ptr(new_node.start());
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.update_allocator(&new_node)?;
        self.backend.persist()?;
        Ok(new_node)
    }

    pub fn pop(&mut self) -> Result<Option<Node<T>>, Box<dyn Error>> {
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
    pub fn unshift(&mut self, data: &T) -> Result<Node<T>, Box<dyn Error>> {
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut first_node) = self.first_node()? {
            first_node.set_prev(&mut *self.backend, &new_node)?;
            new_node.set_next(&mut *self.backend, &first_node)?;
        } else {
            self.header.set_last_node_ptr(new_node.start());
        }
        new_node.data_store(&mut *self.backend, &data)?;
        self.header.set_first_node_ptr(new_node.start());
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.update_allocator(&new_node)?;
        self.backend.persist()?;
        Ok(new_node)
    }

    // removes one item from front
    pub fn shift(&mut self) -> Result<Option<Node<T>>, Box<dyn Error>> {
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

    fn update_allocator(&mut self, node: &Node<T>) -> Result<(), Box<dyn Error>> {
        let position = node.start() + Node::<T>::size() + node.data_size();
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
        let mut list = LinkedList::<String>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        list.push(&"hello".to_string()).expect("couldn't push");
        assert_eq!(list.count(), 1);

        list.push(&"world".to_string()).expect("couldn't push");
        assert_eq!(list.count(), 2);

        let node = list.pop().expect("couldn't pop");
        let content = node.unwrap().data_fetch(&*list.backend).unwrap();
        assert_eq!(content, "world");
        assert_eq!(list.count(), 1);

        let node = list.pop().expect("couldn't pop");
        let content = node.unwrap().data_fetch(&*list.backend).unwrap();
        assert_eq!(content, "hello");
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn shift_and_unshift() {
        let mut list = LinkedList::<usize>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        list.unshift(&1).expect("couldn't unshift");
        assert_eq!(list.count(), 1);

        list.unshift(&2).expect("couldn't unshift");
        assert_eq!(list.count(), 2);

        let node = list.shift().expect("couldn't shift").unwrap();
        let content = node.data_fetch(&*list.backend).unwrap();
        assert_eq!(content, 2);
        assert_eq!(list.count(), 1);

        let node = list.shift().expect("couldn't shift").unwrap();
        let content = node.data_fetch(&*list.backend).unwrap();
        assert_eq!(content, 1);
        assert_eq!(list.count(), 0);
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Example {
        name: String,
        score: usize,
    }

    #[test]

    fn generic_data() {
        let mut list = LinkedList::<Example>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        let example = Example {
            name: "john".to_owned(),
            score: 42,
        };
        list.push(&example).expect("couldn't push");
        assert_eq!(list.count(), 1);

        let example = Example {
            name: "hans".to_owned(),
            score: 43,
        };
        list.unshift(&example).expect("couldn't unshift");
        assert_eq!(list.count(), 2);

        let node = list.pop().expect("couldn't pop").unwrap();
        let content = node.data_fetch(&*list.backend).unwrap();
        assert_eq!(content.name, "john");
        assert_eq!(list.count(), 1);

        let node = list.shift().expect("couldn't pop").unwrap();
        let content = node.data_fetch(&*list.backend).unwrap();
        assert_eq!(content.name, "hans");
        assert_eq!(list.count(), 0);
    }
}
