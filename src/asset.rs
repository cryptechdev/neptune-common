use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, StdError, StdResult, Uint256};
use cw_storage_plus::{Bound, Bounder, Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{neptune_map::NeptuneMap, traits::KeyVec};

/// AssetInfo can represent either a native token or a token in cosmwasm.
#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

const NATIVE_TOKEN_DISCRIMINANT: u8 = 0;
const TOKEN_DISCRIMINANT: u8 = 1;

pub type AssetMap<T> = NeptuneMap<AssetInfo, T>;

impl Display for AssetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token { contract_addr } => contract_addr.fmt(f),
            Self::NativeToken { denom } => denom.fmt(f),
        }
    }
}

impl<'a> PrimaryKey<'a> for &'a AssetInfo {
    type Prefix = String;
    type SubPrefix = ();
    type Suffix = u8;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // The descriminate is added as a prefix.
        match self {
            AssetInfo::Token { contract_addr: addr } => {
                vec![Key::Val8([TOKEN_DISCRIMINANT]), Key::Ref(addr.as_bytes())]
            }
            AssetInfo::NativeToken { denom } => {
                vec![Key::Val8([NATIVE_TOKEN_DISCRIMINANT]), Key::Ref(denom.as_bytes())]
            }
        }
    }
}

impl<'a> Prefixer<'a> for &'a AssetInfo {
    fn prefix(&self) -> Vec<Key> {
        match self {
            AssetInfo::Token { contract_addr: addr } => {
                vec![Key::Val8([TOKEN_DISCRIMINANT]), Key::Ref(addr.as_bytes())]
            }
            AssetInfo::NativeToken { denom } => {
                vec![Key::Val8([NATIVE_TOKEN_DISCRIMINANT]), Key::Ref(denom.as_bytes())]
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
        // The descriminate is the first byte after the prefix.
        // Split off after the 3rd.
        let split = value.split_off(3);

        // Pop off the last byte (3rd) in value which is the discriminate.
        match value.pop().unwrap() {
            TOKEN_DISCRIMINANT => Ok(AssetInfo::Token { contract_addr: Addr::from_vec(split)? }),
            NATIVE_TOKEN_DISCRIMINANT => Ok(AssetInfo::NativeToken { denom: String::from_vec(split)? }),
            _ => Err(StdError::GenericErr { msg: "Failed deserializing.".into() }),
        }
    }
}

impl KeyVec<Self> for AssetInfo {
    fn key_vec(&self) -> Vec<Self> { vec![self.clone()] }
}

#[cw_serde]
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

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_dependencies;

    use super::*;
    use crate::storage::read_map;

    #[test]
    fn test_key_serialize_deserialzie() {
        let mut owned_deps = mock_dependencies();
        let deps = owned_deps.as_mut();
        pub const ASSETS: cw_storage_plus::Map<&AssetInfo, String> = cw_storage_plus::Map::new("assets");

        let native_token_1 = AssetInfo::NativeToken { denom: "utest1".into() };
        let native_token_2 = AssetInfo::NativeToken { denom: "utest2".into() };
        let token_1 = AssetInfo::Token { contract_addr: Addr::unchecked("my_address1") };
        let token_2 = AssetInfo::Token { contract_addr: Addr::unchecked("my_address2") };

        ASSETS.save(deps.storage, &token_1, &"token_1".into()).unwrap();
        ASSETS.save(deps.storage, &token_2, &"token_2".into()).unwrap();
        ASSETS.save(deps.storage, &native_token_1, &"native_token_1".into()).unwrap();
        ASSETS.save(deps.storage, &native_token_2, &"native_token_2".into()).unwrap();

        assert_eq!(ASSETS.load(deps.storage, &native_token_1).unwrap(), "native_token_1");
        assert_eq!(ASSETS.load(deps.storage, &native_token_2).unwrap(), "native_token_2");
        assert_eq!(ASSETS.load(deps.storage, &token_1).unwrap(), "token_1");
        assert_eq!(ASSETS.load(deps.storage, &token_2).unwrap(), "token_2");

        let list = read_map(deps.as_ref(), None, None, ASSETS).unwrap();
        assert_eq!(list.len(), 4);
        // native tokens have a discriminate of 0 so are sorted first
        assert_eq!(list[0].0, native_token_1);
        assert_eq!(list[1].0, native_token_2);
        assert_eq!(list[2].0, token_1);
        assert_eq!(list[3].0, token_2);
    }
}
