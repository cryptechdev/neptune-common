use std::{
    fmt::Debug,
    iter::FromIterator,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use cosmwasm_std::Decimal256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;

use crate::{
    asset::AssetInfo,
    error::{CommonError, CommonResult},
    traits::{KeyVec, Zeroed},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Map<K, V>(pub Vec<(K, V)>);

impl<K, V> Map<K, V>
where
    K: PartialEq + Clone + Debug,
{
    pub const fn new() -> Self { Self(Vec::new()) }

    pub fn insert(&mut self, tuple: (K, V)) -> Option<V> {
        match self.get_mut(&tuple.0) {
            Some(value) => Some(std::mem::replace(value, tuple.1)),
            None => {
                self.0.push(tuple);
                None
            }
        }
    }

    pub fn contains_key(&self, key: &K) -> bool { self.get(key).is_some() }

    pub fn must_get(&self, key: &K) -> CommonResult<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&self.0[index].1),
            None => Err(CommonError::KeyNotFound(format!("{:?}", key.clone()))),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&self.0[index].1),
            None => None,
        }
    }

    pub fn must_get_mut(&mut self, key: &K) -> CommonResult<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&mut self.0[index].1),
            None => Err(CommonError::KeyNotFound(format!("{:?}", key.clone()))),
        }
    }

    pub fn must_get_muts<const LEN: usize>(&mut self, keys: [&K; LEN]) -> CommonResult<[&mut V; LEN]>
    where
        V: Debug,
    {
        find_map_many(self, keys, |item, key| &item.0 == key, |item| &mut item.1)
            .ok_or_else(|| CommonError::KeyNotFound(String::new()))
    }

    pub fn get_muts_or_default<const LEN: usize>(&mut self, keys: [&K; LEN]) -> [&mut V; LEN]
    where
        V: Debug + Default,
    {
        // add a default if it doesn't exist
        for key in keys {
            if !self.contains_key(key) {
                self.insert((key.to_owned(), V::default()));
            }
        }
        // unwrap is safe because we just added all the keys
        self.must_get_muts(keys).unwrap()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&mut self.0[index].1),
            None => None,
        }
    }

    pub fn map<F: Fn(&V) -> O, O>(&mut self, f: F) -> Map<K, O> {
        let mut output = vec![];
        for (key, val) in &self.0 {
            let function_output = f(val);
            output.push((key.clone(), function_output));
        }
        output.into()
    }

    pub fn map_result_val<F: Fn(&V) -> Result<O, E>, O, E>(&mut self, f: F) -> Result<Map<K, O>, E> {
        let mut output = vec![];
        for (key, val) in &self.0 {
            let function_output = f(val)?;
            output.push((key.clone(), function_output));
        }
        Ok(output.into())
    }

    pub fn get_mut_or_default<'a>(&'a mut self, key: &K) -> &'a mut V
    where
        V: Default,
    {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert((key.clone(), V::default()));
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    /// multiplies every value in self with the corresponding value in rhs. Returns an error if rhs
    /// is missing a key. Rhs must contain every key in self, but self needs not contain every key
    /// in rhs.
    pub fn mul_all<U>(self, rhs: &Map<K, U>) -> CommonResult<Map<K, <V as Mul<U>>::Output>>
    where
        V: Mul<U>,
        U: Clone,
    {
        let mut output = Vec::with_capacity(self.len());
        for (key, lhs_val) in self {
            let rhs_val = rhs.must_get(&key)?.clone();
            output.push((key, lhs_val * rhs_val))
        }
        Ok(output.into())
    }

    pub fn sum(&self) -> V
    where
        V: Default + Add<Output = V> + Clone,
    {
        let mut sum = V::default();
        for (_, val) in &self.0 {
            sum = sum + val.clone();
        }
        sum
    }

    pub fn sort_by_val(&mut self)
    where
        V: Default + Ord + Clone,
    {
        self.0.sort_by(|a, b| a.1.cmp(&b.1))
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self { Self(Vec::new()) }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self { Vec::<(K, V)>::from_iter(iter).into() }
}

impl<K, V> IntoIterator for Map<K, V> {
    type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type IntoIter = <&'a Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = &'a (K, V);

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type IntoIter = <&'a mut Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = &'a mut (K, V);

    fn into_iter(self) -> Self::IntoIter { self.0.iter_mut() }
}

impl<K, V> Mul<Decimal256> for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Mul<Decimal256, Output = V> + Clone,
{
    type Output = Self;

    /// multiplies each value with a Decimal256
    fn mul(mut self, rhs: Decimal256) -> Self::Output {
        for (_, val) in &mut self {
            *val = val.clone() * rhs
        }
        self
    }
}

impl<K, V> Div<Decimal256> for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Div<Decimal256, Output = V> + Clone,
{
    type Output = Self;

    /// Divides each value with a Decimal256
    fn div(mut self, rhs: Decimal256) -> Self::Output {
        for (_, val) in &mut self {
            *val = val.clone() / rhs
        }
        self
    }
}

impl<K, V> Add for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Add<Output = V> + Clone + Default,
{
    type Output = Self;

    /// Adds the corresponding values from two maps together.
    /// If a key exists in one map but not the other, the default is used.
    fn add(mut self, rhs: Self) -> Self::Output {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() + rhs_key_val.1;
        }
        self
    }
}

impl<K, V> AddAssign for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Add<Output = V> + Clone + Default,
{
    /// Adds the corresponding values from two maps together.
    /// If a key exists in one map but not the other, the default is used.
    fn add_assign(&mut self, rhs: Self) {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() + rhs_key_val.1;
        }
    }
}

impl<K, V> Sub for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Sub<Output = V> + Clone + Default,
{
    type Output = Self;

    /// Subs the corresponding values from two maps together.
    /// If a key exists in one map but not the other, the default is used.
    fn sub(mut self, rhs: Self) -> Self::Output {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() - rhs_key_val.1;
        }
        self
    }
}

impl<K, V> SubAssign for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Sub<Output = V> + Clone + Default,
{
    /// Subs the corresponding values from two maps together.
    /// If a key exists in one map but not the other, the default is used.
    fn sub_assign(&mut self, rhs: Self) {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() - rhs_key_val.1;
        }
    }
}

impl<K, V> From<Vec<(K, V)>> for Map<K, V> {
    fn from(object: Vec<(K, V)>) -> Self { Self(object) }
}

impl<K, V> From<(K, V)> for Map<K, V> {
    fn from(object: (K, V)) -> Self { Self(vec![object]) }
}

impl<K, V> Zeroed for Map<K, V>
where
    V: Zeroed,
{
    fn is_zeroed(&self) -> bool { self.iter().all(|x| x.1.is_zeroed()) }

    fn remove_zeroed(&mut self) {
        self.iter_mut().for_each(|x| x.1.remove_zeroed());
        self.retain(|x| !x.1.is_zeroed())
    }
}

impl<T, K> KeyVec<K> for Vec<T>
where
    K: PartialEq + Clone,
    T: KeyVec<K>,
{
    fn key_vec(&self) -> Vec<K> {
        let mut key_vec = vec![];
        for val in self {
            for key in val.key_vec() {
                if !key_vec.contains(&key) {
                    key_vec.push(key.clone());
                }
            }
        }
        key_vec
    }
}

impl<K, V> KeyVec<K> for Map<K, V>
where
    K: PartialEq + Clone,
{
    fn key_vec(&self) -> Vec<K> {
        let mut key_vec = vec![];
        for (key, _) in &self.0 {
            if !key_vec.contains(key) {
                key_vec.push(key.clone());
            }
        }
        key_vec
    }
}

impl KeyVec<Self> for AssetInfo {
    fn key_vec(&self) -> Vec<Self> { vec![self.clone()] }
}

pub fn extract_keys<'a, K: 'a + PartialEq + Clone>(vec: Vec<&'a dyn KeyVec<K>>) -> Vec<K> {
    let mut asset_vec = vec![];
    for object in vec {
        for asset in object.key_vec() {
            if !asset_vec.contains(&asset) {
                asset_vec.push(asset.clone());
            }
        }
    }
    asset_vec
}

pub fn find_many<'a, I, T, F, K, const LEN: usize>(
    collection: I, keys: [&K; LEN], mut predicate: F,
) -> Option<[&'a mut T; LEN]>
where
    T: Debug,
    I: IntoIterator<Item = &'a mut T>,
    F: FnMut(&T, &K) -> bool,
{
    let mut remaining = LEN;
    let mut output = Vec::with_capacity(LEN);
    (0..LEN).for_each(|_| output.push(None));

    'collection: for elem in collection {
        for (key, out) in std::iter::zip(&keys, &mut output) {
            if out.is_none() && predicate(elem, key) {
                *out = Some(elem);
                remaining -= 1;
                if remaining == 0 {
                    break 'collection;
                }
                break;
            }
        }
    }

    let Some(vec) = output.into_iter().collect::<Option<Vec<&mut T>>>() else {
        return None
    };
    Some(vec.try_into().unwrap())
}

/// finds multiple items in a collection and maps the elements to &muts.
///
/// ```
/// # use neptune_common::map::find_map_many;
/// # fn test_find_many() {
/// let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
/// let [left, right] =
///     find_map_many(&mut v, [&2, &3], |item, key| &item.0 == key, |item| &mut item.1).unwrap();
/// assert_eq!(*left, 3);
/// assert_eq!(*right, 4);
/// # }
/// ```
pub fn find_map_many<'a, I, T, U, F, M, K, const LEN: usize>(
    collection: I, keys: [&K; LEN], mut predicate: F, mut map: M,
) -> Option<[&'a mut U; LEN]>
where
    I: IntoIterator<Item = &'a mut T>,
    T: 'a,
    U: Debug,
    F: FnMut(&T, &K) -> bool,
    M: FnMut(&'a mut T) -> &'a mut U,
{
    let mut remaining = LEN;
    let mut output = Vec::with_capacity(LEN);
    (0..LEN).for_each(|_| output.push(None));

    'collection: for elem in collection {
        for (key, out) in std::iter::zip(&keys, &mut output) {
            if out.is_none() && predicate(elem, key) {
                *out = Some(map(elem));
                remaining -= 1;
                if remaining == 0 {
                    break 'collection;
                }
                break;
            }
        }
    }

    let Some(vec) = output.into_iter().collect::<Option<Vec<&mut U>>>() else {
        return None
    };
    Some(vec.try_into().unwrap())
}

#[cfg(test)]
mod test {

    use crate::map::find_map_many;

    #[test]
    fn test_scrambled_key() {
        let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let keys = [&4, &2];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        let unwrapped = res.unwrap();
        assert_eq!(*unwrapped[0], 5);
        assert_eq!(*unwrapped[1], 3);
    }

    #[test]
    fn test_duplicate_key() {
        let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let keys = [&2, &2];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        assert!(res.is_none());
    }

    #[test]
    fn test_duplicate_matching_keys() {
        let mut v = vec![(0, 1), (2, 3), (2, 4), (3, 4), (4, 5)];
        let keys = [&2, &2];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        let unwrapped = res.unwrap();
        assert_eq!(*unwrapped[0], 3);
        assert_eq!(*unwrapped[1], 4);
    }

    #[test]
    fn test_missing_key() {
        let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let keys = [&2, &7];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        assert!(res.is_none());
    }

    #[test]
    fn test_too_many_keys() {
        let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let keys = [&1, &2, &3, &4, &5, &5, &5];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        assert!(res.is_none());
    }

    #[test]
    fn test_zero_len() {
        let mut v = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        let keys: [&u64; 0] = [];
        let res = find_map_many(&mut v, keys, |item, key| &item.0 == key, |item| &mut item.1);
        assert_eq!(res, Some([]));
    }
}
// TODO: Unit tests for everything in here
