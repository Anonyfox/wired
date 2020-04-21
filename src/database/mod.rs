pub mod queue;
pub mod stack;

/// General trait that all `wired` databases must implement.
///
/// Allows for a centralized handling of database operations that are the same
/// no matter which kind of specific database is used, like disk space
/// management.
pub trait Database {
    /// defragment the database into a pristine state
    ///
    /// This will rebuild the database file under the hood and swap out/delete
    /// the current one. This operation is quite expensive but frees up all
    /// unused disk space, so decide for yourself when you want to do this.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use wired::{Database, Queue};
    ///
    /// let mut queue = Queue::<String>::new("/path/to/file.queue")?;
    /// queue.compact()?;
    /// # Ok(())
    /// # }
    fn compact(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// get the ratio of bytes marked for deletion
    ///
    /// will return a value between `0.0` (optimal) and `1.0` (highly fragmented)
    fn wasted_file_space(&self) -> f64;

    /// get the amount of records/items currently in the database
    fn len(&self) -> usize;

    /// check if the database is empty (contains no records/items at all)
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
