// Cosmos and Terra imports
use cosmwasm_std::{
    Addr, 
    Deps,
    CosmosMsg,
    to_binary, Env, Decimal,
};
use cosmwasm_std::{Uint256};
use serde::{Serialize, de::DeserializeOwned};

use terraswap::asset::{ AssetInfo };

// Neptune Package crate imports
use crate::{
    base_config::{
        get_asset_denom,
        get_stable_asset_pool, 
        get_asset_basset_pool,
        get_basset_token_contract, get_stable_basset_pool, get_anc_pool, get_anc_token_contract, get_stable_asset,
    },
    common::{
        msg_to_self
    },
    error::{
        NeptuneResult, NeptuneError, 
    },
    execute_base::{BaseExecuteMsg, SendFundsMsg},
    math::to_uint128
};

pub fn msg_to_terraswap<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps,
    env: &Env,
    swap_pool: Addr,
    offer_asset: SendFundsMsg,
    offer_amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {

    let mut msgs = vec![];

    if offer_amount.is_zero(){ return Ok(msgs); }

    let receive_amount = query_lp_coin_simulation(deps, &swap_pool, offer_asset.clone().into(), offer_amount)?;

    if receive_amount.is_zero(){ return Ok(msgs); }

    msgs.push(msg_to_self(env, &E::from(BaseExecuteMsg::SendFunds{
        recipient: swap_pool,
        amount: offer_amount,
        send_msg: offer_asset.clone(),
        exec_msg: Some(to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: terraswap::asset::Asset {
                info: offer_asset.into(),
                amount: to_uint128(offer_amount)?,
            },
            belief_price: Option::None,
            max_spread: Some(Decimal::percent(50)),
            to: Option::None,
        })?)
    }))?);

    Ok(msgs)
}

fn swap_stable_to_asset<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    msg_to_terraswap::<E>(
        deps,
        env,
        get_stable_asset_pool(deps)?,
        get_stable_asset(deps)?.into(),
        amount
    )
}

fn swap_asset_to_stable<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    msg_to_terraswap::<E>(
        deps,
        env,
        get_stable_asset_pool(deps)?,
        SendFundsMsg::SendCoins(get_asset_denom(deps)?),
        amount
    )
}

fn swap_asset_to_basset<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    msg_to_terraswap::<E>(
        deps,
        env,
        get_asset_basset_pool(deps)?,
        SendFundsMsg::SendCoins(get_asset_denom(deps)?),
        amount
    )
}

fn swap_basset_to_asset<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    msg_to_terraswap::<E>(
        deps,
        env,
        get_asset_basset_pool(deps)?,
        SendFundsMsg::SendTokens(get_basset_token_contract(deps)?),
        amount
    )
}

pub fn swap_anc_to_stable<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    msg_to_terraswap::<E>(
        deps,
        env,
        get_anc_pool(deps)?,
        SendFundsMsg::SendTokens(get_anc_token_contract(deps)?),
        amount
    )
}

pub fn swap_stable_to_basset<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {
    
    let mut msgs = vec![];
    match swap_stable_to_asset::<E>(deps,env,amount) {
        Ok( msg ) => { 
            msgs.extend(msg);
            let asset_returned = query_sim_stable_to_asset(deps, amount)?;
            msgs.extend(swap_asset_to_basset::<E>(deps, env, asset_returned)?);
        },
        Err( NeptuneError::MissingAddress(..) ) => {
            msgs.extend(
                msg_to_terraswap::<E>(
                    deps,
                    env,
                    get_stable_basset_pool(deps)?,
                    get_stable_asset(deps)?.into(),
                    amount
                )?
            );
        },
        Err( .. ) => {}
    }
    Ok(msgs)
}

pub fn swap_basset_to_stable<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
    deps: Deps, env: &Env, amount: Uint256
) -> NeptuneResult<Vec<CosmosMsg>> {

    let mut msgs = vec![];
    if !amount.is_zero() {
        match swap_basset_to_asset::<E>(deps,env,amount) {
            Ok( msg ) => { 
                msgs.extend(msg);
                let asset_returned = query_sim_basset_to_asset(deps, amount)?;
                msgs.extend(swap_asset_to_stable::<E>(deps, env, asset_returned)?);
            },
            Err( NeptuneError::MissingAddress(..) ) => {
                msgs.extend(
                    msg_to_terraswap::<E>(
                        deps,
                        env,
                        get_stable_basset_pool(deps)?,
                        SendFundsMsg::SendTokens(get_basset_token_contract(deps)?),
                        amount
                    )?
                );
            },
            Err( .. ) => {}
        }
    }
    Ok(msgs)
}

pub fn query_sim_stable_to_basset(
    deps: Deps,   
    offer_amount: Uint256
) -> NeptuneResult<Uint256> {
    if offer_amount.is_zero() { return Ok(Uint256::zero()) }

    let basset_returned;
    match query_sim_stable_to_asset(deps, offer_amount) {
        Ok( asset_returned ) => {
            basset_returned = query_sim_asset_to_basset(deps, asset_returned)?;
        },
        Err( NeptuneError::MissingAddress(..) ) => {
            basset_returned = query_lp_coin_simulation(
                deps,
                &get_stable_basset_pool(deps)?,
                get_stable_asset(deps)?,
                offer_amount
            )?;
        },
        Err( .. ) => { return Ok(Uint256::zero()) }
    }
    Ok(basset_returned)
}

pub fn query_sim_basset_to_stable(
    deps: Deps,
    offer_amount: Uint256
) -> NeptuneResult<Uint256> {
    if offer_amount.is_zero() { return Ok(Uint256::zero()) }

    let stable_returned;
    match query_sim_basset_to_asset(deps, offer_amount) {
        Ok( asset_returned ) => {
            stable_returned = query_sim_asset_to_stable(deps, asset_returned)?;
        },
        Err ( NeptuneError::MissingAddress(..) ) => {
            stable_returned = query_lp_token_simulation(
                deps, 
                &get_stable_basset_pool(deps)?, 
                &get_basset_token_contract(deps)?, 
                offer_amount
            )?;
        },
        Err( .. ) => { return Ok(Uint256::zero()) }
    }
    Ok(stable_returned)
}

pub fn query_reverse_sim_stable_to_basset(
    deps: Deps,
    ask_amount: Uint256
) -> NeptuneResult<Uint256> {
    if ask_amount.is_zero() { return Ok(Uint256::zero()) }
    let stable_needed;
    match query_reverse_sim_asset_to_basset(deps, ask_amount) {
        Ok( asset_needed ) => {
            stable_needed = query_reverse_sim_stable_to_asset(deps, asset_needed)?;
        },
        Err( NeptuneError::MissingAddress(..) ) => {
            stable_needed = query_reverse_token_sim(
                deps, get_stable_basset_pool(deps)?, get_basset_token_contract(deps)?, ask_amount
            )?;
        },
        Err(e) => { return Err(e) }
    }
    Ok(stable_needed)
}

pub fn query_reverse_sim_basset_to_stable(
    deps: Deps,
    ask_amount: Uint256
) -> NeptuneResult<Uint256> {
    if ask_amount.is_zero() { return Ok(Uint256::zero()) }

    let basset_needed;
    match query_reverse_sim_asset_to_stable(deps, ask_amount) {
        Ok( asset_needed ) => {
            basset_needed = query_reverse_sim_basset_to_asset(deps, asset_needed)?;
        },
        Err( NeptuneError::MissingAddress(..) ) => {
            basset_needed = query_reverse_coin_sim(
                deps, get_stable_basset_pool(deps)?, get_stable_asset(deps)?, ask_amount
            )?;
        },
        Err(e) => { return Err(e) }
    }
    Ok(basset_needed.into())
}

// Forward simulations
fn query_sim_stable_to_asset(deps: Deps, offer_amount: Uint256) -> NeptuneResult<Uint256>{
    query_lp_coin_simulation( deps, &get_stable_asset_pool(deps)?, get_stable_asset(deps)?, offer_amount )
}

fn query_sim_asset_to_stable(deps: Deps, offer_amount: Uint256) -> NeptuneResult<Uint256>{
    query_lp_coin_simulation( deps, &get_stable_asset_pool(deps)?, AssetInfo::NativeToken{denom: get_asset_denom(deps)?}, offer_amount )
}

fn query_sim_asset_to_basset(deps: Deps, offer_amount: Uint256) -> NeptuneResult<Uint256>{
    query_lp_coin_simulation( deps, &get_asset_basset_pool(deps)?, AssetInfo::NativeToken{denom: get_asset_denom(deps)?}, offer_amount )
}

fn query_sim_basset_to_asset(deps: Deps, offer_amount: Uint256) -> NeptuneResult<Uint256>{
    query_lp_token_simulation( deps, &get_asset_basset_pool(deps)?, &get_basset_token_contract(deps)?, offer_amount )
}

pub fn query_sim_anc_to_stable(deps: Deps, offer_amount: Uint256) -> NeptuneResult<Uint256>{
    query_lp_token_simulation( deps, &get_anc_pool(deps)?, &get_anc_token_contract(deps)?, offer_amount )
}

// Reverse simulations
fn query_reverse_sim_stable_to_asset(deps: Deps, ask_amount: Uint256) -> NeptuneResult<Uint256>{
    query_reverse_coin_sim(deps, get_stable_asset_pool(deps)?, AssetInfo::NativeToken{denom: get_asset_denom(deps)?}, ask_amount)
}

fn query_reverse_sim_asset_to_stable(deps: Deps, ask_amount: Uint256) -> NeptuneResult<Uint256>{
    query_reverse_coin_sim(deps, get_stable_asset_pool(deps)?, get_stable_asset(deps)?, ask_amount)
}

fn query_reverse_sim_asset_to_basset(deps: Deps, ask_amount: Uint256) -> NeptuneResult<Uint256>{
    query_reverse_token_sim(deps, get_asset_basset_pool(deps)?, get_basset_token_contract(deps)?, ask_amount)
}

fn query_reverse_sim_basset_to_asset(deps: Deps, ask_amount: Uint256) -> NeptuneResult<Uint256>{
     query_reverse_coin_sim(deps, get_asset_basset_pool(deps)?, AssetInfo::NativeToken{denom: get_asset_denom(deps)?}, ask_amount)
}

pub fn query_lp_token_simulation(
    deps: Deps,
    pool_addr: &Addr,
    token_addr: &Addr,
    amount: Uint256
) -> NeptuneResult<Uint256> {

    if amount.is_zero() { return Ok(Uint256::zero()) }

    Ok(terraswap::querier::simulate(
        &deps.querier,
        pool_addr.clone(),
        &terraswap::asset::Asset {
            info: terraswap::asset::AssetInfo::Token {
                contract_addr: token_addr.to_string()
            },
            amount: to_uint128(amount)?,
        }
    )?.return_amount.into())
}

pub fn query_lp_coin_simulation(
    deps: Deps,
    pool_addr: &Addr,
    offer_asset: AssetInfo,
    amount: Uint256
) -> NeptuneResult<Uint256> {

    if amount.is_zero() { return Ok(Uint256::zero()) }

    Ok(terraswap::querier::simulate(
        &deps.querier,
        pool_addr.clone(),
        &terraswap::asset::Asset {
            info: offer_asset,
            amount: to_uint128(amount)?,
        }
    )?.return_amount.into())
}

pub fn query_reverse_token_sim(
    deps: Deps,
    pool_addr: Addr,
    token_addr: Addr,
    ask_amount: Uint256
) -> NeptuneResult<Uint256> {

    if ask_amount.is_zero() { return Ok(Uint256::zero()) }

    Ok(match terraswap::querier::reverse_simulate(
        &deps.querier,
        pool_addr.clone(),
        &terraswap::asset::Asset {
            info:  AssetInfo::Token {
                contract_addr: token_addr.to_string(),
            },
            amount: to_uint128(ask_amount)?,
        }
    ) {
        Ok(response) => response.offer_amount.into(),
        Err(_) => {
            let token_price = query_lp_token_simulation(
                deps, &pool_addr, &token_addr, Uint256::from(1000000u128)
            )?;
            if token_price.is_zero() { return Err(NeptuneError::ZeroDenominator {})}
            // include a 1% extra to account for slippage and protocol fees (1000000/990099 = ~1.01)
            ask_amount.multiply_ratio(token_price,Uint256::from(990099u128))
        },
    })
}

pub fn query_reverse_coin_sim(
    deps: Deps,
    pool_addr: Addr,
    ask_asset: AssetInfo,
    ask_amount: Uint256
) -> NeptuneResult<Uint256> {

    if ask_amount.is_zero() { return Ok(Uint256::zero()) }

    Ok(match terraswap::querier::reverse_simulate(
        &deps.querier,
        pool_addr.clone(),
        &terraswap::asset::Asset {
            info:  ask_asset.clone(),
            amount: to_uint128(ask_amount)?,
        }
    ) {
        Ok(response) => response.offer_amount.into(),
        Err(_) => {
            let coin_price = query_lp_coin_simulation(
                deps, &pool_addr, ask_asset, Uint256::from(1000000u128)
            )?;
            if coin_price.is_zero() { return Err(NeptuneError::ZeroDenominator {})}
            // include a 1% extra to account for slippage and protocol fees (1000000/990099 = ~1.01)
            ask_amount.multiply_ratio(coin_price,Uint256::from(990099u128))
        },
    })
}