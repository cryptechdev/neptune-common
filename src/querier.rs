// Cosmos and Terra imports
use cosmwasm_std::{
    Deps, Addr, BankQuery, BalanceResponse, QueryRequest, WasmQuery, to_binary,
};
use cosmwasm_std::{Uint256};
use cw20::{Cw20QueryMsg, TokenInfoResponse, BalanceResponse as Cw20BalanceResponse,};
use terraswap::asset::AssetInfo;

use crate::error::CommonError;

pub fn query_balance(deps: Deps, account_addr: &Addr, denom: String) -> Result<Uint256, CommonError> {
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
) -> Result<Uint256, CommonError> {
    let res: Cw20BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        })?,
    }))?;

    // load balance form the token contract
    Ok(res.balance.into())
}

pub fn query_supply(deps: Deps, contract_addr: &Addr) -> Result<Uint256, CommonError> {
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
) -> Result<Uint256, CommonError> {
    match asset {
        AssetInfo::NativeToken{ denom } => { Ok(query_balance(deps, &account, denom.clone())?) }
        AssetInfo::Token{ contract_addr } => { Ok(query_token_balance(deps, &Addr::unchecked(contract_addr), &account)?) }
    }
}