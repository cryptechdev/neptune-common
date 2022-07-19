use cosmwasm_std::{
    Deps, QueryRequest, WasmQuery, to_binary, Addr
};
use cosmwasm_std::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

// Anchor package imports
use moneymarket::{
    custody::QueryMsg::Borrower as AnchorQueryBorrower,
    custody::BorrowerResponse as AnchorBorrowerResponse,
    market::QueryMsg::BorrowerInfo as AnchorQueryBorrowerInfo,
    market::BorrowerInfoResponse as AnchorBorrowerInfoResponse,
    interest_model::{
        QueryMsg as InterestQueryMsg,
        BorrowRateResponse
    },
    overseer::WhitelistResponse,
};
use terraswap::asset::AssetInfo;

// Neptune Package crate imports
use crate::{
    base_config::{
        get_anchor_overseer_contract,
        //get_anchor_oracle_contract,
        get_basset_token_contract,
        get_anchor_market_contract,
        get_anchor_interest_model_contract, 
        get_anchor_custody_contract, get_stable_asset
    },
    error::{NeptuneResult, NeptuneError},
    querier::{query_asset_balance},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AnchorOverseerEpochState {
    pub deposit_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
    pub prev_interest_buffer: Uint256,
    pub last_executed_height: u64,
}

pub fn query_anchor_borrower(
    deps: Deps,
    address: Addr
) -> NeptuneResult<AnchorBorrowerResponse> {
    Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: get_anchor_custody_contract(deps)?.to_string(),
        msg: to_binary(&AnchorQueryBorrower {
            address: address.to_string(),
        })?,
    }))?)
}

pub fn query_anchor_borrower_info(
    deps: Deps,
    address: Addr
) -> NeptuneResult<AnchorBorrowerInfoResponse> {
    Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: get_anchor_market_contract(deps)?.to_string(),
        msg: to_binary(&AnchorQueryBorrowerInfo {
            borrower: address.to_string(),
            block_height: None,
        })?,
    }))?)
}

pub fn query_anchor_borrow_rate(deps: Deps) -> NeptuneResult<Decimal256> {
    
    let interest_model = get_anchor_interest_model_contract(deps)?;

    let anchor_state = query_anchor_market_state(deps)?;
    let anchor_balance = query_anchor_stable_balance(deps)?;

    Ok(deps.querier.query::<BorrowRateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: interest_model.to_string(),
        msg: to_binary(&InterestQueryMsg::BorrowRate {
            market_balance: anchor_balance,
            total_liabilities: anchor_state.total_liabilities,
            total_reserves: anchor_state.total_reserves,
        })?,
    }))?.rate)
}

pub fn query_anchor_deposit_rate(deps: Deps) -> NeptuneResult<Decimal256> {
    Ok(deps.querier.query::<AnchorOverseerEpochState>(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: get_anchor_overseer_contract(deps)?.to_string(),
            msg: to_binary(&moneymarket::overseer::QueryMsg::EpochState {})?
        })
    )?.deposit_rate)
}

pub fn query_anchor_max_ltv(deps: Deps) -> NeptuneResult<Decimal256> {
    
    let anchor_response : WhitelistResponse = deps.querier.query(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: get_anchor_overseer_contract(deps)?.to_string(),
            msg: to_binary(&moneymarket::overseer::QueryMsg::Whitelist {
                collateral_token: Some(get_basset_token_contract(deps)?.to_string()),
                start_after: None, limit: None
            })?
        })
    )?;

    let anchor_response_elem = anchor_response.elems.get(0).unwrap();

    Ok(anchor_response_elem.max_ltv)
}

pub fn query_anchor_basset_price(deps: Deps) -> NeptuneResult<Decimal256> {

    match get_stable_asset(deps)? {
        AssetInfo::Token { .. } => return Err(NeptuneError::Unimplemented {  }),
        AssetInfo::NativeToken { .. } => return Err(NeptuneError::Unimplemented {  }),
    }
}

pub fn query_anchor_market_state(deps: Deps) -> NeptuneResult<moneymarket::market::StateResponse> {
    
    Ok(deps.querier.query(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: get_anchor_market_contract(deps)?.to_string(),
            msg: to_binary(&moneymarket::market::QueryMsg::State {
                block_height: None
            })?
        })
    )?)
}

pub fn query_anchor_stable_balance(deps: Deps) -> NeptuneResult<Uint256> {
        query_asset_balance(deps, &get_anchor_market_contract(deps)?, &get_stable_asset(deps)?)
}