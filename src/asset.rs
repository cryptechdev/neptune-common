use std::{convert::TryInto};

use cosmwasm_std::{Addr, StdError, StdResult, Uint256, Coin, Decimal256};
use cw_storage_plus::{Bound, Key, KeyDeserialize, PrimaryKey, Prefixer};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{error::{CommonResult, CommonError}, math::{get_difference_or_zero}};
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

impl<'a> PrimaryKey<'a> for AssetInfo {
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

/// Might not be correct, Untested
impl<'a> Prefixer<'a> for AssetInfo {
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

/// Might not be correct, Untested
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

impl<'a> Into<Bound<'a, AssetInfo>> for AssetInfo {
    fn into(self) -> Bound<'a, AssetInfo> { Bound::exclusive(self) }
}

impl<'a> Into<Bound<'a, &'a AssetInfo>> for &'a AssetInfo {
    fn into(self) -> Bound<'a, &'a AssetInfo> { Bound::exclusive(self) }
}

impl KeyDeserialize for AssetInfo {
    type Output = AssetInfo;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(AssetInfo::Token {
                contract_addr: Addr::from_vec(split)?,
            }),
            1 => Ok(AssetInfo::NativeToken {
                denom: String::from_vec(split)?,
            }),
            _ => Err(StdError::GenericErr {
                msg: "Failed deserializing.".into(),
            }),
        }
    }
}

impl<'a> KeyDeserialize for &'a AssetInfo {
    type Output = AssetInfo;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        
        let mut split = value.split_off(2);

        match split.pop().unwrap() {
            0 => Ok(AssetInfo::Token {
                contract_addr: Addr::from_vec(split)?,
            }),
            1 => Ok(AssetInfo::NativeToken {
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
#[serde(rename ="Asset")]
pub struct AssetAmount {
    pub info:       AssetInfo,
    pub amount:     Uint256,
}

impl From<Coin> for AssetAmount {
    fn from(coin: Coin) -> Self {
        AssetAmount {
            info: AssetInfo::NativeToken { denom: coin.denom },
            amount: coin.amount.into()
        }
    }
}

impl TryInto<Coin> for AssetAmount {
    type Error = StdError;

    fn try_into(self) -> Result<Coin, Self::Error> {
        match self.info {
            AssetInfo::Token { .. } => {
                return Err(StdError::GenericErr { msg: "Cannot convert to AssetAmount".into() })
            },
            AssetInfo::NativeToken { denom } => {
                Ok(Coin {
                    denom,
                    amount: self.amount.try_into().unwrap(),
                })
            },
        }
    }
}

impl From<&Coin> for AssetAmount {
    fn from(coin: &Coin) -> Self {
        AssetAmount {
            info: AssetInfo::NativeToken { denom: coin.denom.clone() },
            amount: coin.amount.into()
        }
    }
}

pub fn get_or_zero_mut<'a, 'b>(asset_amounts: &'a mut Vec<AssetAmount>, info: &'b AssetInfo)
-> &'a mut Uint256 {
    match asset_amounts.iter().position(|x| &x.info == info) {
        Some(index) => {
            &mut asset_amounts.get_mut(index).unwrap().amount
        },
        None => {
            let asset_amount = AssetAmount {
                info: info.clone(),
                amount: Uint256::zero(),
            };
            asset_amounts.push(asset_amount);
            &mut asset_amounts.last_mut().unwrap().amount
        },
    }
}

pub fn get_amount<'a, 'b>(asset_amounts: &'a Vec<AssetAmount>, info: &'b AssetInfo)
-> Option<&'a Uint256> {
    match asset_amounts.iter().position(|x| &x.info == info) {
        Some(index) => {
            Some(&asset_amounts.get(index).unwrap().amount)
        },
        None => {
            None
        },
    }
}

pub fn get_amount_mut<'a, 'b>(asset_amounts: &'a mut Vec<AssetAmount>, info: &'b AssetInfo)
-> Option<&'a mut Uint256> {
    match asset_amounts.iter().position(|x| &x.info == info) {
        Some(index) => {
            Some(&mut asset_amounts.get_mut(index).unwrap().amount)
        },
        None => {
            None
        },
    }
}

pub enum Quantity {
    Shares(Uint256),
    Amount(Uint256)
}

pub fn add_to_pool(
    quantity: Quantity,
    info: &AssetInfo,
    pool_principles:    &mut Vec<AssetAmount>,
    pool_shares:        &mut Vec<AssetAmount>,
    account_principles: &mut Vec<AssetAmount>,
    account_shares:     &mut Vec<AssetAmount>, 
) {
    let pool_principle = get_or_zero_mut(pool_principles, info);
    let pool_shares = get_or_zero_mut(pool_shares, info);
    let account_shares = get_or_zero_mut(account_shares, info);
    let account_principle = get_or_zero_mut(account_principles, info);

    let shares_to_issue;
    let amount_to_issue;

    match quantity {
        Quantity::Shares(shares) => {
            shares_to_issue = shares;
            amount_to_issue = shares_to_issue * *pool_principle / *pool_shares
        },
        Quantity::Amount(amount) => {
            amount_to_issue = amount;
            shares_to_issue = if *pool_principle == Uint256::zero() {
                amount
            } else {
                amount / *pool_principle * *pool_shares
            };
        },
    }

    *account_shares = *account_shares + shares_to_issue;

    *account_principle = *account_principle + amount_to_issue;

    *pool_shares = *pool_shares + shares_to_issue;
    *pool_principle = *pool_principle + amount_to_issue;

}

pub struct RemoveFromPoolResponse {
    pub shares_removed: Uint256,
    pub amount_removed: Uint256
}

pub fn remove_from_pool(
    quantity: Quantity,
    info: &AssetInfo,
    pool_principles:    &mut Vec<AssetAmount>,
    pool_shares:        &mut Vec<AssetAmount>,
    account_principles: &mut Vec<AssetAmount>,
    account_shares:     &mut Vec<AssetAmount>,
) -> CommonResult<RemoveFromPoolResponse> {
    if let Some(pool_principle)    = get_amount_mut(pool_principles, info)
    && let Some(pool_shares)       = get_amount_mut(pool_shares, info)
    && let Some(account_principle) = get_amount_mut(account_principles, &info)
    && let Some(account_shares)    = get_amount_mut(account_shares, &info) {

        let mut shares_to_remove;
        let mut amount_to_remove;
    
        match quantity {
            Quantity::Shares(shares) => {
                shares_to_remove = shares.min(*account_shares);
                let fraction_to_withdraw = Decimal256::from_ratio(shares_to_remove, *pool_shares);
                amount_to_remove = fraction_to_withdraw * *pool_principle;
            },
            Quantity::Amount(amount) => {
                amount_to_remove = amount;
                shares_to_remove = if *pool_principle == Uint256::zero() {
                    Uint256::zero()
                } else {
                    amount / *pool_principle * *pool_shares
                };
                if shares_to_remove > *account_shares {
                    shares_to_remove = *account_shares;
                    let fraction_to_withdraw = Decimal256::from_ratio(shares_to_remove, *pool_shares);
                    amount_to_remove = fraction_to_withdraw * *pool_principle;
                }
            },
        }

        *account_shares = *account_shares - shares_to_remove;
        *account_principle = get_difference_or_zero(*account_principle, amount_to_remove);

        *pool_shares = *pool_shares - shares_to_remove;
        *pool_principle = *pool_principle - amount_to_remove;

        // TODO: think about removing entry from vec if amount remaining is 0

        Ok(RemoveFromPoolResponse{
            shares_removed: shares_to_remove,
            amount_removed: amount_to_remove,
        })
    } else { Err(CommonError::InsufficientLiquidity {  }) }
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