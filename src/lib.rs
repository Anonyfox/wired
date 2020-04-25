mod backend;
mod database;
mod model;
pub mod storage;

pub use database::queue::Queue;
pub use database::stack::Stack;
pub use database::Database;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
