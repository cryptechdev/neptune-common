use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, StdError, StdResult, Uint256};
use cw_storage_plus::{Bound, Bounder, Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{neptune_map::NeptuneMap, traits::KeyVec};

/// AssetInfo can represent either a native token or a token in cosmwasm.
#[cw_serde]
#[repr(u8)]
#[derive(Eq, PartialOrd, Ord)]
pub enum AssetInfo {
    NativeToken { denom: String } = 0,
    Token { contract_addr: Addr } = 1,
}

const NATIVE_TOKEN_DISCRIMINANT: u8 = 0;
const TOKEN_DISCRIMINANT: u8 = 1;

pub type AssetMap<T> = NeptuneMap<AssetInfo, T>;

impl AssetInfo {
    pub fn as_str(&self) -> &str {
        match self {
            AssetInfo::Token { contract_addr } => contract_addr.as_str(),
            AssetInfo::NativeToken { denom } => denom.as_str(),
        }
    }
}

impl From<Addr> for AssetInfo {
    fn from(contract_addr: Addr) -> Self {
        AssetInfo::Token { contract_addr }
    }
}

impl Display for AssetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json_wasm::to_string(self)
                .map_err(|_| core::fmt::Error)?
                .as_str(),
        )
    }
}

impl<'a> PrimaryKey<'a> for &'a AssetInfo {
    type Prefix = u8;
    type SubPrefix = ();
    type Suffix = String;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // The discriminate is added as a prefix.
        match self {
            AssetInfo::Token {
                contract_addr: addr,
            } => {
                vec![Key::Val8([TOKEN_DISCRIMINANT]), Key::Ref(addr.as_bytes())]
            }
            AssetInfo::NativeToken { denom } => {
                vec![
                    Key::Val8([NATIVE_TOKEN_DISCRIMINANT]),
                    Key::Ref(denom.as_bytes()),
                ]
            }
        }
    }
}

impl<'a> Prefixer<'a> for &'a AssetInfo {
    fn prefix(&self) -> Vec<Key> {
        match self {
            AssetInfo::Token {
                contract_addr: addr,
            } => {
                vec![Key::Val8([TOKEN_DISCRIMINANT]), Key::Ref(addr.as_bytes())]
            }
            AssetInfo::NativeToken { denom } => {
                vec![
                    Key::Val8([NATIVE_TOKEN_DISCRIMINANT]),
                    Key::Ref(denom.as_bytes()),
                ]
            }
        }
    }
}

impl<'a> Bounder<'a> for &'a AssetInfo {
    fn inclusive_bound(self) -> Option<Bound<'a, Self>> {
        Some(Bound::inclusive(self))
    }

    fn exclusive_bound(self) -> Option<Bound<'a, Self>> {
        Some(Bound::exclusive(self))
    }
}

impl<'a> KeyDeserialize for &'a AssetInfo {
    type Output = AssetInfo;

    const KEY_ELEMS: u16 = 2;

    /// See: https://github.com/xd009642/tarpaulin/issues/1192    
    #[cfg(not(tarpaulin_include))]
    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        // The discriminate is the first byte after the length prefix.
        // Split off after the 3rd.
        let split = value.split_off(3);

        // Pop off the last byte (3rd) in value which is the discriminate.
        match value.pop().unwrap() {
            TOKEN_DISCRIMINANT => Ok(AssetInfo::Token {
                contract_addr: Addr::from_vec(split)?,
            }),
            NATIVE_TOKEN_DISCRIMINANT => Ok(AssetInfo::NativeToken {
                denom: String::from_vec(split)?,
            }),
            _ => Err(StdError::GenericErr {
                msg: "Failed deserializing.".into(),
            }),
        }
    }
}

impl KeyVec<Self> for AssetInfo {
    fn key_vec(&self) -> Vec<Self> {
        vec![self.clone()]
    }
}

#[cw_serde]
pub struct AssetAmount {
    pub info: AssetInfo,
    pub amount: Uint256,
}

impl From<AssetAmount> for (AssetInfo, Uint256) {
    fn from(val: AssetAmount) -> Self {
        (val.info, val.amount)
    }
}

impl From<Coin> for AssetAmount {
    fn from(coin: Coin) -> Self {
        Self {
            info: AssetInfo::NativeToken { denom: coin.denom },
            amount: coin.amount.into(),
        }
    }
}

impl TryInto<Coin> for AssetAmount {
    type Error = StdError;

    fn try_into(self) -> Result<Coin, Self::Error> {
        match self.info {
            AssetInfo::Token { .. } => Err(StdError::GenericErr {
                msg: "Cannot convert to AssetAmount".into(),
            }),
            AssetInfo::NativeToken { denom } => Ok(Coin {
                denom,
                amount: self.amount.try_into().unwrap(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Uint128};

    use super::*;
    use crate::storage::paginate;

    #[test]
    fn test_key_serialize_deserialzie() {
        let mut owned_deps = mock_dependencies();
        let deps = owned_deps.as_mut();
        pub const ASSETS: cw_storage_plus::Map<&AssetInfo, String> =
            cw_storage_plus::Map::new("assets");

        let token_1 = AssetInfo::Token {
            contract_addr: Addr::unchecked("my_address1"),
        };
        let token_2 = AssetInfo::Token {
            contract_addr: Addr::unchecked("my_address2"),
        };
        let native_token_1 = AssetInfo::NativeToken {
            denom: "utest1".into(),
        };
        let native_token_2 = AssetInfo::NativeToken {
            denom: "utest2".into(),
        };

        // Add the assets out of order.
        ASSETS
            .save(deps.storage, &token_1, &"token_1".into())
            .unwrap();
        ASSETS
            .save(deps.storage, &token_2, &"token_2".into())
            .unwrap();
        ASSETS
            .save(deps.storage, &native_token_1, &"native_token_1".into())
            .unwrap();
        ASSETS
            .save(deps.storage, &native_token_2, &"native_token_2".into())
            .unwrap();

        assert_eq!(
            ASSETS.load(deps.storage, &native_token_1).unwrap(),
            "native_token_1"
        );
        assert_eq!(
            ASSETS.load(deps.storage, &native_token_2).unwrap(),
            "native_token_2"
        );
        assert_eq!(ASSETS.load(deps.storage, &token_1).unwrap(), "token_1");
        assert_eq!(ASSETS.load(deps.storage, &token_2).unwrap(), "token_2");

        let list = paginate(deps.storage, None, None, ASSETS).unwrap();
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted);
        assert_eq!(list.len(), 4);

        // Native tokens have a discriminate of 0 so are sorted first.
        assert_eq!(list[0].0, native_token_1);
        assert_eq!(list[1].0, native_token_2);
        assert_eq!(list[2].0, token_1);
        assert_eq!(list[3].0, token_2);

        // Test the bounder and prefixer impl.
        let list = paginate(deps.storage, Some(&native_token_1), Some(2), ASSETS).unwrap();
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted);
        assert_eq!(list.len(), 2);
        assert_eq!(
            list,
            vec![
                (native_token_2, "native_token_2".to_string()),
                (token_1, "token_1".to_string())
            ]
            .into()
        )
    }

    #[test]
    fn test_as_str() {
        let string = "test".to_string();
        let native = AssetInfo::NativeToken {
            denom: string.clone(),
        };
        let token = AssetInfo::Token {
            contract_addr: Addr::unchecked(string.clone()),
        };
        assert_eq!(string.as_str(), native.as_str());
        assert_eq!(string.as_str(), token.as_str());
    }

    #[test]
    fn test_coin_conversion() {
        let coin = Coin {
            denom: "test".to_string(),
            amount: Uint128::from(0u64),
        };
        let asset_amount: AssetAmount = coin.clone().into();
        let res: Coin = asset_amount.try_into().unwrap();
        assert_eq!(coin, res);

        let asset_amount = AssetAmount {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("test"),
            },
            amount: 0u64.into(),
        };
        let res: Result<Coin, _> = asset_amount.clone().try_into();
        assert!(res.is_err());

        let tuple: (AssetInfo, Uint256) = asset_amount.into();
        assert_eq!(
            tuple,
            (
                AssetInfo::Token {
                    contract_addr: Addr::unchecked("test".to_string())
                },
                0u64.into()
            )
        )
    }
}
