use cosmwasm_std::{
    Deps, Addr, StdResult,
    CanonicalAddr,
};

// Neptune Package crate imports
use crate::{
    error::{CommonError},
};

/// ================ ///
/// Helper functions ///
/// ================ ///

pub const BASE_OWNER_KEY: &str = "owner";
pub const BASE_CONFIG_KEY: &str = "base_config";
pub const CONFIG_KEY: &str = "config";
pub const PARAMS_KEY: &str = "params";
pub const STATE_KEY: &str = "state";

pub fn get_contract_addr(
    deps: Deps,
    contract_name: &str,
    contract_address: &Option<CanonicalAddr>
) -> Result<Addr, CommonError> {

    Ok(deps.api.addr_humanize(&contract_address.clone().ok_or(
        CommonError::MissingAddress(contract_name.to_string())
    )?)?)
}

pub fn get_config_string(
    var: Option<String>
) -> Result<String, CommonError> {

    Ok(var.ok_or(
        CommonError::MissingConfigVariable {}
    )?)
}

pub fn canonicalize_address(deps: Deps, address: &String) -> StdResult<Option<CanonicalAddr>> {
    if address.is_empty() { Ok(None) }
    else { Ok(Some(deps.api.addr_canonicalize(address.as_str())?)) }
}

pub fn canonicalize_addresses(deps: Deps, addresses: &Vec<String>) -> StdResult<Vec<CanonicalAddr>> {
    addresses.iter().map(|x| deps.api.addr_canonicalize(x.as_str())).collect()
}

pub fn humanize_addresses(deps: Deps, addresses: &Vec<CanonicalAddr>) -> StdResult<Vec<Addr>> {
    addresses.iter().map(|x| deps.api.addr_humanize(x)).collect()
}