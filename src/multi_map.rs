use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Map, Path, PrimaryKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct MultiMap<'a, K1, K2, V> {
    main_map: Map<'a, K1, (K2, V)>,
    key_map:  Map<'a, K2, K1>,
}

impl<'a, K1, K2, V> MultiMap<'a, K1, K2, V>
where
    V: Serialize,
    for<'de> V: Deserialize<'de>,
    (K2, V): Serialize,
    (K2, V): DeserializeOwned,
    K1: PrimaryKey<'a>,
    K1: Serialize,
    K1: DeserializeOwned,
    K2: PrimaryKey<'a>,
{
    pub const fn new(namespace1: &'a str, namespace2: &'a str) -> Self {
        Self { main_map: Map::new(namespace1), key_map: Map::new(namespace2) }
    }

    pub fn key1(&self, key1: K1) -> Path<(K2, V)> { self.main_map.key(key1) }

    pub fn key2(&self, store: &dyn Storage, key2: K2) -> StdResult<Path<(K2, V)>> {
        let key1 = self.key_map.load(store, key2)?;
        Ok(self.key1(key1))
    }

    pub fn load1(&self, store: &dyn Storage, key1: K1) -> StdResult<V> { Ok(self.main_map.load(store, key1)?.1) }

    pub fn load2(&self, store: &dyn Storage, key2: K2) -> StdResult<V> {
        let key1 = self.key_map.load(store, key2)?;
        self.load1(store, key1)
    }

    pub fn save(&self, store: &mut dyn Storage, key1: K1, key2: K2, data: V) -> StdResult<()> {
        let tuple = (key2.clone(), data);
        self.main_map.save(store, key1.clone(), &tuple)?;
        self.key_map.save(store, key2, &key1)?;
        Ok(())
    }
}
