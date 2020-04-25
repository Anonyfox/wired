use crate::block_storage::BlockStorage;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;

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

    // #[test]
    // fn iteration() {
    //     let mut stack = Stack::<i32>::new("works.stack").expect("could not create");
    //     stack.push(&1).expect("could not push");
    //     stack.push(&2).expect("could not push");
    //     assert_eq!(stack.len(), 2);

    //     let vec: Vec<i32> = stack.collect();
    //     assert_eq!(vec, vec![2, 1]);
    // }
}
