use cosmwasm_std::{
    to_binary, Addr, BalanceResponse, BankQuery, CustomQuery, QuerierWrapper, QueryRequest,
    Uint256, WasmQuery,
};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

use crate::{asset::AssetInfo, error::NeptuneError};

// Query the balance of a coin for a specific account.
pub fn query_coin_balance(
    querier: QuerierWrapper<impl CustomQuery>,
    account_addr: &Addr,
    denom: String,
) -> Result<Uint256, NeptuneError> {
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(Uint256::from(balance.amount.amount))
}

/// Queries the balance of a cw20 token for a specific account.
pub fn query_token_balance(
    querier: QuerierWrapper<impl CustomQuery>,
    account_addr: &Addr,
    token_addr: &Addr,
) -> Result<Uint256, NeptuneError> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        })?,
    }))?;
    Ok(res.balance.into())
}

/// Queries the supply of a cw20 token.
pub fn query_supply(
    querier: QuerierWrapper<impl CustomQuery>,
    contract_addr: &Addr,
) -> Result<Uint256, NeptuneError> {
    let token_info: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(token_info.total_supply.into())
}

/// Queries the balance of an asset for a specific account.
pub fn query_asset_balance(
    querier: QuerierWrapper<impl CustomQuery>,
    account: &Addr,
    asset: &AssetInfo,
) -> Result<Uint256, NeptuneError> {
    match asset {
        AssetInfo::NativeToken { denom } => {
            Ok(query_coin_balance(querier, account, denom.clone())?)
        }
        AssetInfo::Token { contract_addr } => {
            Ok(query_token_balance(querier, account, contract_addr)?)
        }
    }
}
