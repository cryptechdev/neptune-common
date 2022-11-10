use std::{convert::TryInto, str::FromStr};

use clap::Subcommand;
use cosmwasm_std::{Addr, Coin, StdError, StdResult, Uint256};
use cw_storage_plus::{Bound, Bounder, Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    asset_map::{AssetMap, AssetVec},
    error::CommonError,
    parser::addr_parser,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Subcommand)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
#[repr(u8)]
pub enum AssetInfo {
    Token {
        /// "atom3h6lk23h6lk2j3has09d8fg"
        #[arg(value_parser=addr_parser)]
        contract_addr: Addr,
    },
    NativeToken {
        /// "uatom"
        denom: String,
    },
}

impl FromStr for AssetInfo {
    type Err = CommonError;

    /// TODO: Not rigorous, should only be used for command line
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 10 || s.starts_with("ibc") {
            Ok(Self::NativeToken { denom: s.to_string() })
        } else {
            Ok(Self::Token { contract_addr: Addr::unchecked(s) })
        }
    }
}

impl ToString for AssetInfo {
    fn to_string(&self) -> String {
        match self {
            Self::Token { contract_addr } => contract_addr.to_string(),
            Self::NativeToken { denom } => denom.clone(),
        }
    }
}

impl<'a> PrimaryKey<'a> for AssetInfo {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = u8;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            Self::Token { contract_addr: addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            Self::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
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

/// TODO: Might not be correct, Untested
impl<'a> Prefixer<'a> for AssetInfo {
    fn prefix(&self) -> Vec<Key> {
        match self {
            Self::Token { contract_addr: addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            Self::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

/// TODO: Might not be correct, Untested
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

impl<'a> From<AssetInfo> for Bound<'a, AssetInfo> {
    fn from(val: AssetInfo) -> Self { Bound::exclusive(val) }
}

impl<'a> From<&'a AssetInfo> for Bound<'a, &'a AssetInfo> {
    fn from(val: &'a AssetInfo) -> Self { Bound::exclusive(val) }
}

impl<'a> Bounder<'a> for AssetInfo {
    fn inclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::inclusive(self)) }

    fn exclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::exclusive(self)) }
}

impl<'a> Bounder<'a> for &'a AssetInfo {
    fn inclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::inclusive(self)) }

    fn exclusive_bound(self) -> Option<Bound<'a, Self>> { Some(Bound::exclusive(self)) }
}

impl KeyDeserialize for AssetInfo {
    type Output = Self;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(Self::Token { contract_addr: Addr::from_vec(split)? }),
            1 => Ok(Self::NativeToken { denom: String::from_vec(split)? }),
            _ => Err(StdError::GenericErr { msg: "Failed deserializing.".into() }),
        }
    }
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

impl From<AssetAmount> for AssetVec {
    fn from(val: AssetAmount) -> Self { vec![val.info].into() }
}

impl From<AssetAmount> for (AssetInfo, Uint256) {
    fn from(val: AssetAmount) -> Self { (val.info, val.amount) }
}

impl From<AssetMap<Uint256>> for AssetVec {
    fn from(val: AssetMap<Uint256>) -> Self {
        let mut asset_vec = vec![];
        for object in val {
            if !asset_vec.contains(&object.0) {
                asset_vec.push(object.0.clone());
            }
        }
        asset_vec.into()
    }
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
//     let k = Asset::NativeToken {
//         denom: "test".to_string(),
//     };
//     let path = k.key();
//     let asset_key_vec = Into::<Vec<u8>>::into(k.clone());

//     println!("asset_key = {:?}", asset_key_vec);
//     println!("path length = {:?}", path.len());

//     assert_eq!(asset_key_vec, [path[0].as_ref(), path[1].as_ref()].concat());
//     assert_eq!(k, Asset::from_vec(asset_key_vec.clone()).unwrap());

//     // let joined = k.joined_key();
//     // assert_eq!(joined, asset_key_vec);
// }
