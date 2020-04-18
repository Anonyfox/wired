use crate::model::LinkedList;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// a First-In-First-Out data structure
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
/// let mut queue = wired::Queue::<Example>::new("/path/to/file.queue")?;
///
/// // insert an item
/// let item = Example { num: 42 };
/// queue.enqueue(&item)?;
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
    list: LinkedList<T>,
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
    /// let queue = wired::Queue::<String>::new("/path/to/file.queue")?;
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
    /// let queue = wired::Queue::<Example>::new("/path/to/file.queue")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let list = LinkedList::new(path)?;
        Ok(Self { list })
    }

    /// get the amount of items currently in the queue
    pub fn len(&self) -> usize {
        self.list.count()
    }

    /// check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// get the ratio of bytes marked for deletion
    ///
    /// will return a value between `0.0` (optimal) and `1.0` (highly fragmented)
    pub fn wasted_file_space(&self) -> f64 {
        self.list.wasted_file_space()
    }

    /// insert a new item in front of the queue and persist to disk
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut queue = wired::Queue::<String>::new("/path/to/file.queue")?;
    /// let item = String::from("some item");
    /// queue.enqueue(&item)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enqueue(&mut self, data: &T) -> Result<(), Box<dyn Error>> {
        self.list.insert_start(data)?;
        Ok(())
    }

    /// remove the item at the back of the queue, persist to disk and return the item
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut queue = wired::Queue::<String>::new("/path/to/file.queue")?;
    /// queue.enqueue(&String::from("some item"))?;
    /// let item = queue.dequeue()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn dequeue(&mut self) -> Result<Option<T>, Box<dyn Error>> {
        if self.len() == 0 {
            Ok(None)
        } else {
            let node = self.list.last_node()?.unwrap();
            let data: T = self.list.get_node_data(&node)?;
            self.list.remove(node)?;
            Ok(Some(data))
        }
    }

    /// defragment the database into a pristine state
    ///
    /// This will rebuild the database file under the hood and swap out/delete
    /// the current one. This operation is quite expensive but frees up all
    /// unused disk space, so decide for yourself when you want to do this.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut queue = wired::Queue::<String>::new("/path/to/file.queue")?;
    /// queue.compact()?;
    /// # Ok(())
    /// # }
    pub fn compact(&mut self) -> Result<(), Box<dyn Error>> {
        self.list.compact()?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut queue = Queue::<i32>::new("works.queue").expect("could not create");
        assert_eq!(queue.len(), 0);

        queue.enqueue(&1).expect("could not enqueue");
        queue.enqueue(&2).expect("could not enqueue");
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
        let mut queue = Queue::<i32>::new("works.queue").expect("could not create");
        queue.enqueue(&1).expect("could not enqueue");
        queue.enqueue(&2).expect("could not enqueue");
        assert_eq!(queue.len(), 2);

        let vec: Vec<i32> = queue.collect();
        assert_eq!(vec, vec![1, 2]);
    }
}
