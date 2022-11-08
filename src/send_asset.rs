use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Env, Response, Uint256, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
// Neptune Package crate imports
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    asset::AssetInfo,
    asset_map::AssetMap,
    error::{CommonError, CommonResult},
    math::to_uint128,
    warn,
    warning::NeptuneWarning,
};

pub type SendFundsMsg = AssetInfo;

pub fn transfer_funds_tuple(
    recipient: &Addr, funds: AssetMap<Uint256>,
) -> Result<(Vec<CosmosMsg>, Vec<Attribute>), CommonError> {
    let mut msgs = vec![];
    let mut attrs = vec![];
    for (asset, amount) in funds {
        if amount.is_zero() {
            warn!(attrs, NeptuneWarning::AmountWasZero);
        }
        msgs.push(match asset {
            AssetInfo::NativeToken { denom } => {
                transfer_coins(vec![Coin { denom, amount: to_uint128(amount)? }], recipient)
            }
            AssetInfo::Token { contract_addr } => transfer_tokens(&contract_addr, amount, recipient)?,
        });
    }

    Ok((msgs, attrs))
}

pub fn transfer_funds(recipient: &Addr, funds: AssetMap<Uint256>) -> Result<Response, CommonError> {
    let (msgs, attrs) = transfer_funds_tuple(recipient, funds)?;
    Ok(Response::new().add_messages(msgs).add_attributes(attrs))
}

pub fn send_funds_tuple(
    recipient: &Addr, amount: Uint256, send_msg: SendFundsMsg, exec_msg: Binary,
) -> Result<(CosmosMsg, Vec<Attribute>), CommonError> {
    let mut attrs: Vec<Attribute> = vec![];
    if amount.is_zero() {
        warn!(attrs, NeptuneWarning::AmountWasZero);
    }

    let cosmos_msg = match send_msg {
        SendFundsMsg::NativeToken { denom } => {
            send_coins(vec![Coin { denom, amount: to_uint128(amount)? }], recipient, exec_msg)
        }
        SendFundsMsg::Token { contract_addr: token_addr } => send_tokens(&token_addr, amount, recipient, exec_msg)?,
    };

    Ok((cosmos_msg, attrs))
}

pub fn send_funds(
    recipient: &Addr, amount: Uint256, send_msg: SendFundsMsg, exec_msg: Binary,
) -> Result<Response, CommonError> {
    let tuple = send_funds_tuple(recipient, amount, send_msg, exec_msg)?;

    Ok(Response::new().add_message(tuple.0).add_attributes(tuple.1))
}

fn transfer_coins(coins: Vec<Coin>, recipient_addr: &Addr) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send { to_address: recipient_addr.to_string(), amount: coins })
}

fn send_coins(coins: Vec<Coin>, recipient_addr: &Addr, exec_msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: recipient_addr.to_string(),
        msg:           exec_msg,
        funds:         coins,
    })
}

fn transfer_tokens(token_addr: &Addr, token_amount: Uint256, recipient_addr: &Addr) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        funds:         vec![],
        msg:           to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient_addr.to_string(),
            amount:    to_uint128(token_amount)?,
        })?,
    }))
}

fn send_tokens(
    token_addr: &Addr, token_amount: Uint256, recipient_addr: &Addr, exec_msg: Binary,
) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg:           to_binary(&Cw20ExecuteMsg::Send {
            contract: recipient_addr.to_string(),
            amount:   to_uint128(token_amount)?,
            msg:      exec_msg,
        })?,
        funds:         vec![],
    }))
}

pub fn msg_to_self<ExecuteMsg: Serialize + DeserializeOwned>(env: &Env, msg: &ExecuteMsg) -> CommonResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds:         vec![],
        msg:           to_binary(&msg)?,
    }))
}
