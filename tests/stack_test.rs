use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use wired::{Database, Stack};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    name: String,
    timestamp: u128,
}

impl Message {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_millis();
        Self { name, timestamp }
    }
}

#[test]
fn works() {
    // create temporary folder
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("message.queue");
    let path = file_path.to_str().unwrap();

    // create new database containing "Message" instances
    let mut db = Stack::<Message>::new(path).unwrap();

    // push some data
    db.push(&Message::new("msg 1")).unwrap();
    db.push(&Message::new("msg 2")).unwrap();
    db.push(&Message::new("msg 3")).unwrap();
    db.push(&Message::new("msg 4")).unwrap();

    // check for usage stats
    assert_eq!(db.len(), 4);
    assert_eq!(db.wasted_file_space(), 0.0);

    // pop some data and check for IFIO ordering
    assert_eq!(db.pop().unwrap().unwrap().name, "msg 4".to_string());
    assert_eq!(db.pop().unwrap().unwrap().name, "msg 3".to_string());

    // check for usage stats
    assert_eq!(db.len(), 2);
    assert_ne!(db.wasted_file_space(), 0.0);

    // compact the database ("defragmentation")
    db.compact().unwrap();
    assert_eq!(db.len(), 2);
    assert_eq!(db.wasted_file_space(), 0.0);

    // works after compaction
    assert_eq!(db.pop().unwrap().unwrap().name, "msg 2".to_string());

    // works after reopen
    let mut db = Stack::<Message>::new(path).unwrap();
    assert_eq!(db.pop().unwrap().unwrap().name, "msg 1".to_string());
    assert_eq!(db.len(), 0);
    assert!(db.is_empty());
}
