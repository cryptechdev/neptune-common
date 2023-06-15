use astroport::pair::SimulationResponse;
use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, Deps, Uint256, CustomQuery, QueryRequest, WasmQuery};

use crate::{
    asset::{AssetAmount, AssetInfo},
    error::CommonResult,
    send_asset::{send_assets, SendFundsMsg},
};

/// Sends a swap message to the given pool.
pub fn msg_to_dex(swap_pool: Addr, offer_asset: SendFundsMsg, offer_amount: Uint256) -> CommonResult<Vec<CosmosMsg>> {
    let swap_msg = to_binary(&astroport::pair::ExecuteMsg::Swap {
        offer_asset: AssetAmount { info: offer_asset.clone(), amount: offer_amount }.try_into()?,
        belief_price: None,
        max_spread: Some(Decimal::percent(50)),
        to: None,
    })?;
    let msg = send_assets(&swap_pool, offer_amount, offer_asset, swap_msg)?;
    Ok(vec![msg])
}

/// queries a pool and simulates a swap.
pub fn query_sim_pool(
    deps: Deps<impl CustomQuery>, pool_addr: Addr, offer_asset: AssetInfo, offer_amount: Uint256,
) -> CommonResult<Uint256> {
    if offer_amount.is_zero() {
        return Ok(Uint256::zero());
    }

    let res: SimulationResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_addr.to_string(),
        msg: to_binary(&astroport::pair::QueryMsg::Simulation {
            offer_asset: astroport::asset::Asset {
                info: offer_asset.into(),
                amount: offer_amount.try_into()?,
            }
        })?,
    }))?;

    Ok(res.return_amount.into())
}
