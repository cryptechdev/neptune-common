use cosmwasm_std::{to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Uint256, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::{
    asset::{AssetInfo, AssetMap},
    error::CommonError,
    traits::Zeroed,
};

pub type SendFundsMsg = AssetInfo;

/// Transfers both tokens and native tokens to the recipient.
/// If the amount is zero, it is not included in the returned messages.
pub fn transfer_assets(
    recipient: &Addr,
    mut assets: AssetMap<Uint256>,
) -> Result<Vec<CosmosMsg>, CommonError> {
    let mut msgs = vec![];
    // remove any elements that are zero
    assets.remove_zeroed();
    for (asset, amount) in assets {
        msgs.push(match asset {
            AssetInfo::NativeToken { denom } => transfer_coins(
                vec![Coin {
                    denom,
                    amount: amount.try_into()?,
                }],
                recipient,
            ),
            AssetInfo::Token { contract_addr } => {
                transfer_token(&contract_addr, amount, recipient)?
            }
        });
    }

    Ok(msgs)
}

/// Sends both tokens and native tokens to the recipient along with an attached message.
/// If the amount is zero the message is still sent.
pub fn send_assets(
    recipient: &Addr,
    amount: Uint256,
    send_msg: SendFundsMsg,
    exec_msg: Binary,
) -> Result<CosmosMsg, CommonError> {
    let msg = match send_msg {
        SendFundsMsg::NativeToken { denom } => send_coins(
            vec![Coin {
                denom,
                amount: amount.try_into()?,
            }],
            recipient,
            exec_msg,
        ),
        SendFundsMsg::Token {
            contract_addr: token_addr,
        } => send_token(&token_addr, amount, recipient, exec_msg)?,
    };

    Ok(msg)
}

/// Transfers native tokens to the recipient.
/// Does not check if the amount is zero.
fn transfer_coins(coins: Vec<Coin>, recipient_addr: &Addr) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: coins,
    })
}

/// Sends native tokens to the recipient along with a message.
/// Does not check if the amount is zero.
fn send_coins(coins: Vec<Coin>, recipient_addr: &Addr, exec_msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: recipient_addr.to_string(),
        msg: exec_msg,
        funds: coins,
    })
}

/// Transfers tokens to the recipient.
/// Does not check if the amount is zero.
fn transfer_token(
    token_addr: &Addr,
    token_amount: Uint256,
    recipient_addr: &Addr,
) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient_addr.to_string(),
            amount: token_amount.try_into()?,
        })?,
    }))
}

/// Sends tokens to the recipient along with a message.
/// Does not check if the amount is zero.
fn send_token(
    token_addr: &Addr,
    token_amount: Uint256,
    recipient_addr: &Addr,
    exec_msg: Binary,
) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: recipient_addr.to_string(),
            amount: token_amount.try_into()?,
            msg: exec_msg,
        })?,
        funds: vec![],
    }))
}
