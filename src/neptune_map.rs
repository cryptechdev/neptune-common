use std::{
    fmt::Debug,
    iter::FromIterator,
    ops::{Add, AddAssign, Mul},
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal256;
use shrinkwraprs::Shrinkwrap;

use crate::{
    error::{CommonError, CommonResult},
    traits::{KeyVec, Zeroed},
};

/// A map that uses a vector as its underlying data structure.
#[cw_serde]
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct NeptuneMap<K, V>(pub Vec<(K, V)>);

impl<K, V> NeptuneMap<K, V>
where
    K: PartialEq + Clone + Debug,
{
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.get_mut(&key) {
            Some(value_mut) => Some(std::mem::replace(value_mut, value)),
            None => {
                self.0.push((key, value));
                None
            }
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&self.0[index].1),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&mut self.0[index].1),
            None => None,
        }
    }

    pub fn must_get(&self, key: &K) -> CommonResult<&V> {
        self.get(key).ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))
    }

    pub fn must_get_mut(&mut self, key: &K) -> CommonResult<&mut V> {
        self.get_mut(key).ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))
    }

    pub fn get_mut_or_default<'a>(&'a mut self, key: &K) -> &'a mut V
    where
        V: Default,
    {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert(key.clone(), V::default());
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    /// multiplies every value in self with the corresponding value in rhs. Returns an error if rhs
    /// is missing a key. Rhs must contain every key in self, but self needs not contain every key
    /// in rhs.
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// let quantity: NeptuneMap<_, _> = vec![("cars", 2.0), ("bikes", 3.0)].into();
    /// let prices: NeptuneMap<_, _> = vec![("cars", 2.0), ("bikes", 1.0)].into();
    /// let values = quantity.mul_all(&prices).unwrap();
    /// assert_eq!(values, vec![("cars", 4.0), ("bikes", 3.0)].into());
    /// ```
    pub fn mul_all<U>(self, rhs: &NeptuneMap<K, U>) -> CommonResult<NeptuneMap<K, <V as Mul<U>>::Output>>
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

    /// Sums all values in the map.
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// let this: NeptuneMap<_, _> = vec![("cars", 2), ("bikes", 3)].into();
    /// let total = this.sum();
    /// assert_eq!(total, 5);
    /// ```
    pub fn sum(&self) -> V
    where
        V: Default + Add<Output = V> + Clone,
    {
        self.iter().fold(V::default(), |acc, (_, val)| acc + val.clone())
    }
}

impl<K, V> Default for NeptuneMap<K, V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K, V> FromIterator<(K, V)> for NeptuneMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Vec::<(K, V)>::from_iter(iter).into()
    }
}

impl<K, V> IntoIterator for NeptuneMap<K, V> {
    type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a NeptuneMap<K, V> {
    type IntoIter = <&'a Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = &'a (K, V);

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut NeptuneMap<K, V> {
    type IntoIter = <&'a mut Vec<(K, V)> as IntoIterator>::IntoIter;
    type Item = &'a mut (K, V);

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V> Mul<Decimal256> for NeptuneMap<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Mul<Decimal256, Output = V> + Clone,
{
    type Output = Self;

    /// multiplies each value in the map with a Decimal256
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// # use cosmwasm_std::{Uint256, Decimal256};
    /// # use std::str::FromStr;
    /// let map: NeptuneMap<_, _> =
    ///     vec![("foo", Uint256::from(2u64)), ("bar", Uint256::from(3u64))].into();
    /// let decimal = Decimal256::from_str("2.0").unwrap();
    /// let result = map * decimal;
    /// assert_eq!(result, vec![("foo", Uint256::from(4u64)), ("bar", Uint256::from(6u64))].into());
    /// ```
    fn mul(mut self, rhs: Decimal256) -> Self::Output {
        for (_, val) in &mut self {
            *val = val.clone() * rhs
        }
        self
    }
}

impl<K, V> Add for NeptuneMap<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Add<Output = V> + Clone + Default,
{
    type Output = Self;

    /// Adds the corresponding values from two maps together.
    ///
    /// If a key exists in one map but not the other, the default is used.
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// let this: NeptuneMap<_, _> = vec![("foo", 2), ("bar", 3)].into();
    /// let that: NeptuneMap<_, _> = vec![("bar", 1), ("baz", 4)].into();
    /// let sum = this + that;
    /// assert_eq!(sum, vec![("foo", 2), ("bar", 4), ("baz", 4)].into());
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() + rhs_key_val.1;
        }
        self
    }
}

impl<K, V> AddAssign for NeptuneMap<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Add<Output = V> + Clone + Default,
{
    /// Adds the corresponding values from two maps together.
    ///
    /// If a key exists in one map but not the other, the default is used.
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// let mut this: NeptuneMap<_, _> = vec![("foo", 2), ("bar", 3)].into();
    /// let that: NeptuneMap<_, _> = vec![("bar", 1), ("baz", 4)].into();
    /// this += that;
    /// assert_eq!(this, vec![("foo", 2), ("bar", 4), ("baz", 4)].into());
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() + rhs_key_val.1;
        }
    }
}

impl<K, V> From<Vec<(K, V)>> for NeptuneMap<K, V> {
    fn from(object: Vec<(K, V)>) -> Self {
        Self(object)
    }
}

impl<K, V> From<(K, V)> for NeptuneMap<K, V> {
    fn from(object: (K, V)) -> Self {
        Self(vec![object])
    }
}

impl<K, V> Zeroed for NeptuneMap<K, V>
where
    V: Zeroed,
{
    fn is_zeroed(&self) -> bool {
        self.iter().all(|x| x.1.is_zeroed())
    }

    fn remove_zeroed(&mut self) {
        self.iter_mut().for_each(|x| x.1.remove_zeroed());
        self.retain(|x| !x.1.is_zeroed())
    }
}

impl<K, V> KeyVec<K> for NeptuneMap<K, V>
where
    K: PartialEq + Ord + Clone,
{
    /// Adds the corresponding values from two maps together.
    ///
    /// If a key exists in one map but not the other, the default is used.
    /// ```
    /// # use neptune_common::neptune_map::NeptuneMap;
    /// # use neptune_common::traits::KeyVec;
    /// let mut map: NeptuneMap<_, _> = vec![("foo", 2), ("bar", 3)].into();
    /// let key_vec = map.key_vec();
    /// assert_eq!(key_vec, vec!["foo", "bar"]);
    /// ```
    fn key_vec(&self) -> Vec<K> {
        // We don't need to worry about deduping here because the keys are unique
        self.iter().map(|(key, _)| key.clone()).collect::<Vec<_>>()
    }
}
