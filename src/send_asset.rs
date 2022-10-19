use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, Env, Response,
    Uint256, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

use crate::{
    asset::AssetInfo,
    error::{CommonError, CommonResult},
    math::to_uint128,
    querier::{query_balance, query_token_balance},
    warn,
    warning::NeptuneWarning,
};
// Neptune Package crate imports
use neptune_authorization::authorization::{
    neptune_execute_authorize, NeptuneContractAuthorization,
};

/// The private messages for sending funds out of a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SendFundsMsg {
    SendCoins(String),
    SendTokens(Addr),
}

impl FromStr for SendFundsMsg {
    type Err = CommonError;

    /// TODO: Not rigorous, should only be used for commandline
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 10 || s.starts_with("ibc") {
            Ok(Self::SendCoins(s.to_string()))
        } else {
            Ok(Self::SendTokens(Addr::unchecked(s)))
        }
    }
}

impl From<AssetInfo> for SendFundsMsg {
    fn from(asset: AssetInfo) -> Self {
        match asset {
            AssetInfo::NativeToken { denom } => SendFundsMsg::SendCoins(denom),
            AssetInfo::Token { contract_addr: addr } => SendFundsMsg::SendTokens(Addr::unchecked(addr)),
        }
    }
}

impl Into<AssetInfo> for SendFundsMsg {
    fn into(self) -> AssetInfo {
        match self {
            SendFundsMsg::SendCoins(denom) => AssetInfo::NativeToken { denom },
            SendFundsMsg::SendTokens(addr) => AssetInfo::Token { contract_addr: addr.into() },
        }
    }
}

/// TODO: this should be refactored to not modify the send value, Should likely contain a fail and no_fail variant.
pub fn send_funds_tuple<A: NeptuneContractAuthorization<SendFundsMsg>>(
    deps: Deps,
    env: &Env,
    recipient: &Addr,
    mut amount: Uint256,
    send_msg: SendFundsMsg,
    exec_msg: Option<Binary>,
) -> Result<(CosmosMsg, Vec<Attribute>), CommonError> {
    neptune_execute_authorize::<SendFundsMsg, A>(deps, &env, &recipient, &send_msg)?;

    let mut attrs: Vec<Attribute> = vec![];

    let cosmos_msg = match send_msg {
        SendFundsMsg::SendCoins(denom) => {
            // Cap by our balance
            let coin_balance = query_balance(deps, &env.contract.address, denom.to_string())?;
            if amount > coin_balance {
                warn!(attrs, NeptuneWarning::InsuffBalance);
                amount = coin_balance;
            }
            if amount.is_zero() {
                warn!(attrs, NeptuneWarning::AmountWasZero);
            }

            // Create the Coin array and either send coins or attach to a message
            let coins = vec![Coin {
                denom:  denom.to_string(),
                amount: to_uint128(amount)?,
            }];
            match exec_msg {
                Some(binary) => attach_coins(coins, recipient, binary),
                None => send_coins(coins, recipient),
            }
        }
        SendFundsMsg::SendTokens(token_addr) => {
            // Cap by our balance
            let token_balance = query_token_balance(deps, &token_addr, &env.contract.address)?;
            if amount > token_balance {
                warn!(attrs, NeptuneWarning::InsuffBalance);
                amount = token_balance;
            }
            if amount.is_zero() {
                warn!(attrs, NeptuneWarning::AmountWasZero);
            }

            send_tokens(&token_addr, amount, exec_msg, recipient)?
        }
    };

    Ok((cosmos_msg, attrs))
}

pub fn send_funds<A: NeptuneContractAuthorization<SendFundsMsg>>(
    deps: Deps,
    env: &Env,
    recipient: &Addr,
    amount: Uint256,
    send_msg: SendFundsMsg,
    exec_msg: Option<Binary>,
) -> Result<Response, CommonError> {
    let tuple = send_funds_tuple::<A>(deps, env, recipient, amount, send_msg, exec_msg)?;

    Ok(Response::new()
        .add_message(tuple.0)
        .add_attributes(tuple.1))
}

pub fn attach_coins(coins: Vec<Coin>, recipient_addr: &Addr, exec_msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: recipient_addr.to_string(),
        msg:           exec_msg,
        funds:         coins,
    })
}

pub fn send_coins(coins: Vec<Coin>, recipient_addr: &Addr) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount:     coins,
    })
}

pub fn send_tokens(
    token_addr: &Addr,
    token_amount: Uint256,
    exec_msg: Option<Binary>,
    recipient_addr: &Addr,
) -> Result<CosmosMsg, CommonError> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        funds:         vec![],
        msg:           to_binary(&match exec_msg {
            Some(binary) => Cw20ExecuteMsg::Send {
                contract: recipient_addr.to_string(),
                amount:   to_uint128(token_amount)?,
                msg:      binary,
            },
            None => Cw20ExecuteMsg::Transfer {
                recipient: recipient_addr.to_string(),
                amount:    to_uint128(token_amount)?,
            },
        })?,
    }))
}

pub fn msg_to_self<ExecuteMsg: Serialize + DeserializeOwned>(
    env: &Env,
    msg: &ExecuteMsg,
) -> CommonResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds:         vec![],
        msg:           to_binary(&msg)?,
    }))
}
