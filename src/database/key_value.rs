use crate::block_storage::BlockStorage;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;

pub struct KeyValue<K, V> {
    store: BlockStorage,
    header: Header,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl<K, V> KeyValue<K, V>
where
    K: Serialize + Eq,
    for<'de> K: Deserialize<'de>,
    V: Serialize,
    for<'de> V: Deserialize<'de>,
{
    pub fn new(file: File) -> Result<Self, Box<dyn Error>> {
        let mut store = BlockStorage::new(file)?;
        let header = Self::read_header(&mut store)?;
        let mut kv = Self {
            store,
            header,
            key_type: PhantomData,
            value_type: PhantomData,
        };
        kv.save_header()?;
        Ok(kv)
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

    pub fn len(&self) -> usize {
        self.header.key_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&mut self, key: K) -> Result<Option<V>, Box<dyn Error>> {
        let mut hit: Option<KeyEntry<K>> = None;
        for index in self.header.key_indices.iter() {
            let key_bytes = self.store.read(*index)?;
            let key_entry: KeyEntry<K> = bincode::deserialize_from(key_bytes.as_slice())?;
            if key_entry.body == key {
                hit = Some(key_entry);
            }
        }
        if let Some(key_entry) = hit {
            let value_bytes = self.store.read(key_entry.value_index)?;
            let value = bincode::deserialize_from(value_bytes.as_slice())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: K, value: V) -> Result<(), Box<dyn Error>> {
        // todo: check existing key

        // insert value
        let value_bytes: Vec<u8> = bincode::serialize(&value)?;
        let value_index = self.store.create(value_bytes.as_slice())?;

        // insert key
        let key = KeyEntry {
            body: key,
            value_index,
        };
        let key_bytes = bincode::serialize(&key)?;
        let key_index = self.store.create(key_bytes.as_slice())?;

        // update header
        self.header.key_indices.push(key_index);
        self.save_header()?;
        Ok(())
    }

    pub fn remove(&mut self, key: K) -> Result<(), Box<dyn Error>> {
        let mut hit: Option<KeyEntry<K>> = None;
        let mut hit_index: Option<usize> = None;
        for index in self.header.key_indices.iter() {
            let key_bytes = self.store.read(*index)?;
            let key_entry: KeyEntry<K> = bincode::deserialize_from(key_bytes.as_slice())?;
            if key_entry.body == key {
                hit = Some(key_entry);
                hit_index = Some(*index);
            }
        }
        if let Some(key_entry) = hit {
            self.store.delete(key_entry.value_index)?;
            self.store.delete(hit_index.unwrap())?;
            let index_position = self
                .header
                .key_indices
                .iter()
                .position(|&x| x == hit_index.unwrap())
                .unwrap();
            self.header.key_indices.remove(index_position);
            self.save_header()?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Header {
    key_indices: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct KeyEntry<K> {
    body: K,
    value_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        // setup db
        let file = tempfile::tempfile().expect("could not create tempfile");
        let mut kv = KeyValue::<i32, i32>::new(file).expect("could not create");
        assert_eq!(kv.len(), 0);

        // insert data
        kv.set(17, 42).expect("can not set");
        assert_eq!(kv.len(), 1);

        // read data
        let v = kv.get(17).expect("can not get");
        assert_eq!(v, Some(42));

        // remove data
        kv.remove(17).expect("could not remove");
        assert_eq!(kv.len(), 0);
        let v = kv.get(17).expect("can not get");
        assert_eq!(v, None);
    }
}
