use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use wired::Queue;

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
    // create new database containing "Message" instances
    let file = tempfile::tempfile().expect("could not create tempfile");
    let mut db = Queue::<Message>::new(file.try_clone().unwrap()).unwrap();

    // enqueue some data
    db.enqueue(Message::new("msg 1")).unwrap();
    db.enqueue(Message::new("msg 2")).unwrap();
    db.enqueue(Message::new("msg 3")).unwrap();
    db.enqueue(Message::new("msg 4")).unwrap();

    // check for usage stats
    assert_eq!(db.len(), 4);

    // dequeue some data and check for IFIO ordering
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 1".to_string());
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 2".to_string());

    // check for usage stats
    assert_eq!(db.len(), 2);

    // works after reopen
    let mut db = Queue::<Message>::new(file).unwrap();
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 3".to_string());
    assert_eq!(db.dequeue().unwrap().unwrap().name, "msg 4".to_string());
    assert_eq!(db.len(), 0);
    assert!(db.is_empty());
}
