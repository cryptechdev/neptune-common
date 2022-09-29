use std::{ops::Mul, iter::FromIterator};

use num_traits::Zero;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{error::{CommonResult, CommonError}, asset::AssetInfo};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct Map<K, V>(Vec<(K, V)>);

impl<K, V> Map<K, V>
where
    K: PartialEq + Clone
{
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut (K, V)> {
        self.0.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item=&(K, V)> {
        self.0.iter()
    }

    pub fn insert(&mut self, tuple: (K, V)) {
        self.0.push(tuple);
    }

    pub fn contains(&self, key: &K) -> bool {
        match self.may_get_ref(key) {
            Some(_) => true,
            None => false,
        }
    }

    /// This consumes the entire map, not a great idea to use.
    pub fn get(self, key: &K) -> CommonResult<V> {
        for val in self.0 {
            if &val.0 == key {
                return Ok(val.1)
            }
        }
        return Err(CommonError::KeyNotFound{});
    }

    /// This consumes the entire map, not a great idea to use.
    pub fn may_get(self, key: &K) -> Option<V> {
        for val in self.0 {
            if &val.0 == key {
                return Some(val.1)
            }
        }
        return None;
    }

    pub fn get_ref(&self, key: &K) -> CommonResult<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&self.0[index].1),
            None => Err(CommonError::KeyNotFound{}),
        }
    }

    pub fn may_get_ref(&self, key: &K) -> Option<&V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&self.0[index].1),
            None => None,
        }
    }

    pub fn get_ref_mut(&mut self, key: &K) -> CommonResult<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&mut self.0[index].1),
            None => Err(CommonError::KeyNotFound{}),
        }
    }

    pub fn may_get_ref_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => Some(&mut self.0[index].1),
            None => None,
        }
    }

    pub fn map_val<F: Fn(&V) -> O, O>(&mut self, f:F) -> Map<K, O> {
        let mut output = vec![];
        for (key, val) in &self.0 {
            let function_output  = f(&val);
            output.push((key.clone(), function_output));
        }
        output.into()
    }

    pub fn map_result_val<F: Fn(&V) -> Result<O, E>, O, E>(&mut self, f:F) -> Result<Map<K, O>, E>
    {
        let mut output = vec![];
        for (key, val) in &self.0 {
            let function_output  = f(&val)?;
            output.push((key.clone(), function_output));
        }
        Ok(output.into())
    }

    pub fn get_or_zero_mut<'a, 'b>(&'a mut self, key: &'b K) -> &'a mut V 
    where V: Zero
    {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert((key.clone(), V::zero()));
                &mut self.0.last_mut().unwrap().1
            },
        }
    }

    pub fn get_mut_or<'a, 'b>(&'a mut self, key: &'b K, val: V) -> &'a mut V 
    {
        match self.0.iter().position(|x| &x.0 == key) {
            Some(index) => &mut self.0[index].1,
            None => {
                self.insert((key.clone(), val));
                &mut self.0.last_mut().unwrap().1
            },
        }
    }

    pub fn mul_all<U>(self, rhs: Map<K, U>) -> CommonResult<Map<K, <V as Mul<U>>::Output>>
    where
        V: Mul<U>, U: Clone
    {
        let mut output = vec![];
        for (key, lhs_val) in self {
            let rhs_val = rhs.get_ref(&key)?.clone();
            output.push((key, lhs_val * rhs_val ))
        }
        Ok(output.into())
    }

    pub fn map<F, U>(&self, f: F) -> Map<K, U> 
    where F: Fn(&V) -> U
    {
        let mut vec = vec![];
        for (key, val) in &self.0 {
            vec.push((key.clone(), f(val)));
        }
        vec.into()
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self { 0: vec![] }
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Vec::<(K, V)>::from_iter(iter).into()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);

    type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = &'a (K, V);

    type IntoIter = <&'a Vec<(K, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type Item = &'a mut (K, V);

    type IntoIter = <&'a mut Vec<(K, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V, U> Mul<Map<K, U>> for Map<K, V>
where
    K: PartialEq + Clone,
    V: Mul<U> + Clone
{
    type Output = Map<K, <V as Mul<U>>::Output>;

    fn mul(self, rhs: Map<K, U>) -> Self::Output {
        let mut output = vec![];
        for rhs_val in rhs.0 {
            if let Some(val) = self.may_get_ref(&rhs_val.0) {
                output.push((rhs_val.0, val.clone() * rhs_val.1 ))
            }
        }
        output.into()
    }
}

impl<K, V> From<Vec<(K, V)>> for Map<K, V> {
    fn from(object: Vec<(K, V)>) -> Self {
        Map(object)
    }
}

pub trait GetKeyVec<K> {
    fn get_key_vec(&self) -> Vec<K>;
}

impl<K, V> GetKeyVec<K> for Map<K, V> 
where
    K: PartialEq + Clone
{
    fn get_key_vec(&self) -> Vec<K> {
        let mut key_vec = vec![];
        for (key, _) in &self.0 {
            if !key_vec.contains(key) {
                key_vec.push(key.clone());
            }
        }
        key_vec.into()
    }
}

impl GetKeyVec<AssetInfo> for AssetInfo 
{
    fn get_key_vec(&self) -> Vec<AssetInfo> {
        vec![self.clone()]
    }
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
    asset_vec.into()
}

// pub fn extract_keys<'a, K: 'a + PartialEq + Clone, I: IntoIterator<Item = &'a dyn GetKeyVec<K>>>(iter: I) -> Vec<K> {
//     let mut asset_vec = vec![];
//     for object in iter.into_iter() {
//         for asset in object.get_key_vec() {
//             if !asset_vec.contains(&asset) {
//                 asset_vec.push(asset.clone());
//             }
//         }
//     }
//     asset_vec.into()
// }