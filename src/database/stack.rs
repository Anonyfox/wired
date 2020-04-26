use crate::block_storage::BlockStorage;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;

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
/// # let file = tempfile::tempfile()?;
/// let mut stack = wired::Stack::<Example>::new(file)?;
///
/// // insert an item
/// let item = Example { num: 42 };
/// stack.push(item)?;
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
    store: BlockStorage,
    header: Header,
    data_type: PhantomData<T>,
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
    /// # let file = tempfile::tempfile()?;
    /// let stack = wired::Stack::<String>::new(file)?;
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
    /// # let file = tempfile::tempfile()?;
    /// let stack = wired::Stack::<Example>::new(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let mut store = BlockStorage::new(file)?;
        let header = Self::read_header(&mut store)?;
        let data_type = PhantomData;
        let mut stack = Self {
            store,
            header,
            data_type,
        };
        stack.save_header()?;
        Ok(stack)
    }

    pub fn len(&self) -> usize {
        self.header.elements_count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn read_header(store: &mut BlockStorage) -> Result<Header, Box<dyn Error>> {
        let bytes = store.read(0)?;
        if store.is_empty() {
            let header = Header::default();
            let bytes: Vec<u8> = bincode::serialize(&header)?;
            store.create(bytes.as_slice())?;
            Ok(header)
        } else {
            let header = bincode::deserialize_from(bytes.as_slice())?;
            Ok(header)
        }
    }

    fn save_header(&mut self) -> Result<(), Box<dyn Error>> {
        let bytes: Vec<u8> = bincode::serialize(&self.header)?;
        self.store.update(0, bytes.as_slice())
    }

    /// insert a new item at the end of the stack and persist to disk
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let mut stack = wired::Stack::<String>::new(file)?;
    /// let item = String::from("some item");
    /// stack.push(item)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&mut self, data: T) -> Result<(), Box<dyn Error>> {
        let mut element = Element {
            body: data,
            prev: 0,
        };
        if self.header.last_element != 0 {
            element.prev = self.header.last_element;
        }
        let bytes: Vec<u8> = bincode::serialize(&element)?;
        let index = self.store.create(bytes.as_slice())?;
        self.header.last_element = index;
        self.header.elements_count += 1;
        self.save_header()?;
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
    /// # let file = tempfile::tempfile()?;
    /// let mut stack = wired::Stack::<String>::new(file)?;
    /// stack.push(String::from("some item"))?;
    /// let item = stack.pop()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pop(&mut self) -> Result<Option<T>, Box<dyn Error>> {
        if self.header.elements_count == 0 {
            return Ok(None);
        }
        let index = self.header.last_element;
        let bytes = self.store.read(index)?;
        let element: Element<T> = bincode::deserialize_from(bytes.as_slice())?;
        self.store.delete(index)?;
        self.header.last_element = element.prev;
        self.header.elements_count -= 1;
        self.save_header()?;
        Ok(Some(element.body))
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

#[derive(Serialize, Deserialize, Debug, Default)]
struct Header {
    last_element: usize,
    elements_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Element<T> {
    prev: usize,
    body: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut stack = Stack::<i32>::new(file).expect("could not create");
        assert_eq!(stack.len(), 0);

        stack.push(1).expect("could not push");
        stack.push(2).expect("could not push");
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
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut stack = Stack::<i32>::new(file).expect("could not create");
        stack.push(1).expect("could not push");
        stack.push(2).expect("could not push");
        assert_eq!(stack.len(), 2);

        let vec: Vec<i32> = stack.collect();
        assert_eq!(vec, vec![2, 1]);
    }
}
