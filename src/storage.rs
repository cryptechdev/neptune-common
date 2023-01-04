use cosmwasm_std::{Addr, CanonicalAddr, Deps, Order, StdError, StdResult};
use cw_storage_plus::{Bounder, KeyDeserialize, Map, PrimaryKey};
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

pub fn read_map<
    'a,
    K: Bounder<'a> + PrimaryKey<'a> + KeyDeserialize<Output = K> + 'static,
    V: Serialize + DeserializeOwned,
>(
    deps: Deps, start_after: Option<K>, limit: Option<u32>, map: Map<'a, K, V>,
) -> Result<Vec<(K, V)>, CommonError> {
    let start = start_after.map(|key| key.inclusive_bound().unwrap());
    let vec = match limit {
        Some(limit) => map
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit as usize)
            .collect::<Result<Vec<(K, V)>, StdError>>()?,
        None => map.range(deps.storage, start, None, Order::Ascending).collect::<Result<Vec<(K, V)>, StdError>>()?,
    };
    Ok(vec)
}

pub fn get_contract_addr(
    deps: Deps, contract_name: &str, contract_address: &Option<CanonicalAddr>,
) -> Result<Addr, CommonError> {
    Ok(deps.api.addr_humanize(
        &contract_address.clone().ok_or_else(|| CommonError::MissingAddress(contract_name.to_string()))?,
    )?)
}

pub fn get_config_string(var: Option<String>) -> Result<String, CommonError> {
    var.ok_or(CommonError::MissingConfigVariable {})
}

pub fn canonicalize_address(deps: Deps, address: &String) -> StdResult<Option<CanonicalAddr>> {
    if address.is_empty() {
        Ok(None)
    } else {
        Ok(Some(deps.api.addr_canonicalize(address.as_str())?))
    }
}

pub fn canonicalize_addresses(deps: Deps, addresses: &[String]) -> StdResult<Vec<CanonicalAddr>> {
    addresses.iter().map(|x| deps.api.addr_canonicalize(x.as_str())).collect()
}

pub fn humanize_addresses(deps: Deps, addresses: &[CanonicalAddr]) -> StdResult<Vec<Addr>> {
    addresses.iter().map(|x| deps.api.addr_humanize(x)).collect()
}
