use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use wired;

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
    let mut db = wired::Queue::<Message>::new(path).unwrap();

    // enqueue some data
    db.enqueue(&Message::new("msg 1")).unwrap();
    db.enqueue(&Message::new("msg 2")).unwrap();
    db.enqueue(&Message::new("msg 3")).unwrap();

    // check for usage stats
    assert_eq!(db.len(), 3);
    assert_eq!(db.wasted_file_space(), 0.0);

    // dequeue some data and check for IFIO ordering
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 1".to_string());
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 2".to_string());

    // check for usage stats
    assert_eq!(db.len(), 1);
    assert_ne!(db.wasted_file_space(), 0.0);

    // compact the database ("defragmentation")
    db.compact().unwrap();
    assert_eq!(db.len(), 1);
    assert_eq!(db.wasted_file_space(), 0.0);

    // works after compaction
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 3".to_string());
}
