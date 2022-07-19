// Cosmos and Terra imports
use cosmwasm_std::{Deps, QueryRequest, WasmQuery, to_binary, Fraction};
use cosmwasm_std::{Decimal256, Uint256};

// Neptune Package crate imports
use crate::{
    anchor::{
        query_anchor_borrower, 
        query_anchor_borrower_info,
    },
    base_config::{
        get_vault_contract, 
        get_basset_token_contract, 
        //get_basset_rewards_contract, 
        get_stable_asset
    },
    error::{NeptuneError, NeptuneResult},
    querier::{
        query_basset_price, 
        query_investment_value,
        query_token_balance, query_investment_pending_rewards_value, query_asset_balance,
    }, 
    terraswap::{
        query_sim_anc_to_stable
    }, 
    vault::{Balances, State, QueryMsg, BalanceValues, TvlResponse}, 
    banker::PendingRewardsValue, 
    math::UINT256_ONE
};



pub fn query_balances(deps: Deps) -> Result<Balances, NeptuneError>
{
    Ok(Balances {
        collateral_basset:  query_collateral_amount(deps)?,
        debt_stable:           query_loan_value(deps)?,
        investment_stable:     query_investment_value(deps)?,
        liquid_stable:         query_stable_amount(deps)?,
        liquid_basset:      query_basset_amount(deps)?,
    })
}

pub fn query_balance_values(deps: Deps) -> Result<BalanceValues, NeptuneError>
{
    query_balances(deps)?.get_balance_values(query_basset_price(deps, false)?)
}

pub fn query_tvl(deps: Deps, include_unclaimed_rewards: bool) -> Result<TvlResponse, NeptuneError>
{
    // Get terraswap's bAsset price
    let basset_price = query_basset_price(deps, false)?;

    // Calculate total net worth
    let balance_amounts = query_balances(deps)?;
    let mut tvl_stable = balance_amounts.get_total_net_worth_as_stable(basset_price)?;
    let mut tvl_basset = balance_amounts.get_total_net_worth_as_basset(basset_price)?;

    if include_unclaimed_rewards {
        let pending_rewards_stable = query_pending_rewards_value(deps)?.total_pending_rewards_value;
        tvl_stable += pending_rewards_stable;
        tvl_basset += pending_rewards_stable * basset_price.inv().ok_or(NeptuneError::BassetPriceIsZero {})?;
    }

    Ok(TvlResponse{
        tvl_basset,
        tvl_stable,
        basset_price,
    })
}

pub fn query_ltv_ratio(deps: Deps, anchor_pricing: bool) -> Result<Decimal256, NeptuneError>
{
    let loan_value = query_loan_value(deps)?;
    let collateral_value = query_collateral_value(deps, anchor_pricing)?;
    let ratio = if collateral_value > Uint256::zero() {
        Decimal256::from_ratio(loan_value, collateral_value)
    } else {
        Decimal256::zero()
    };

    Ok(ratio)
}

pub fn query_stable_amount(deps: Deps) -> Result<Uint256, NeptuneError> {
    Ok(query_asset_balance(
        deps,
        &get_vault_contract(deps)?,
        &get_stable_asset(deps)?
    )?.into())
}

pub fn query_basset_amount(deps: Deps) -> Result<Uint256, NeptuneError> {
    Ok(query_token_balance(
        deps,
        &get_basset_token_contract(deps)?,
        &get_vault_contract(deps)?,
    )?.into())
}

/// Send a query to Anchor to determine the value of the collateral we have locked in Anchor
pub fn query_collateral_amount(deps: Deps) -> Result<Uint256, NeptuneError> {
    Ok(query_anchor_borrower(deps, get_vault_contract(deps)?)?.balance.into())
}

/// Calculates the value in stable of the collateral locked in Anchor
pub fn query_collateral_value(deps: Deps, anchor_price: bool) -> Result<Uint256, NeptuneError> {
    let collateral_amount = query_collateral_amount(deps)?;
    let basset_price = query_basset_price(deps,anchor_price)?;

    Ok(collateral_amount * basset_price)
}

pub fn query_loan_value(deps: Deps) -> Result<Uint256, NeptuneError> {
    Ok(query_anchor_borrower_info(deps, get_vault_contract(deps)?)?.loan_amount)
}

pub fn query_pending_rewards_value(deps: Deps) -> Result<PendingRewardsValue, NeptuneError> {

    let anc_pending_rewards_value = query_pending_anc_rewards_value(deps)?;
    let staking_pending_rewards_value = query_pending_basset_rewards_value(deps)?;
    let investment_pending_rewards_value = query_investment_pending_rewards_value(deps)?;
    let total_pending_rewards_value = anc_pending_rewards_value + staking_pending_rewards_value + investment_pending_rewards_value;

    Ok(PendingRewardsValue {
        total_pending_rewards_value,
        anc_pending_rewards_value,
        investment_pending_rewards_value,
        staking_pending_rewards_value,
    })
}

pub fn query_pending_anc_rewards_amount(deps: Deps) -> Result<Uint256, NeptuneError> {
    Ok(query_anchor_borrower_info(deps, get_vault_contract(deps)?)?.pending_rewards * UINT256_ONE)
}

pub fn query_pending_anc_rewards_value(deps: Deps) -> Result<Uint256, NeptuneError> {
    let amount = query_pending_anc_rewards_amount(deps)?;
    let value = query_sim_anc_to_stable(deps, amount)?;
    Ok(value)
}

pub fn query_pending_basset_rewards_value(_deps: Deps) -> Result<Uint256, NeptuneError> {
/*
    if let Ok(basset_rewards_addr) = get_basset_rewards_contract(deps) {
        let res: AccruedRewardsResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: basset_rewards_addr.to_string(),
            msg: to_binary(&AccruedRewards {
                address: get_vault_contract(deps)?.to_string(),
            })?,
        }))?;
    
        Ok(res.rewards.into())
    }
    else { return Ok(Uint256::zero()) }*/
    Err(NeptuneError::Unimplemented {  })
}

pub fn query_vault_state(deps: Deps) -> NeptuneResult<State> {
    let vault_state = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: get_vault_contract(deps)?.to_string(),
        msg: to_binary(&QueryMsg::GetState { })?,
    }))?;
    Ok(vault_state)
}