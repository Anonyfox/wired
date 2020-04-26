use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use wired::KeyValue;

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
    let mut db = KeyValue::<String, Message>::new(file.try_clone().unwrap()).unwrap();
    assert!(db.is_empty());

    // insert some data
    db.set(String::from("m1"), Message::new("msg 1")).unwrap();
    db.set(String::from("m2"), Message::new("msg 2")).unwrap();
    db.set(String::from("m3"), Message::new("msg 3")).unwrap();
    db.set(String::from("m4"), Message::new("msg 4")).unwrap();

    // check for usage stats
    assert_eq!(db.len(), 4);

    // read some data
    let msg = db.get(&String::from("m2")).unwrap().unwrap();
    assert_eq!(msg.name, "msg 2");

    // delete some data
    db.remove(&String::from("m3")).unwrap();
    assert_eq!(db.len(), 3);

    // works after reopen
    let mut db = KeyValue::<String, Message>::new(file).unwrap();
    let msg = db.get(&String::from("m4")).unwrap().unwrap();
    assert_eq!(msg.name, "msg 4");
    assert_eq!(db.len(), 3);
}
