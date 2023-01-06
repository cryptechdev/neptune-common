use std::convert::TryInto;

use cosmwasm_std::{Addr, Coin, StdError, StdResult, Uint256};
use cw_storage_plus::{Bound, Bounder, Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::map::Map;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

pub type AssetMap<T> = Map<AssetInfo, T>;

impl ToString for AssetInfo {
    fn to_string(&self) -> String {
        match self {
            Self::Token { contract_addr } => contract_addr.to_string(),
            Self::NativeToken { denom } => denom.clone(),
        }
    }
}

impl<'a> PrimaryKey<'a> for &'a AssetInfo {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = u8;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            AssetInfo::Token { contract_addr: addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            AssetInfo::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

impl<'a> Prefixer<'a> for &'a AssetInfo {
    fn prefix(&self) -> Vec<Key> {
        match self {
            AssetInfo::Token { contract_addr: addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            AssetInfo::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

impl<'a> Bounder<'a> for &'a AssetInfo {
    fn inclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::inclusive(self)) }

    fn exclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::exclusive(self)) }
}

impl<'a> KeyDeserialize for &'a AssetInfo {
    type Output = AssetInfo;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(AssetInfo::Token { contract_addr: Addr::from_vec(split)? }),
            1 => Ok(AssetInfo::NativeToken { denom: String::from_vec(split)? }),
            _ => Err(StdError::GenericErr { msg: "Failed deserializing.".into() }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename = "Asset")]
pub struct AssetAmount {
    pub info:   AssetInfo,
    pub amount: Uint256,
}

impl From<AssetAmount> for (AssetInfo, Uint256) {
    fn from(val: AssetAmount) -> Self { (val.info, val.amount) }
}

impl From<Coin> for AssetAmount {
    fn from(coin: Coin) -> Self {
        Self { info: AssetInfo::NativeToken { denom: coin.denom }, amount: coin.amount.into() }
    }
}

impl TryInto<Coin> for AssetAmount {
    type Error = StdError;

    fn try_into(self) -> Result<Coin, Self::Error> {
        match self.info {
            AssetInfo::Token { .. } => Err(StdError::GenericErr { msg: "Cannot convert to AssetAmount".into() }),
            AssetInfo::NativeToken { denom } => Ok(Coin { denom, amount: self.amount.try_into().unwrap() }),
        }
    }
}

impl From<&Coin> for AssetAmount {
    fn from(coin: &Coin) -> Self {
        Self { info: AssetInfo::NativeToken { denom: coin.denom.clone() }, amount: coin.amount.into() }
    }
}

// #[test]
// fn asset_key_works() {
//     let k = Asset::NativeToken { denom: "test".to_string() };
//     let path = k.key();
//     let asset_key_vec = Into::<Vec<u8>>::into(k.clone());

//     println!("asset_key = {:?}", asset_key_vec);
//     println!("path length = {:?}", path.len());

//     assert_eq!(asset_key_vec, [path[0].as_ref(), path[1].as_ref()].concat());
//     assert_eq!(k, Asset::from_vec(asset_key_vec.clone()).unwrap());

//     // let joined = k.joined_key();
//     // assert_eq!(joined, asset_key_vec);
// }
