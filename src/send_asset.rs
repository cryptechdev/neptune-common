use cosmwasm_std::{to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Env, Uint256, WasmMsg};
use cw20::Cw20ExecuteMsg;
// Neptune Package crate imports
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    asset::{AssetInfo, AssetMap},
    error::{CommonError, CommonResult},
    math::to_uint128,
};

pub type SendFundsMsg = AssetInfo;

pub fn transfer_funds(recipient: &Addr, mut funds: AssetMap<Uint256>) -> Result<Vec<CosmosMsg>, CommonError> {
    let mut msgs = vec![];
    // remove any elements that are zero
    funds.retain(|x| !x.1.is_zero());
    for (asset, amount) in funds {
        msgs.push(match asset {
            AssetInfo::NativeToken { denom } => {
                transfer_coins(vec![Coin { denom, amount: to_uint128(amount)? }], recipient)
            }
            AssetInfo::Token { contract_addr } => transfer_token(&contract_addr, amount, recipient)?,
        });
    }

    Ok(msgs)
}

pub fn send_funds(
    recipient: &Addr, amount: Uint256, send_msg: SendFundsMsg, exec_msg: Binary,
) -> Result<CosmosMsg, CommonError> {
    let msg = match send_msg {
        SendFundsMsg::NativeToken { denom } => {
            send_coins(vec![Coin { denom, amount: to_uint128(amount)? }], recipient, exec_msg)
        }
        SendFundsMsg::Token { contract_addr: token_addr } => send_token(&token_addr, amount, recipient, exec_msg)?,
    };

    Ok(msg)
}

fn transfer_coins(mut coins: Vec<Coin>, recipient_addr: &Addr) -> CosmosMsg {
    coins.retain(|x| !x.amount.is_zero());
    CosmosMsg::Bank(BankMsg::Send { to_address: recipient_addr.to_string(), amount: coins })
}

fn send_coins(coins: Vec<Coin>, recipient_addr: &Addr, exec_msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: recipient_addr.to_string(),
        msg:           exec_msg,
        funds:         coins,
    })
}

fn transfer_token(token_addr: &Addr, token_amount: Uint256, recipient_addr: &Addr) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        funds:         vec![],
        msg:           to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient_addr.to_string(),
            amount:    to_uint128(token_amount)?,
        })?,
    }))
}

fn send_token(
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
