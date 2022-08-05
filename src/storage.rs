use cosmwasm_std::{Addr, CanonicalAddr, Deps, StdResult, StdError, Order, Storage};
use cosmwasm_storage::Singleton;
use cw_storage_plus::{Bound, PrimaryKey, KeyDeserialize, Map};
use serde::{de::DeserializeOwned, Serialize};

// Neptune Package crate imports
use crate::error::CommonError;

/// ================ ///
/// Helper functions ///
/// ================ ///

pub const BASE_OWNER_KEY: &str = "owner";
pub const BASE_CONFIG_KEY: &str = "base_config";
pub const CONFIG_KEY: &str = "config";
pub const PARAMS_KEY: &str = "params";
pub const STATE_KEY: &str = "state";

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_map<
    'a,
    K: Into<Bound<'a, K>> + PrimaryKey<'a> + KeyDeserialize<Output = K> + 'static,
    V: Serialize + DeserializeOwned,
>(
    deps: Deps,
    start_after: Option<K>,
    limit: Option<u32>,
    map: Map<'a, K, V>,
) -> Result<Vec<(K, V)>, CommonError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|key| key.into());

    Ok(map
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .into_iter()
        .collect::<Result<Vec<(K,V)>,StdError>>()?)
}

pub fn get_contract_addr(
    deps: Deps,
    contract_name: &str,
    contract_address: &Option<CanonicalAddr>,
) -> Result<Addr, CommonError> {
    Ok(deps.api.addr_humanize(
        &contract_address
            .clone()
            .ok_or(CommonError::MissingAddress(contract_name.to_string()))?,
    )?)
}

pub fn get_config_string(var: Option<String>) -> Result<String, CommonError> {
    Ok(var.ok_or(CommonError::MissingConfigVariable {})?)
}

pub fn canonicalize_address(deps: Deps, address: &String) -> StdResult<Option<CanonicalAddr>> {
    if address.is_empty() {
        Ok(None)
    } else {
        Ok(Some(deps.api.addr_canonicalize(address.as_str())?))
    }
}

pub fn canonicalize_addresses(
    deps: Deps,
    addresses: &Vec<String>,
) -> StdResult<Vec<CanonicalAddr>> {
    addresses
        .iter()
        .map(|x| deps.api.addr_canonicalize(x.as_str()))
        .collect()
}

pub fn humanize_addresses(deps: Deps, addresses: &Vec<CanonicalAddr>) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|x| deps.api.addr_humanize(x))
        .collect()
}
