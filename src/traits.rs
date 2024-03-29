use cosmwasm_std::Uint256;

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

impl Zeroed for Uint256 {
    fn is_zeroed(&self) -> bool {
        self.is_zero()
    }

    fn remove_zeroed(&mut self) {}
}

/// This trait defines how to get a vector of keys from a collection.
pub trait KeyVec<K> {
    fn key_vec(&self) -> Vec<K>;
}

impl<T, K> KeyVec<K> for Vec<T>
where
    K: Ord + PartialEq + Clone,
    T: KeyVec<K>,
{
    fn key_vec(&self) -> Vec<K> {
        let mut list = self.iter().flat_map(|x| x.key_vec()).collect::<Vec<_>>();
        list.sort_unstable();
        list.dedup();
        list
    }
}

pub fn extract_keys<'a, K: 'a + Ord + PartialEq + Clone>(vec: Vec<&'a dyn KeyVec<K>>) -> Vec<K> {
    let mut list = vec
        .into_iter()
        .flat_map(|x| x.key_vec())
        .collect::<Vec<_>>();
    list.sort_unstable();
    list.dedup();
    list
}
