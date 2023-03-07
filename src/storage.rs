use cosmwasm_std::{Addr, Deps, DepsMut, Order};
use cw_storage_plus::{Bounder, KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::{
    asset::AssetInfo,
    error::{CommonError, CommonResult},
    neptune_map::*,
};

pub const PARAMS_KEY: &str = "params";
pub const STATE_KEY: &str = "state";

pub enum Method<K> {
    Paginate { start_after: Option<K>, limit: Option<u32> },
    Select { keys: Vec<K> },
}

pub trait KeyToOutput {
    type Output;
    fn to_output(self) -> Self::Output;
}

impl KeyToOutput for &Addr {
    type Output = Addr;
    fn to_output(self) -> Self::Output {
        self.clone()
    }
}

impl KeyToOutput for &AssetInfo {
    type Output = AssetInfo;
    fn to_output(self) -> Self::Output {
        self.clone()
    }
}

pub fn read_map<'k, K, O, V>(deps: Deps, method: Method<K>, map: Map<'k, K, V>) -> Result<NeptuneMap<O, V>, CommonError>
where
    K: Bounder<'k> + PrimaryKey<'k> + KeyDeserialize<Output = O> + KeyToOutput<Output = O>,
    O: 'static,
    V: Serialize + DeserializeOwned,
{
    match method {
        Method::Paginate { start_after, limit } => paginate(deps, start_after, limit, map),
        Method::Select { keys } => select(deps, keys, map),
    }
}

/// Reads a map from storage is ascending order.
pub fn paginate<'k, K, O, V>(
    deps: Deps, start_after: Option<K>, limit: Option<u32>, map: Map<'k, K, V>,
) -> Result<NeptuneMap<O, V>, CommonError>
where
    K: Bounder<'k> + PrimaryKey<'k> + KeyDeserialize<Output = O>,
    O: 'static,
    V: Serialize + DeserializeOwned,
{
    let start = start_after.map(|key| key.exclusive_bound().unwrap());
    let vec = match limit {
        Some(limit) => map
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit as usize)
            .collect::<Result<Vec<_>, _>>()?,
        None => map.range(deps.storage, start, None, Order::Ascending).collect::<Result<Vec<_>, _>>()?,
    };
    Ok(vec.into())
}

/// Loads a specific set of values from a map.
pub fn select<'k, K, O, V>(deps: Deps, keys: Vec<K>, map: Map<'k, K, V>) -> Result<NeptuneMap<O, V>, CommonError>
where
    K: Bounder<'k> + PrimaryKey<'k> + KeyDeserialize<Output = O> + KeyToOutput<Output = O>,
    O: 'static,
    V: Serialize + DeserializeOwned,
{
    keys.into_iter()
        .map(|asset| {
            let value = map.load(deps.storage, asset.clone())?;
            Ok((asset.to_output(), value))
        })
        .collect::<CommonResult<NeptuneMap<O, _>>>()
}

/// Trait for types which act as a storage cache with cosmwasm storage plus.
pub trait Cacher<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    fn must_get_mut(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&mut V>;
    fn must_get(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&V>;
}

/// The inner part of the cache which keeps track of wether the value has been modified.
pub struct CacheInner<V>
where
    V: Clone + Serialize + DeserializeOwned,
{
    value: V,
    is_modified: bool,
}

/// A cache which stores values in memory to avoid repeated disk reads/writes.
pub struct Cache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    map: NeptuneMap<K, CacheInner<V>>,
    storage: Map<'s, &'k K, V>,
}

impl<'s, 'k, K, V> Cache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    pub const fn new(storage: Map<'s, &'k K, V>) -> Self {
        Self { map: NeptuneMap::new(), storage }
    }

    pub fn save(&mut self, deps: DepsMut<'_>) -> CommonResult<()> {
        for (key, inner) in self.map.iter() {
            if inner.is_modified {
                self.storage.save(deps.storage, key, &inner.value)?;
            }
        }
        Ok(())
    }
}

impl<'s, 'k, K, V> Cacher<'s, 'k, K, V> for Cache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    fn must_get_mut(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&mut V> {
        match self.map.iter().position(|x| &x.0 == key) {
            Some(index) => {
                let inner = &mut self.map.0[index].1;
                inner.is_modified = true;
                Ok(&mut inner.value)
            }
            None => {
                let value = self.storage.load(deps.storage, key)?;
                let inner = CacheInner { value, is_modified: true };
                self.map.insert(key.clone(), inner);
                Ok(&mut self.map.last_mut().unwrap().1.value)
            }
        }
    }

    fn must_get(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&V> {
        match self.map.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&self.map.0[index].1.value),
            None => {
                let value = self.storage.load(deps.storage, key)?;
                let inner = CacheInner { value, is_modified: false };
                self.map.insert(key.clone(), inner);
                Ok(&self.map.last().unwrap().1.value)
            }
        }
    }
}

/// A cache which stores values in memory to avoid repeated disk reads/writes.
/// Values are accessed through a raw query to another contracts storage.
pub struct QueryCache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    map: NeptuneMap<K, V>,
    storage: Map<'s, &'k K, V>,
    addr: Addr,
}

impl<'s, 'k, K, V> QueryCache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    pub fn new(storage: Map<'s, &'k K, V>, addr: Addr) -> Self {
        Self { map: NeptuneMap::new(), storage, addr }
    }
}

impl<'s, 'k, K, V> Cacher<'s, 'k, K, V> for QueryCache<'s, 'k, K, V>
where
    for<'a> &'a K: Debug + PartialEq + Eq + PrimaryKey<'a>,
    K: Clone + Debug + PartialEq + Eq,
    V: Clone + Serialize + DeserializeOwned,
{
    fn must_get_mut(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&mut V> {
        match self.map.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&mut self.map.0[index].1),
            None => {
                let value = self
                    .storage
                    .query(&deps.querier, self.addr.clone(), key)?
                    .ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))?;
                self.map.insert(key.clone(), value);
                Ok(&mut self.map.last_mut().unwrap().1)
            }
        }
    }

    fn must_get(&mut self, deps: Deps<'_>, key: &K) -> CommonResult<&V> {
        match self.map.iter().position(|x| &x.0 == key) {
            Some(index) => Ok(&self.map.0[index].1),
            None => {
                let value = self
                    .storage
                    .query(&deps.querier, self.addr.clone(), key)?
                    .ok_or_else(|| CommonError::KeyNotFound(format!("{key:?}")))?;
                self.map.insert(key.clone(), value);
                Ok(&self.map.last().unwrap().1)
            }
        }
    }
}
