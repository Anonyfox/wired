mod backend;
mod database;
mod model;

pub use database::queue::Queue;
pub use database::stack::Stack;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
