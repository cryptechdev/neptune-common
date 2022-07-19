// Cosmos and Terra imports
use cosmwasm_std::{Uint256, Decimal256, CosmosMsg};
use cosmwasm_std::{attr, Deps, DepsMut, Env, Response, to_binary};
use serde::{Serialize, de::DeserializeOwned};



// Anchor imports
use moneymarket::market::{
    ExecuteMsg::DepositStable as AnchorExecuteDepositStable,
    Cw20HookMsg::RedeemStable as AnchorExecuteRedeemStable,
    StateResponse as AnchorStateResponse,
};

use crate::math::UINT256_ONE;
// Neptune package imports
use crate::{
    anchor::{
        query_anchor_market_state,
        query_anchor_stable_balance, query_anchor_deposit_rate,
    },
    base_config::{
        get_anchor_market_contract,
        get_anchor_aust_contract, get_stable_asset
    },
    common::{
        msg_to_self,
    },
    error::{NeptuneResult},
    execute_base::{BaseExecuteMsg, SendFundsMsg},
    querier::{
        query_token_balance, query_asset_balance, 
    },
    warning::NeptuneWarning, warn,
};

pub const BLOCKS_PER_YEAR : Decimal256 = Decimal256::raw(4656810u128);

pub fn deposit_in_earn<ExecuteMsg: From<BaseExecuteMsg> + Serialize + DeserializeOwned>(
    deps: DepsMut,
    env: &Env,
    mut amount: Uint256,
) -> NeptuneResult<Response> {

    let stable_balance =
        query_asset_balance(
            deps.as_ref(),
            &env.contract.address,
            &get_stable_asset(deps.as_ref())?
        )?;

    let mut attrs = vec![];
    if amount > stable_balance {
        warn!(attrs, NeptuneWarning::InsuffBalance);
        amount = stable_balance;
    }

    let mut msgs : Vec<CosmosMsg> = vec![];
    // TODO: find out the correct value for this threshold
    if amount < Uint256::from(5u64) { return warn!(NeptuneWarning::AmountBelowThreshold); }
    else {
        msgs.push(
            msg_to_self(&env, &ExecuteMsg::from(BaseExecuteMsg::SendFunds{
                recipient: get_anchor_market_contract(deps.as_ref())?,
                amount: amount,
                send_msg: get_stable_asset(deps.as_ref())?.into(),
                exec_msg: Some(to_binary(&AnchorExecuteDepositStable {} )?)
            }))?
        );
    }
    attrs = vec![
        attr("neptune_action", "deposit_in_earn"),
        attr("amount", amount),
    ];
    Ok(Response::new().add_messages(msgs).add_attributes(attrs))
}

pub fn withdraw_from_earn<ExecuteMsg: From<BaseExecuteMsg> + Serialize + DeserializeOwned>(
    deps: DepsMut,
    env: &Env,
    mut amount: Uint256,
) -> NeptuneResult<Response> {

    let redeemable_stable = query_earn_redeemable(deps.as_ref(), env)?;

    let mut msgs = vec![];
    let mut attrs = vec![];
    if amount > redeemable_stable {
        warn!(attrs, NeptuneWarning::InsuffBalance);
        amount = redeemable_stable;
    }

    let total_aust = query_aust_amount(deps.as_ref(), env)?;
    let aust_to_redeem = total_aust.multiply_ratio(amount, redeemable_stable);

    if aust_to_redeem.is_zero() { return warn!(NeptuneWarning::AmountWasZero); }
    else {
        msgs.push(
            msg_to_self(&env, &ExecuteMsg::from(BaseExecuteMsg::SendFunds{
                recipient: get_anchor_market_contract(deps.as_ref())?,
                amount: aust_to_redeem,
                send_msg: SendFundsMsg::SendTokens(get_anchor_aust_contract(deps.as_ref())?),
                exec_msg: Some(to_binary(&AnchorExecuteRedeemStable {} )?)
            }))?
        );
    }
    attrs = vec![
        attr("neptune_action", "withdraw_from_earn"),
        attr("amount", amount),
        attr("aust_redeemed", aust_to_redeem),
    ];
    Ok(Response::new().add_messages(msgs).add_attributes(attrs))
}

/// Query the value of the investment as measured in stable
pub fn query_earn_value(deps: Deps, env: &Env) -> NeptuneResult<Uint256> {

    let anchor_state: AnchorStateResponse = query_anchor_market_state(deps)?;

    Ok( query_aust_amount(deps,env)? * anchor_state.prev_exchange_rate )
}

/// Gets the value of the investment that can actually be withdrawn as measured in stable
pub fn query_earn_redeemable(deps: Deps, env: &Env) -> NeptuneResult<Uint256> {

    let anchor_state: AnchorStateResponse = query_anchor_market_state(deps)?;
    let anchor_balance = query_anchor_stable_balance(deps)?;
    let anchor_redeemable_stable = anchor_balance - anchor_state.total_reserves * UINT256_ONE;
    let stable_balance = query_aust_amount(deps,env)? * anchor_state.prev_exchange_rate;

    Ok( std::cmp::min(anchor_redeemable_stable, stable_balance) )
}

pub fn query_aust_amount(deps: Deps, env: &Env) -> NeptuneResult<Uint256> {
    Ok(query_token_balance(
        deps,
        &get_anchor_aust_contract(deps)?,
        &env.contract.address
    )?)
}

pub fn query_earn_apy(deps: Deps) -> NeptuneResult<Decimal256>
{
    Ok(BLOCKS_PER_YEAR * query_anchor_deposit_rate(deps)?)
}