mod header;
mod node;

use super::Model;
use crate::backend::{Backend, DataBlock, StaticBlock};
use header::Header;
use node::Node;
use std::clone::Clone;
use std::marker::PhantomData;

type Error = Box<dyn std::error::Error>;

pub struct LinkedList<T> {
    header: Header,
    path: String,
    backend: Box<dyn Backend>,
    data_type: PhantomData<T>,
}

impl<T> Model for LinkedList<T> {}

impl<T> LinkedList<T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
{
    pub fn new(path: &str) -> Result<Self, Error> {
        let backend = Self::connect_backend(&path)?;
        Self::initialize_state(path, backend)
    }

    fn initialize_state(path: &str, mut backend: Box<dyn Backend>) -> Result<Self, Error> {
        let mut header = Header::load(&*backend, 0)?;
        if header.element_count() == 0 {
            header.set_allocator(Header::size());
        }
        header.save(&mut *backend)?;
        Ok(Self {
            header,
            path: path.to_string(),
            backend,
            data_type: PhantomData,
        })
    }

    pub fn compact(&mut self) -> Result<(), Error> {
        let mut new_path = self.path.clone();
        new_path.push_str(".tmp");
        let mut new_list = Self::new(&new_path)?;
        for node in self.iter()? {
            let data = self.get_node_data(&node)?;
            new_list.insert_end(&data)?;
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.header.element_count()
    }

    pub fn allocated_bytes(&self) -> usize {
        self.header.get_allocator()
    }

    pub fn used_bytes(&self) -> usize {
        let max = self.allocated_bytes();
        let unused = self.header.get_unused_bytes();
        max - unused
    }

    pub fn first_node(&self) -> Result<Option<Node<T>>, Error> {
        let position = self.header.get_first_node_ptr();
        let node = Node::load(&*self.backend, position)?;
        Ok(Some(node))
    }

    pub fn last_node(&self) -> Result<Option<Node<T>>, Error> {
        let position = self.header.get_last_node_ptr();
        let node = Node::load(&*self.backend, position)?;
        Ok(Some(node))
    }

    pub fn insert_before(&mut self, node: &mut Node<T>, data: &T) -> Result<Node<T>, Error> {
        let position = self.header.get_allocator();
        let mut new_node = Node::<T>::create(&mut *self.backend, position)?;
        if let Some(mut prev_node) = node.prev(&*self.backend)? {
            prev_node.set_next(&mut *self.backend, &new_node)?;
            new_node.set_prev(&mut *self.backend, &prev_node)?;
        }
        node.set_prev(&mut *self.backend, &new_node)?;
        new_node.set_next(&mut *self.backend, &node)?;
        new_node.data_store(&mut *self.backend, &data)?;
        self.finalize_insert(&new_node)?;
        Ok(new_node)
    }

    pub fn insert_after(&mut self, node: &mut Node<T>, data: &T) -> Result<Node<T>, Error> {
        let position = self.header.get_allocator();
        let mut new_node = Node::<T>::create(&mut *self.backend, position)?;
        if let Some(mut next_node) = node.next(&*self.backend)? {
            next_node.set_prev(&mut *self.backend, &new_node)?;
            new_node.set_next(&mut *self.backend, &next_node)?;
        }
        node.set_next(&mut *self.backend, &new_node)?;
        new_node.set_prev(&mut *self.backend, &node)?;
        new_node.data_store(&mut *self.backend, &data)?;
        self.finalize_insert(&new_node)?;
        Ok(new_node)
    }

    pub fn insert_start(&mut self, data: &T) -> Result<Node<T>, Error> {
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut first_node) = self.first_node()? {
            first_node.set_prev(&mut *self.backend, &new_node)?;
            new_node.set_next(&mut *self.backend, &first_node)?;
        }
        new_node.data_store(&mut *self.backend, &data)?;
        self.finalize_insert(&new_node)?;
        Ok(new_node)
    }

    pub fn insert_end(&mut self, data: &T) -> Result<Node<T>, Error> {
        let position = self.header.get_allocator();
        let mut new_node = Node::create(&mut *self.backend, position)?;
        if let Some(mut last_node) = self.last_node()? {
            last_node.set_next(&mut *self.backend, &new_node)?;
            new_node.set_prev(&mut *self.backend, &last_node)?;
        };
        new_node.data_store(&mut *self.backend, &data)?;
        self.finalize_insert(&new_node)?;
        Ok(new_node)
    }

    pub fn remove(&mut self, node: Node<T>) -> Result<(), Error> {
        if let Some(mut prev_node) = node.prev(&mut *self.backend)? {
            if let Some(mut next_node) = node.next(&mut *self.backend)? {
                next_node.set_prev(&mut *self.backend, &prev_node)?;
                prev_node.set_next(&mut *self.backend, &next_node)?;
            } else {
                prev_node.set_next_empty(&mut *self.backend)?;
            }
        } else {
            if let Some(mut next_node) = node.next(&mut *self.backend)? {
                next_node.set_prev_empty(&mut *self.backend)?;
            }
        }
        self.finalize_removal(&node)?;
        Ok(())
    }

    fn finalize_insert(&mut self, new_node: &Node<T>) -> Result<(), Error> {
        if new_node.is_first() {
            self.header.set_first_node_ptr(new_node.start());
        }
        if new_node.is_last() {
            self.header.set_last_node_ptr(new_node.start());
        }
        self.header.inc_counter();
        self.header.save(&mut *self.backend)?;
        self.update_allocator(&new_node)?;
        self.backend.persist()?;
        Ok(())
    }

    fn finalize_removal(&mut self, old_node: &Node<T>) -> Result<(), Error> {
        if old_node.is_first() {
            let next_ptr = old_node
                .next(&*self.backend)?
                .unwrap_or(Node::new(0))
                .start();
            self.header.set_first_node_ptr(next_ptr);
        }
        if old_node.is_last() {
            let prev_ptr = old_node
                .prev(&*self.backend)?
                .unwrap_or(Node::new(0))
                .start();
            self.header.set_last_node_ptr(prev_ptr);
        }
        self.header.dec_counter();
        let unused_bytes = Node::<T>::size() + old_node.data_size();
        self.header.inc_unused_bytes(unused_bytes);
        self.header.save(&mut *self.backend)?;
        self.backend.persist()?;
        Ok(())
    }

    fn update_allocator(&mut self, node: &Node<T>) -> Result<(), Error> {
        let position = node.start() + Node::<T>::size() + node.data_size();
        if position > self.header.get_allocator() {
            self.header.set_allocator(position);
            self.header.save(&mut *self.backend)?;
        }
        Ok(())
    }

    pub fn get_node_data(&self, node: &Node<T>) -> Result<T, Error> {
        node.data_fetch(&*self.backend)
    }

    pub fn iter(&self) -> Result<LinkedListIterator<T>, Error> {
        Ok(LinkedListIterator {
            current_node_ptr: self.first_node()?.map(|n| n.start()),
            backend: &self.backend,
            data_type: PhantomData,
        })
    }
}

pub struct LinkedListIterator<'a, T> {
    current_node_ptr: Option<usize>,
    backend: &'a Box<dyn Backend>,
    data_type: PhantomData<T>,
}

impl<'a, T> Iterator for LinkedListIterator<'a, T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
{
    type Item = Node<T>;
    fn next(&mut self) -> Option<Self::Item> {
        // if let Some(current_node_ptr) = self.current_node_ptr {
        //     if let Ok(current_node) = Node::load(&**self.backend, current_node_ptr) {
        //         if let
        //         let next_node = Node::load(&**self.backend, current_node_ptr).unwrap_or(None);
        //     }
        //     let next_node = Node::load(&**self.backend, current_node_ptr).unwrap_or(None);
        //     self.current_node_ptr = next_node
        //     let current_node = std::mem::replace(&mut self.current_node, next_node);
        //     current_node
        // } else {
        //     None
        // }
        if let Ok(node) = self.try_next() {
            node
        } else {
            None
        }
    }
}

impl<'a, T> LinkedListIterator<'a, T>
where
    T: serde::Serialize,
    for<'de> T: serde::Deserialize<'de>,
{
    fn try_next(&mut self) -> Result<Option<Node<T>>, Error> {
        if let Some(current_node_ptr) = self.current_node_ptr {
            let current_node = Node::load(&**self.backend, current_node_ptr)?;
            let next_node = current_node.next(&**self.backend)?;
            self.current_node_ptr = next_node.map(|n| n.start());
            Ok(Some(current_node))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_methods_left() {
        let mut list = LinkedList::<String>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        let mut node1 = list
            .insert_start(&"hello".to_string())
            .expect("couldn't insert start");
        assert_eq!(list.count(), 1);
        assert!(node1.is_first());
        assert!(node1.is_last());

        let node2 = list
            .insert_before(&mut node1, &"world".to_string())
            .expect("couldn't insert before");
        assert_eq!(list.count(), 2);
        assert!(!node1.is_first());
        assert!(node2.is_first());
        assert!(node1.is_last());
        assert!(!node2.is_last());
        assert_eq!(list.used_bytes(), 130);

        list.compact().expect("could not compact");
    }

    #[test]
    fn insert_methods_right() {
        let mut list = LinkedList::<String>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        let mut node1 = list
            .insert_end(&"hello".to_string())
            .expect("couldn't insert start");
        assert_eq!(list.count(), 1);
        assert!(node1.is_first());
        assert!(node1.is_last());

        let node2 = list
            .insert_after(&mut node1, &"world".to_string())
            .expect("couldn't insert before");
        assert_eq!(list.count(), 2);
        assert!(node1.is_first());
        assert!(!node2.is_first());
        assert!(!node1.is_last());
        assert!(node2.is_last());
    }

    #[test]
    fn remove() {
        let mut list = LinkedList::<String>::new("works.list").expect("can not create");
        assert_eq!(list.count(), 0);

        let mut node1 = list
            .insert_start(&"hello".to_string())
            .expect("couldn't insert start");
        let mut node2 = list
            .insert_after(&mut node1, &"world".to_string())
            .expect("couldn't insert before");
        assert_eq!(list.count(), 2);
        assert!(node1.is_first());
        assert!(node2.is_last());

        list.remove(node1).expect("couldn't remove");
        node2
            .init(&mut *list.backend)
            .expect("could not refresh node");
        assert_eq!(list.count(), 1);

        assert!(node2.is_first());
        assert!(node2.is_last());

        list.remove(node2).expect("couldn't remove");
        assert_eq!(list.count(), 0);
    }
}
