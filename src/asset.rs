use std::{hash::Hash};
use cosmwasm_std::{Uint256, Addr, StdResult};
use cw_storage_plus::{PrimaryKey, Key, Prefixer, KeyDeserialize, Bound};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema)]
#[repr(u8)]
pub enum Asset {
    Token{ addr: Addr },
    NativeToken{ denom: String },
}

impl<'a> PrimaryKey<'a> for Asset {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            Asset::Token { addr } => vec![Key::Ref(addr.as_bytes())],
            Asset::NativeToken { denom } => vec![Key::Ref(denom.as_bytes())],
        }
    }
}

impl<'a> Prefixer<'a> for Asset {
    fn prefix(&self) -> Vec<Key> {
        match self {
            Asset::Token { addr } => addr.prefix(),
            Asset::NativeToken { denom } => denom.prefix(),
        }
    }
}

impl<'a> Into<Bound<'a, Asset>> for Asset {
    fn into(self) -> Bound<'a, Asset> {
        Bound::exclusive(self.as_slice().to_vec())
    }
}

impl KeyDeserialize for Asset {
    type Output = Asset;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        if let Ok(addr) = Addr::from_vec(value.clone()) {
            Ok(Asset::Token { addr })
        } else {
            Ok(Asset::NativeToken { denom: String::from_vec(value)? })
        }
    }
}

impl From<Vec<u8>> for Asset {
    fn from(array: Vec<u8>) -> Self {
        Self::from_vec(array).unwrap()
    }
}

impl Asset {
    fn as_slice(&self) -> &[u8] {
        match self {
            Asset::Token { addr } => {
                addr.as_bytes()
            },
            Asset::NativeToken { denom } => {
                denom.as_bytes()
            },
        } 
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AssetAmount {
    pub asset_info: Asset,
    pub amount: Uint256,
}