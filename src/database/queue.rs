use crate::block_storage::BlockStorage;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;

/// a First-In-First-Out Database
///
/// A `Queue` is backed by a memory-mapped file, so the full content does not
/// reside in RAM when not needed. It is type-safe over a generic type that
/// can be serialized through `serde`.
///
/// What exactly is a `Queue`?
///
/// > In computer science, a queue is a collection of entities that are
/// > maintained in a sequence and can be modified by the addition of entities
/// > at one end of the sequence and removal from the other end of the sequence.
/// > By convention, the end of the sequence at which elements are added is
/// > called the back, tail, or rear of the queue and the end at which
/// > elements are removed is called the head or front of the queue,
/// > analogously to the words used when people line up to wait for goods or services.
/// >
/// > - The operation of adding an element to the rear of the queue is known as
/// >   enqueue,
/// > - and the operation of removing an element from the front is known as
/// >   dequeue.
/// >
/// > -- <cite>[Wikipedia](https://en.wikipedia.org/wiki/Queue_(abstract_data_type))</cite>
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
/// let mut queue = wired::Queue::<Example>::new(file)?;
///
/// // insert an item
/// let item = Example { num: 42 };
/// queue.enqueue(item)?;
///
/// // retrieve an item
/// let item = queue.dequeue()?;
/// dbg!(item); // Some(Example { num: 42 })
///
/// // try retrieve an item from the now-empty queue
/// let item = queue.dequeue()?;
/// dbg!(item); // None
/// # Ok(())
/// # }
/// ```

pub struct Queue<T> {
    store: BlockStorage,
    header: Header,
    data_type: PhantomData<T>,
}

impl<T> Queue<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    /// Create a new database or open an existing one for the given location.
    /// The database is generic over a serializable datatype.
    ///
    /// # Examples
    ///
    /// Queue for strings:
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let queue = wired::Queue::<String>::new(file)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Queue for structs:
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
    /// let queue = wired::Queue::<Example>::new(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let mut store = BlockStorage::new(file)?;
        let header = Self::read_header(&mut store)?;
        let data_type = PhantomData;
        let mut queue = Self {
            store,
            header,
            data_type,
        };
        queue.save_header()?;
        Ok(queue)
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

    /// insert a new item in front of the queue and persist to disk
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let mut queue = wired::Queue::<String>::new(file)?;
    /// let item = String::from("some item");
    /// queue.enqueue(item)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enqueue(&mut self, data: T) -> Result<(), Box<dyn Error>> {
        let mut element = Element {
            body: data,
            next: 0,
            prev: 0,
        };
        if self.header.first_element != 0 {
            element.next = self.header.first_element;
        }
        let bytes: Vec<u8> = bincode::serialize(&element)?;
        let index = self.store.create(bytes.as_slice())?;

        if self.header.first_element != 0 {
            let first_index = self.header.first_element;
            let first_bytes = self.store.read(first_index)?;
            let mut first: Element<T> = bincode::deserialize_from(first_bytes.as_slice())?;
            first.prev = index;
            let first_bytes: Vec<u8> = bincode::serialize(&first)?;
            self.store.update(first_index, first_bytes.as_slice())?;
        }
        if self.header.last_element == 0 {
            self.header.last_element = index;
        }
        self.header.first_element = index;
        self.header.elements_count += 1;
        self.save_header()?;
        Ok(())
    }

    /// remove the item at the back of the queue, persist to disk and return the item
    ///
    /// Note: if you discard the dequeued item it will be lost permanently!
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let file = tempfile::tempfile()?;
    /// let mut queue = wired::Queue::<String>::new(file)?;
    /// queue.enqueue(String::from("some item"))?;
    /// let item = queue.dequeue()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn dequeue(&mut self) -> Result<Option<T>, Box<dyn Error>> {
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

impl<T> Iterator for Queue<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.dequeue().unwrap_or(None)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Header {
    first_element: usize,
    last_element: usize,
    elements_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Element<T> {
    next: usize,
    prev: usize,
    body: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut queue = Queue::<i32>::new(file).expect("could not create");
        assert_eq!(queue.len(), 0);

        queue.enqueue(1).expect("could not enqueue");
        queue.enqueue(2).expect("could not enqueue");
        assert_eq!(queue.len(), 2);

        let data = queue.dequeue().expect("could not dequeue");
        assert_eq!(data, Some(1));
        assert_eq!(queue.len(), 1);

        let data = queue.dequeue().expect("could not dequeue");
        assert_eq!(data, Some(2));
        assert_eq!(queue.len(), 0);

        let data = queue.dequeue().expect("could not dequeue");
        assert_eq!(data, None);
    }

    #[test]
    fn iteration() {
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut queue = Queue::<i32>::new(file).expect("could not create");
        queue.enqueue(1).expect("could not enqueue");
        queue.enqueue(2).expect("could not enqueue");
        assert_eq!(queue.len(), 2);

        let vec: Vec<i32> = queue.collect();
        assert_eq!(vec, vec![1, 2]);
    }
}
