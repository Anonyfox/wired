use super::Database;
use crate::model::LinkedList;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// a Last-In-First-Out Database
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
///
/// # Examples
///
/// ```rust,no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // any datatype that can be serialized by serde works
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Example {
///     num: i32,
/// }
///
/// // create a new db
/// let mut stack = wired::Stack::<Example>::new("/path/to/file.stack")?;
///
/// // insert an item
/// let item = Example { num: 42 };
/// stack.push(&item)?;
///
/// // retrieve an item
/// let item = stack.pop()?;
/// dbg!(item); // Some(Example { num: 42 })
///
/// // try retrieve an item from the now-empty queue
/// let item = stack.pop()?;
/// dbg!(item); // None
/// # Ok(())
/// # }
/// ```
pub struct Stack<T> {
    list: LinkedList<T>,
}

impl<T> Stack<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    /// Create a new database or open an existing one for the given location.
    /// The database is generic over a serializable datatype.
    ///
    /// # Examples
    ///
    /// Stack for strings:
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let stack = wired::Stack::<String>::new("/path/to/file.stack")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Stack for structs:
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///     count: i32,
    /// }
    ///
    /// let stack = wired::Stack::<Example>::new("/path/to/file.stack")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let list = LinkedList::new(path)?;
        Ok(Self { list })
    }

    /// insert a new item at the end of the stack and persist to disk
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stack = wired::Stack::<String>::new("/path/to/file.stack")?;
    /// let item = String::from("some item");
    /// stack.push(&item)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self, data: &T) -> Result<(), Box<dyn Error>> {
        self.list.insert_end(data)?;
        Ok(())
    }

    /// remove the item at the and of the stack, persist to disk and return the item
    ///
    /// Note: if you discard the popped item it will be lost permanently!
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stack = wired::Stack::<String>::new("/path/to/file.stack")?;
    /// stack.push(&String::from("some item"))?;
    /// let item = stack.pop()?;
    /// # Ok(())
    /// # }
    /// ```
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
}

impl<T> Database for Stack<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    fn compact(&mut self) -> Result<(), Box<dyn Error>> {
        self.list.compact()?;
        Ok(())
    }

    fn wasted_file_space(&self) -> f64 {
        self.list.wasted_file_space()
    }

    fn len(&self) -> usize {
        self.list.count()
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
