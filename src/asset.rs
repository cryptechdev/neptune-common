use cosmwasm_std::{Addr, StdError, StdResult, Uint256};
use cw_storage_plus::{Bound, Key, KeyDeserialize, PrimaryKey, Prefixer};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema)]
#[repr(u8)]
pub enum Asset {
    Token { addr: Addr },
    NativeToken { denom: String },
}

impl<'a> PrimaryKey<'a> for Asset {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = u8;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            Asset::Token { addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            Asset::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

impl<'a> PrimaryKey<'a> for &'a Asset {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = u8;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        match self {
            Asset::Token { addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            Asset::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

/// Might not be correct, Untested
impl<'a> Prefixer<'a> for Asset {
    fn prefix(&self) -> Vec<Key> {
        match self {
            Asset::Token { addr } => {
                vec![Key::Ref(addr.as_bytes()), Key::Val8([0])]
            }
            Asset::NativeToken { denom } => {
                vec![Key::Ref(denom.as_bytes()), Key::Val8([1])]
            }
        }
    }
}

impl<'a> Into<Bound<'a, Asset>> for Asset {
    fn into(self) -> Bound<'a, Asset> { Bound::exclusive(self) }
}

impl<'a> Into<Bound<'a, &'a Asset>> for &'a Asset {
    fn into(self) -> Bound<'a, &'a Asset> { Bound::exclusive(self) }
}

impl KeyDeserialize for Asset {
    type Output = Asset;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(Asset::Token {
                addr: Addr::from_vec(split)?,
            }),
            1 => Ok(Asset::NativeToken {
                denom: String::from_vec(split)?,
            }),
            _ => Err(StdError::GenericErr {
                msg: "Failed deserializing.".into(),
            }),
        }
    }
}

impl<'a> KeyDeserialize for &'a Asset {
    type Output = Asset;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(Asset::Token {
                addr: Addr::from_vec(split)?,
            }),
            1 => Ok(Asset::NativeToken {
                denom: String::from_vec(split)?,
            }),
            _ => Err(StdError::GenericErr {
                msg: "Failed deserializing.".into(),
            }),
        }
    }
}

// impl From<Vec<u8>> for Asset {
//     fn from(array: Vec<u8>) -> Self { Self::from_vec(array).unwrap() }
// }

// impl Into<Vec<u8>> for Asset {
//     fn into(self) -> Vec<u8> {
//         match self {
//             Asset::Token { addr } => {
//                 //let prefix = Into::<Vec<u8>>::into(0u8);
//                 let mut vec = addr.as_bytes().to_vec();
//                 vec.push(0u8);
//                 vec
//             }
//             Asset::NativeToken { denom } => {
//                 let mut vec = denom.as_bytes().to_vec();
//                 vec.push(1u8);
//                 vec
//             }
//         }
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AssetAmount {
    pub asset_info: Asset,
    pub amount:     Uint256,
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

// #[test]
// fn test_map() {
//     let
//     pub const ASSETS: Map<Asset, String> = Map::new("assets");

// }