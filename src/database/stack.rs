use crate::model::LinkedList;
use serde::{Deserialize, Serialize};
use std::error::Error;

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
    list: LinkedList<T>,
}

impl<T> Stack<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let list = LinkedList::new(path)?;
        Ok(Self { list })
    }

    pub fn len(&self) -> usize {
        self.list.count()
    }

    pub fn wasted_file_space(&self) -> f64 {
        self.list.wasted_file_space()
    }

    pub fn push(&mut self, data: &T) -> Result<(), Box<dyn Error>> {
        self.list.insert_end(data)?;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Option<T>, Box<dyn Error>> {
        if self.len() == 0 {
            Ok(None)
        } else {
            let node = self.list.last_node()?.unwrap();
            let data = self.list.get_node_data(&node)?;
            self.list.remove(node)?;
            Ok(Some(data))
        }
    }

    pub fn compact(&mut self) -> Result<(), Box<dyn Error>> {
        self.list.compact()?;
        Ok(())
    }
}

impl<T> Iterator for Stack<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop().unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut stack = Stack::<i32>::new("works.stack").expect("could not create");
        assert_eq!(stack.len(), 0);

        stack.push(&1).expect("could not push");
        stack.push(&2).expect("could not push");
        assert_eq!(stack.len(), 2);

        let data = stack.pop().expect("could not pop");
        assert_eq!(data, Some(2));
        assert_eq!(stack.len(), 1);

        let data = stack.pop().expect("could not pop");
        assert_eq!(data, Some(1));
        assert_eq!(stack.len(), 0);

        let data = stack.pop().expect("could not pop");
        assert_eq!(data, None);
    }

    #[test]
    fn iteration() {
        let mut stack = Stack::<i32>::new("works.stack").expect("could not create");
        stack.push(&1).expect("could not push");
        stack.push(&2).expect("could not push");
        assert_eq!(stack.len(), 2);

        let vec: Vec<i32> = stack.collect();
        assert_eq!(vec, vec![2, 1]);
    }
}
