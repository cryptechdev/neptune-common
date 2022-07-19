/// This crate should only be imported by Investment contracts that aim to specialize the code

// Cosmos and Terra imports
use cosmwasm_std::Uint256;
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, to_binary, WasmMsg};
use cw20::Cw20ReceiveMsg;

use terraswap::asset::AssetInfo;

// Package crate imports
use crate::{
    authorization::{
		authorize_permissions,
    	BasePermissionGroups::*
	},
    base_config::{get_vault_contract, get_stable_asset},
    error::{NeptuneError, NeptuneResult},
    investment::{
        InvestmentBaseExecuteMsg,
        ExecuteMsg,
    },
    querier::{query_asset_balance}, warning::NeptuneWarning, warn, common::send_stable, 
};

/// Handle a call to invest a set amount of stable attached to this message
pub fn base_execute_invest(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    _cw20_receive_msg: Option<Cw20ReceiveMsg>,
) -> NeptuneResult<Response> {

    authorize_permissions(deps.as_ref(), env, &info.sender, &vec![&Admins, &Vault])?;

    let stable_received: Uint256 = match get_stable_asset(deps.as_ref())? {
        AssetInfo::NativeToken{denom} => {
            match info.funds.iter().find(|x| x.denom == denom) {
                Some(coin) => {
                    if coin.amount.is_zero() {
                        return Err(NeptuneError::NoFundsReceived {});
                    } else {
                        coin.amount.into()
                    }
                }
                None => {
                    return Err(NeptuneError::NoFundsReceived {});
                }
            }
        },
        AssetInfo::Token { .. } => { return Err(NeptuneError::Unimplemented {  }) }
    };



    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::InvestmentBase(InvestmentBaseExecuteMsg::SendFundsToInvestment { amount: stable_received } ))?,
                funds: vec![]
            }),
        ])
        .add_attributes(vec![
            attr("neptune_action", "invest"),
            attr("sender", info.sender.as_str()),
            attr("amount", stable_received),
        ])
    )
}

pub fn base_execute_divest(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    amount: Uint256,
) -> NeptuneResult<Response> {

    authorize_permissions(deps.as_ref(), env, &info.sender, &vec![&Admins, &Vault])?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::InvestmentBase(InvestmentBaseExecuteMsg::WithdrawFundsFromInvestment { amount } ))?,
                funds: vec![]
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::InvestmentBase(InvestmentBaseExecuteMsg::SendFundsToVaultForDivestment { amount } ))?,
                funds: vec![]
            }),
        ])
        .add_attributes(vec![
            attr("neptune_action", "divest"),
            attr("sender", info.sender.as_str()),
            attr("amount", amount),
        ])
    )
}

/// Send the minimum of the amount requested, and the amount available for sending in the extension.
pub fn base_execute_send_funds_to_vault_for_divestment(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    ask_amount: Uint256,
) -> NeptuneResult<Response> {

    authorize_permissions(deps.as_ref(), env, &info.sender, &vec![&Admins, &Internal])?;

    let stable_balance = query_asset_balance(deps.as_ref(), &env.contract.address, &get_stable_asset(deps.as_ref())?)?;

    let mut msgs = vec![];
    let mut attrs = vec![];
    let mut amount_to_send = ask_amount;

    if stable_balance > ask_amount {
        warn!(attrs, NeptuneWarning::Generic(format!("Excess stable leftover in investment: {}uusd", stable_balance - ask_amount).as_str()))
    }

    if stable_balance < ask_amount {
        warn!(attrs, NeptuneWarning::InsuffBalance);
        amount_to_send = stable_balance;
    }

    if amount_to_send.is_zero()  {
        warn!(attrs, NeptuneWarning::AmountWasZero);
    } else {
        msgs.push(send_stable::<ExecuteMsg>(deps.as_ref(), &env, amount_to_send, get_vault_contract(deps.as_ref())?)?)
    }

    attrs.extend(vec![
        attr("neptune_action", "divest"),
        attr("sender", info.sender.as_str()),
        attr("ask_amount", ask_amount),
        attr("amount_to_send", amount_to_send),
    ]);

    Ok(Response::new().add_messages(msgs).add_attributes(attrs))
}