/// Similar to is_empty, but allows for zeroed entries inside an iterator
///
/// [].is_zeroed == true
///
/// \[0, 0].is_zeroed == true
///
/// \[0, 1].is_zeroed == false
pub trait Zeroed {
    /// Returns true if all elements within the collection are zeroed.
    fn is_zeroed(&self) -> bool;

    /// Removes all zeroed elements from the collection.
    fn remove_zeroed(&mut self);
}

/// This trait defines how to get a vector of keys from a collection.
pub trait KeyVec<K> {
    fn key_vec(&self) -> Vec<K>;
}
