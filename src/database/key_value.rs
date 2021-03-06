use crate::block_storage::BlockStorage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::marker::PhantomData;

/// Key Value Database
///
/// Definition:
///
/// > A key-value database, or key-value store, is a data storage paradigm
/// > designed for storing, retrieving, and managing associative arrays, and a
/// > data structure more commonly known today as a dictionary or hash table.
/// > Dictionaries contain a collection of objects, or records, which in turn
/// > have many different fields within them, each containing data. These
/// > records are stored and retrieved using a key that uniquely identifies the
/// > record, and is used to find the data within the database.
/// >
/// > -- <cite>[Wikipedia](https://en.wikipedia.org/wiki/Key-value_database)</cite>
///
/// Keys and Values can be arbitrary data types, as long as they can be
/// serialized to bincode via serde. using `#[derive(Serialize, Deserialize)]`
/// on your structs should suffice. Keys must implement the `Eq` and `Hash`
/// trait.
///
/// # Examples
///
/// ```rust,no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // any datatype that can be serialized by serde works
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Value {
///     num: i32,
/// }
///
/// // create a new db
/// # let file = tempfile::tempfile()?;
/// let mut kv = wired::KeyValue::<String, Value>::new(file)?;
///
/// // insert an item
/// let key = String::from("some identifier");
/// let value = Value { num: 42 };
/// kv.set(key, value);
///
/// // retrieve an item
/// let key = String::from("some identifier");
/// let value = kv.get(&key)?; // Some(Value { num: 42 })
///
/// // delete an item
/// let key = String::from("some identifier");
/// kv.remove(&key)?;
/// # Ok(())
/// # }
pub struct KeyValue<K, V> {
    store: BlockStorage,
    header: Header,
    lookup: HashMap<K, usize>,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl<K, V> KeyValue<K, V>
where
    K: Serialize + Hash + Eq,
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
            lookup: HashMap::new(),
            key_type: PhantomData,
            value_type: PhantomData,
        };
        kv.save_header()?;
        for index in kv.header.key_indices.iter() {
            let bytes = kv.store.read(*index)?;
            let entry: KeyEntry<K> = bincode::deserialize_from(bytes.as_slice())?;
            kv.lookup.insert(entry.body, entry.value_index);
        }
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

    pub fn keys(&self) -> Vec<&K> {
        let mut result: Vec<&K> = vec![];
        for key in self.lookup.keys() {
            result.push(key);
        }
        result
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, Box<dyn Error>> {
        if let Some(value_index) = self.lookup.get(&key) {
            let value_bytes = self.store.read(*value_index)?;
            let value = bincode::deserialize_from(value_bytes.as_slice())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: K, value: V) -> Result<(), Box<dyn Error>> {
        if self.lookup.contains_key(&key) {
            self.remove(&key)?;
        }

        // insert value
        let value_bytes: Vec<u8> = bincode::serialize(&value)?;
        let value_index = self.store.create(value_bytes.as_slice())?;

        // insert key
        let key_entry = KeyEntry {
            body: key,
            value_index,
        };
        let key_bytes = bincode::serialize(&key_entry)?;
        let key_index = self.store.create(key_bytes.as_slice())?;
        self.lookup.insert(key_entry.body, key_entry.value_index);

        // update header
        self.header.key_indices.push(key_index);
        self.save_header()?;
        Ok(())
    }

    pub fn remove(&mut self, key: &K) -> Result<(), Box<dyn Error>> {
        let mut hit: Option<KeyEntry<K>> = None;
        let mut hit_index: Option<usize> = None;
        for index in self.header.key_indices.iter() {
            let key_bytes = self.store.read(*index)?;
            let key_entry: KeyEntry<K> = bincode::deserialize_from(key_bytes.as_slice())?;
            if key_entry.body == *key {
                hit = Some(key_entry);
                hit_index = Some(*index);
            }
        }
        if let Some(key_entry) = hit {
            self.store.delete(key_entry.value_index)?;
            self.store.delete(hit_index.unwrap())?;
            self.lookup.remove(&key_entry.body);
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
        assert_eq!(kv.keys(), Vec::<&i32>::new());

        // insert data
        kv.set(17, 42).expect("can not set");
        assert_eq!(kv.len(), 1);
        assert_eq!(kv.keys(), vec![&17]);

        // read data
        let v = kv.get(&17).expect("can not get");
        assert_eq!(v, Some(42));

        // update data
        kv.set(17, 101).expect("can not set");
        assert_eq!(kv.len(), 1);
        assert_eq!(kv.keys(), vec![&17]);

        // read data again
        let v = kv.get(&17).expect("can not get");
        assert_eq!(v, Some(101));

        // remove data
        kv.remove(&17).expect("could not remove");
        assert_eq!(kv.len(), 0);
        assert_eq!(kv.keys(), Vec::<&i32>::new());
        let v = kv.get(&17).expect("can not get");
        assert_eq!(v, None);
    }
}
