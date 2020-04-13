mod element;
mod header;

use crate::backend::{Backend, DataBlock, Mmap, StaticBlock};
use element::Element;
use header::Header;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::PhantomData;

/// a Last-In-First-Out data structure
///
/// A `Stack` is backed by a memory-mapped file, so the full content does not
/// reside in RAM when not needed. It is type-safe over a generic type that
/// can be serialized through `serde`.
///
/// What exactly is a Stack?
///
/// > In computer science, a stack is an abstract data type that serves as a
/// > collection of elements, with two principal operations:
/// >
/// > * push, which adds an element to the collection, and
/// > * pop, which removes the most recently added element that was not yet removed.
/// >
/// > The order in which elements come off a stack gives rise to its alternative
/// > name, LIFO (last in, first out). Additionally, a peek operation may give
/// > access to the top without modifying the stack. The name "stack" for
/// > this type of structure comes from the analogy to a set of physical items
/// > stacked on top of each other, which makes it easy to take an item off the
/// > top of the stack, while getting to an item deeper in the stack may require
/// > taking off multiple other items first.
/// >
/// > -- <cite>[Wikipedia](https://en.wikipedia.org/wiki/Stack_(abstract_data_type))</cite>
pub struct Stack<T> {
    header: Header,
    backend: Box<dyn Backend>,
    data_type: PhantomData<T>,
}

impl<T> Stack<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    /// Create a new instance of a `Stack`
    ///
    /// Needs a path to the backing file, will create a new one if it doesn't
    /// exist yet.
    ///
    /// # Examples
    /// ```no-run
    /// # use wired::Stack;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let stack: Stack<i32> = Stack::new("/path/to/file.stack")?;
    /// # Ok(())
    /// # }
    ///
    /// ```
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
            data_type: PhantomData,
        })
    }

    pub fn count(&self) -> usize {
        self.header.element_count()
    }

    pub fn pop(&mut self) -> Result<Option<T>, Box<dyn Error>> {
        if self.header.element_count() == 0 {
            Ok(None)
        } else {
            let current = self.get_current()?;
            self.header.set_current_ptr(current.get_prev_ptr());
            self.header.dec_counter();
            self.header.save(&mut *self.backend)?;
            let data = current.data_fetch(&*self.backend)?;
            self.backend.persist()?;
            Ok(Some(data))
        }
    }

    pub fn push(&mut self, data: T) -> Result<(), Box<dyn Error>> {
        if self.header.element_count() == 0 {
            let position = Header::size();
            let mut element = Element::new(position);
            element.data_store(&mut *self.backend, &data)?;
            element.save(&mut *self.backend)?;
            self.header.set_current_ptr(element.get_ptr());
        } else {
            let current = self.get_current()?;
            let position = current.get_ptr() + Element::<T>::size();
            let mut new = Element::new(position);
            new.data_store(&mut *self.backend, &data)?;
            new.set_prev_ptr(current.get_ptr());
            new.save(&mut *self.backend)?;
            self.header.set_current_ptr(new.get_ptr());
        }
        self.header.inc_counter();
        self.backend.persist()?;
        Ok(())
    }

    fn get_current(&self) -> Result<Element<T>, Box<dyn Error>> {
        let position = self.header.get_current_ptr();
        Element::load(&*self.backend, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut stack = Stack::new("works.stack").expect("can not create");
        assert_eq!(stack.count(), 0);

        stack.push(17).expect("can not push");
        assert_eq!(stack.count(), 1);

        let element = stack.pop().expect("can not pop");
        assert_eq!(stack.count(), 0);
        assert_eq!(element, Some(17));
    }
}
