// Cosmos and Terra imports
use cosmwasm_std::{
    Deps, Addr, BankQuery, BalanceResponse, QueryRequest, WasmQuery, to_binary,
};
use cosmwasm_std::{Decimal256, Uint256};
use cw20::{Cw20QueryMsg, TokenInfoResponse, BalanceResponse as Cw20BalanceResponse,};
use serde::de::DeserializeOwned;
use terraswap::asset::AssetInfo;

// Neptune Package crate imports
use crate::{
    anchor::query_anchor_basset_price,
    terraswap::query_sim_basset_to_stable,
    error::{NeptuneResult, NeptuneError},
    base_config::get_vault_contract,
    vault::QueryMsg as VaultQueryMsg,
};

// Auxiliary function to send a query to the Neptune investment contract
pub fn query_vault<T: DeserializeOwned>(
    deps: Deps,
    msg: &VaultQueryMsg
) -> Result<T, NeptuneError> {
    Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: get_vault_contract(deps)?.to_string(),
        msg: to_binary(msg)?,
    }))?)
}

pub fn query_investment_value(deps: Deps) -> Result<Uint256, NeptuneError> {
    query_vault(deps, &VaultQueryMsg::GetInvestmentValue {})
}

pub fn query_investment_pending_rewards_value(deps: Deps) -> Result<Uint256, NeptuneError> {
    query_vault(deps, &VaultQueryMsg::GetInvestmentPendingRewardsValue {})
}

pub fn query_basset_price(
    deps: Deps,
    anchor_based: bool
) -> NeptuneResult<Decimal256> {

    if anchor_based {
        query_anchor_basset_price(deps)
    }
    else {
        let resulting_amount = query_sim_basset_to_stable(
            deps, Uint256::from(1000000u128)
        )?;
        Ok(Decimal256::from_ratio(Uint256::from(resulting_amount), Uint256::from(1_000_000u128)))
    }
}

pub fn query_balance(deps: Deps, account_addr: &Addr, denom: String) -> NeptuneResult<Uint256> {
    // load price form the oracle
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom: denom,
    }))?;
    Ok(Uint256::from(balance.amount.amount))
}

pub fn query_token_balance(
    deps: Deps,
    token_addr: &Addr,
    account_addr: &Addr,
) -> NeptuneResult<Uint256> {
    let res: Cw20BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        })?,
    }))?;

    // load balance form the token contract
    Ok(res.balance.into())
}

pub fn query_supply(deps: Deps, contract_addr: &Addr) -> NeptuneResult<Uint256> {
    // load price form the oracle
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(token_info.total_supply.into())
}

pub fn query_asset_balance(
    deps: Deps,
    account: &Addr,
    asset: &AssetInfo,
) -> Result<Uint256, NeptuneError> {
    match asset {
        AssetInfo::NativeToken{ denom } => { Ok(query_balance(deps, &account, denom.clone())?) }
        AssetInfo::Token{ contract_addr } => { Ok(query_token_balance(deps, &Addr::unchecked(contract_addr), &account)?) }
    }
}