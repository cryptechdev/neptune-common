use std::{
    fmt::Debug,
    iter::FromIterator,
    ops::{Add, AddAssign, Div, Mul},
};

use cosmwasm_std::{Decimal256, Uint256};
use num_traits::Zero;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;

use crate::{
    asset::AssetInfo,
    error::{CommonError, CommonResult},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Map<K, V>(pub Vec<(K, V)>);

impl<K, V> Map<K, V>
where
    K: PartialEq + Clone + Debug,
{
    pub fn new() -> Self { vec![].into() }

    pub fn insert(&mut self, tuple: (K, V)) { self.0.push(tuple); }

    pub fn contains(&self, key: &K) -> bool { self.may_get(key).is_some() }

    pub fn position(&self, key: &K) -> Option<usize> { self.0.iter().position(|x| &x.0 == key) }

    pub fn get_mut_from_index(&mut self, index: usize) -> Option<&mut V> {
        match self.0.get_mut(index) {
            Some((_, val)) => Some(val),
            None => None,
        }
    }

    pub fn get(&self, key: &K) -> CommonResult<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&self.0[index].1),
            None => Err(CommonError::KeyNotFound(format!("{:?}", key.clone()))),
        }
    }

    pub fn may_get(&self, key: &K) -> Option<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&self.0[index].1),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> CommonResult<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&mut self.0[index].1),
            None => Err(CommonError::KeyNotFound(format!("{:?}", key.clone()))),
        }
    }

    pub fn may_get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&mut self.0[index].1),
            None => None,
        }
    }

    pub fn map_val<F: Fn(&V) -> O, O>(&mut self, f: F) -> Map<K, O> {
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

    pub fn get_mut_or_zero<'a, 'b>(&'a mut self, key: &'b K) -> &'a mut V
    where
        V: Zero,
    {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert((key.clone(), V::zero()));
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    pub fn get_mut_or_default<'a, 'b>(&'a mut self, key: &'b K) -> &'a mut V
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

    pub fn get_mut_or<'a, 'b>(&'a mut self, key: &'b K, val: V) -> &'a mut V {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert((key.clone(), val));
                &mut self.0.last_mut().unwrap().1
            }
        }
    }

    pub fn mul_all<U>(self, rhs: Map<K, U>) -> CommonResult<Map<K, <V as Mul<U>>::Output>>
    where
        V: Mul<U>,
        U: Clone,
    {
        let mut output = vec![];
        for (key, lhs_val) in self {
            let rhs_val = rhs.get(&key)?.clone();
            output.push((key, lhs_val * rhs_val))
        }
        Ok(output.into())
    }

    pub fn map<F, U>(&self, f: F) -> Map<K, U>
    where
        F: Fn(&V) -> U,
    {
        let mut vec = vec![];
        for (key, val) in &self.0 {
            vec.push((key.clone(), f(val)));
        }
        vec.into()
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

    pub fn remove_defaults(&mut self)
    where
        V: Default + PartialEq,
    {
        self.0.retain(|x| x.1 != V::default());
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

impl<K, V, U> Mul<Map<K, U>> for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Mul<U> + Clone,
{
    type Output = Map<K, <V as Mul<U>>::Output>;

    fn mul(self, rhs: Map<K, U>) -> Self::Output {
        let mut output = vec![];
        for rhs_val in rhs.0 {
            if let Some(val) = self.may_get(&rhs_val.0) {
                output.push((rhs_val.0, val.clone() * rhs_val.1))
            }
        }
        output.into()
    }
}

impl<K, V> Mul<Decimal256> for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Mul<Decimal256, Output = V> + Clone,
{
    type Output = Map<K, V>;

    fn mul(mut self, rhs: Decimal256) -> Self::Output {
        for (_, val) in &mut self {
            *val = val.clone() * rhs
        }
        self
    }
}

impl<K> Div for Map<K, Uint256>
where
    K: PartialEq + Clone + Debug,
{
    type Output = Map<K, Decimal256>;

    fn div(self, rhs: Map<K, Uint256>) -> Self::Output {
        let mut output = vec![];
        for rhs_val in rhs.0 {
            if let Some(val) = self.may_get(&rhs_val.0) {
                output.push((rhs_val.0, Decimal256::from_ratio(*val, rhs_val.1)))
            }
        }
        output.into()
    }
}

impl<K, V> Div<Decimal256> for Map<K, V>
where
    K: PartialEq + Clone + Debug,
    V: Div<Decimal256, Output = V> + Clone,
{
    type Output = Map<K, V>;

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
    fn add_assign(&mut self, rhs: Self) {
        for rhs_key_val in rhs {
            let lhs = self.get_mut_or_default(&rhs_key_val.0);
            *lhs = lhs.clone() + rhs_key_val.1;
        }
    }
}

impl<K, V> From<Vec<(K, V)>> for Map<K, V> {
    fn from(object: Vec<(K, V)>) -> Self { Map(object) }
}

impl<K, V> From<(K, V)> for Map<K, V> {
    fn from(object: (K, V)) -> Self { Map(vec![object]) }
}

pub trait GetKeyVec<K> {
    fn get_key_vec(&self) -> Vec<K>;
}

impl<T, K> GetKeyVec<K> for Vec<T>
where
    K: PartialEq + Clone,
    T: GetKeyVec<K>,
{
    fn get_key_vec(&self) -> Vec<K> {
        let mut key_vec = vec![];
        for val in self {
            for key in val.get_key_vec() {
                if !key_vec.contains(&key) {
                    key_vec.push(key.clone());
                }
            }
        }
        key_vec
    }
}

impl<K, V> GetKeyVec<K> for Map<K, V>
where
    K: PartialEq + Clone,
{
    fn get_key_vec(&self) -> Vec<K> {
        let mut key_vec = vec![];
        for (key, _) in &self.0 {
            if !key_vec.contains(key) {
                key_vec.push(key.clone());
            }
        }
        key_vec
    }
}

impl GetKeyVec<AssetInfo> for AssetInfo {
    fn get_key_vec(&self) -> Vec<AssetInfo> { vec![self.clone()] }
}

pub fn extract_keys<'a, K: 'a + PartialEq + Clone>(vec: Vec<&'a dyn GetKeyVec<K>>) -> Vec<K> {
    let mut asset_vec = vec![];
    for object in vec {
        for asset in object.get_key_vec() {
            if !asset_vec.contains(&asset) {
                asset_vec.push(asset.clone());
            }
        }
    }
    asset_vec
}

impl<K, V> Zeroed for Map<K, V>
where
    V: Zeroed,
{
    fn is_zeroed(&self) -> bool { self.iter().all(|x| x.1.is_zeroed()) }

    fn remove_zeroed(&mut self) { self.retain(|x| !x.1.is_zeroed()) }
}

/// Similar to is_empty, but allows for zeroed entries inside an iterator
/// [].is_zeroed ==true
/// [0, 0].is_zeroed == true
/// [0, 1].is_zeroed == false
pub trait Zeroed {
    fn is_zeroed(&self) -> bool;
    fn remove_zeroed(&mut self);
}
