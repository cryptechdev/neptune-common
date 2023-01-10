use std::{
    collections::BTreeMap,
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal256;
use shrinkwraprs::Shrinkwrap;

use crate::{
    error::{CommonError, CommonResult},
    traits::{KeyVec, Zeroed},
};

#[cw_serde]
#[derive(Eq, Shrinkwrap, PartialOrd, Ord)]
#[shrinkwrap(mutable)]
pub struct NeptuneMap<K: Ord, V>(pub BTreeMap<K, V>);

impl<K, V> NeptuneMap<K, V>
where
    K: Ord,
{
    pub const fn new() -> Self { Self(BTreeMap::new()) }

    pub fn must_get(&self, key: &K) -> CommonResult<&V>
    where
        K: Debug,
    {
        self.get(key).ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))
    }

    pub fn must_get_mut(&mut self, key: &K) -> CommonResult<&mut V>
    where
        K: Debug,
    {
        self.get_mut(key).ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))
    }

    pub fn get_mut_or_default(&mut self, key: &K) -> &mut V
    where
        K: Debug + Clone,
        V: Default,
    {
        match self.contains_key(key) {
            true => self.get_mut(key).unwrap(),
            false => {
                self.insert(key.clone(), V::default());
                self.get_mut(key).unwrap()
            }
        }
    }

    /// multiplies every value in self with the corresponding value in rhs. Returns an error if rhs
    /// is missing a key. Rhs must contain every key in self, but self needs not contain every key
    /// in rhs.
    pub fn mul_all<U>(self, rhs: &NeptuneMap<K, U>) -> CommonResult<NeptuneMap<K, <V as Mul<U>>::Output>>
    where
        K: Debug,
        V: Mul<U>,
        U: Clone,
    {
        self.into_iter()
            .map(|(key, val)| {
                let value = val * rhs.must_get(&key)?.clone();
                Ok((key, value))
            })
            .collect::<Result<NeptuneMap<_, _>, _>>()
    }

    pub fn sum(&self) -> V
    where
        V: Default + Add<Output = V> + Clone,
    {
        self.iter().fold(V::default(), |acc, (_, val)| acc + val.clone())
    }
}

impl<K, V> Default for NeptuneMap<K, V>
where
    K: Ord,
{
    fn default() -> Self { Self::new() }
}

impl<K, V> FromIterator<(K, V)> for NeptuneMap<K, V>
where
    K: Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self { Self(BTreeMap::<K, V>::from_iter(iter)) }
}

impl<K, V> IntoIterator for NeptuneMap<K, V>
where
    K: Ord,
{
    type IntoIter = <BTreeMap<K, V> as IntoIterator>::IntoIter;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, K, V> IntoIterator for &'a NeptuneMap<K, V>
where
    K: Ord,
{
    type IntoIter = <&'a BTreeMap<K, V> as IntoIterator>::IntoIter;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<'a, K, V> IntoIterator for &'a mut NeptuneMap<K, V>
where
    K: Ord,
{
    type IntoIter = <&'a mut BTreeMap<K, V> as IntoIterator>::IntoIter;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> Self::IntoIter { self.0.iter_mut() }
}

impl<K, V> Mul<Decimal256> for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
    V: Mul<Decimal256, Output = V> + Clone,
{
    type Output = Self;

    /// multiplies each value with a Decimal256
    fn mul(self, rhs: Decimal256) -> Self::Output { self.into_iter().map(|(key, val)| (key, val * rhs)).collect() }
}

impl<K, V> Div<Decimal256> for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
    V: Div<Decimal256, Output = V> + Clone,
{
    type Output = Self;

    /// Divides each value with a Decimal256
    fn div(self, rhs: Decimal256) -> Self::Output { self.into_iter().map(|(key, val)| (key, val / rhs)).collect() }
}

impl<K, V> Add for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
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

impl<K, V> AddAssign for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
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

impl<K, V> Sub for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
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

impl<K, V> SubAssign for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone + Debug,
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

impl<K, V> From<Vec<(K, V)>> for NeptuneMap<K, V>
where
    K: Ord,
{
    fn from(object: Vec<(K, V)>) -> Self { object.into_iter().collect() }
}

impl<K, V> From<(K, V)> for NeptuneMap<K, V>
where
    K: Ord,
{
    fn from(object: (K, V)) -> Self {
        let mut output = Self::new();
        output.insert(object.0, object.1);
        output
    }
}

impl<K, V> Zeroed for NeptuneMap<K, V>
where
    K: Ord,
    V: Zeroed,
{
    fn is_zeroed(&self) -> bool { self.iter().all(|x| x.1.is_zeroed()) }

    fn remove_zeroed(&mut self) {
        self.iter_mut().for_each(|x| x.1.remove_zeroed());
        self.retain(|_, val| !val.is_zeroed())
    }
}

impl<K, V> KeyVec<K> for NeptuneMap<K, V>
where
    K: Ord + PartialEq + Clone,
{
    fn key_vec(&self) -> Vec<K> { self.iter().map(|(key, _)| key.clone()).collect() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut map = NeptuneMap::new();
        map.insert("Hello", "world");
        assert!(map.contains_key("Hello"));
    }
}
