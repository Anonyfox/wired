use super::LinkedList;
use serde::{Deserialize, Serialize};
use std::error::Error;

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
