use cosmwasm_std::{to_binary, Addr, BalanceResponse, BankQuery, Deps, QueryRequest, Uint256, WasmQuery, CustomQuery};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

use crate::{asset::AssetInfo, error::CommonError};

// Query the balance of a coin for a specific account.
pub fn query_coin_balance(deps: Deps<impl CustomQuery>, account_addr: &Addr, denom: String) -> Result<Uint256, CommonError> {
    let balance: BalanceResponse = deps
        .querier
        .query(&QueryRequest::Bank(BankQuery::Balance { address: account_addr.to_string(), denom }))?;
    Ok(Uint256::from(balance.amount.amount))
}

/// Queries the balance of a cw20 token for a specific account.
pub fn query_token_balance(deps: Deps<impl CustomQuery>, token_addr: &Addr, account_addr: &Addr) -> Result<Uint256, CommonError> {
    let res: Cw20BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance { address: account_addr.to_string() })?,
    }))?;
    Ok(res.balance.into())
}

/// Queries the supply of a cw20 token.
pub fn query_supply(deps: Deps<impl CustomQuery>, contract_addr: &Addr) -> Result<Uint256, CommonError> {
    let token_info: TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(token_info.total_supply.into())
}

/// Queries the balance of an asset for a specific account.
pub fn query_asset_balance(deps: Deps<impl CustomQuery>, account: &Addr, asset: &AssetInfo) -> Result<Uint256, CommonError> {
    match asset {
        AssetInfo::NativeToken { denom } => Ok(query_coin_balance(deps, account, denom.clone())?),
        AssetInfo::Token { contract_addr } => Ok(query_token_balance(deps, contract_addr, account)?),
    }
}
