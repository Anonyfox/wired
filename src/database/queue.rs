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
pub struct Queue<T> {
    list: LinkedList<T>,
}

impl<T> Queue<T>
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

    pub fn enqueue(&mut self, data: &T) -> Result<(), Box<dyn Error>> {
        self.list.insert_start(data)?;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<Option<T>, Box<dyn Error>> {
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
}
