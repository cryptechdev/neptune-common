use serde::{
    Serialize,
    de::DeserializeOwned
};
use cosmwasm_std::{
    Deps,
    Env, to_binary, 
    CosmosMsg, WasmMsg, Addr, 
};
use cosmwasm_std::{Uint256};
use terraswap::asset::AssetInfo;

// Neptune Package crate imports
use crate::{
    error::{NeptuneResult},
    base_config::{get_vault_contract, get_basset_token_contract, get_stable_asset},
    vault::ExecuteMsg as VaultExecuteMsg, execute_base::{SendFundsMsg, BaseExecuteMsg},
};

pub fn msg_to_self<ExecuteMsg: Serialize+DeserializeOwned>(
    env: &Env,
    msg: &ExecuteMsg
) -> NeptuneResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(
        WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&msg)?,
        }
    ))
}

pub fn send_stable<ExecuteMsg: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps,
    env: &Env,
    amount: Uint256,
    address: Addr
) -> NeptuneResult<CosmosMsg> {
    send_asset::<ExecuteMsg>(env, amount, &get_stable_asset(deps)?, address)
}

pub fn send_basset<ExecuteMsg: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps,
    env: &Env,
    amount: Uint256,
    address: Addr
) -> NeptuneResult<CosmosMsg> {
    msg_to_self(&env, &ExecuteMsg::from(BaseExecuteMsg::SendFunds{
        recipient: address,
        amount: amount,
        send_msg: SendFundsMsg::SendTokens(get_basset_token_contract(deps)?),
        exec_msg: None
    }))
}

pub fn send_asset<ExecuteMsg: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    env: &Env,
    amount: Uint256,
    asset: &AssetInfo,
    address: Addr
) -> NeptuneResult<CosmosMsg> {
    msg_to_self(&env, &ExecuteMsg::from(BaseExecuteMsg::SendFunds{
        recipient: address,
        amount: amount,
        send_msg: SendFundsMsg::from(asset.clone()),
        exec_msg: None
    }))
}

pub fn msg_to_vault(
    deps: Deps,
    msg: &VaultExecuteMsg
) -> NeptuneResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(
        WasmMsg::Execute {
            contract_addr: get_vault_contract(deps)?.to_string(),
            funds: vec![],
            msg: to_binary(&msg)?,
        }
    ))
}