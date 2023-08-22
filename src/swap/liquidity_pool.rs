use crate::{
    asset::{AssetAmount, AssetInfo},
    error::NeptuneResult,
    msg_wrapper::MsgWrapper,
    query_wrapper::QueryWrapper,
    send_asset::{send_assets, SendFundsMsg},
};
use astroport::pair::{ReverseSimulationResponse, SimulationResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, Deps, Env, QuerierWrapper, QueryRequest, StdResult,
    Uint128, Uint256, WasmQuery,
};

use super::Swap;

#[cw_serde]
pub struct LiquidityPool {
    pub addr: Addr,
}

impl Swap for LiquidityPool {
    fn swap(
        &self,
        deps: Deps<QueryWrapper>,
        _env: &Env,
        offer_asset: &AssetInfo,
        _ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        let return_amount =
            query_sim_pool(deps, self.addr.clone(), offer_asset.clone(), offer_amount)?;
        if return_amount == Uint256::zero() {
            return Ok(vec![]);
        }
        msg_to_dex(self.addr.clone(), offer_asset.clone(), offer_amount)
    }

    /// sends a query for a swap simulation
    fn query_sim(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        _ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        if offer_amount.is_zero() {
            return Ok(Uint256::zero());
        }
        Ok(simulate(
            &deps.querier,
            self.addr.clone(),
            &AssetAmount {
                info: offer_asset.clone(),
                amount: offer_amount,
            }
            .try_into()?,
        )?
        .return_amount
        .into())
    }

    fn query_reverse_sim(
        &self,
        deps: Deps<QueryWrapper>,
        _offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        if ask_amount.is_zero() {
            return Ok(Uint256::zero());
        }
        let offer_amount = reverse_simulate(
            &deps.querier,
            self.addr.clone(),
            &AssetAmount {
                info: ask_asset.clone(),
                amount: ask_amount + Uint256::one(),
            }
            .try_into()?,
        )?
        .offer_amount
            + Uint128::one(); // We always add 1 here to avoid rounding errors
        Ok(offer_amount.into())
    }
}

fn simulate(
    querier: &QuerierWrapper<QueryWrapper>,
    pool_addr: Addr,
    offer_asset: &astroport::asset::Asset,
) -> StdResult<SimulationResponse> {
    querier.query_wasm_smart(
        pool_addr,
        &astroport::pair::QueryMsg::Simulation {
            offer_asset: offer_asset.clone(),
        },
    )
}

fn reverse_simulate(
    querier: &QuerierWrapper<QueryWrapper>,
    pool_addr: Addr,
    ask_asset: &astroport::asset::Asset,
) -> StdResult<ReverseSimulationResponse> {
    querier.query_wasm_smart(
        pool_addr,
        &astroport::pair::QueryMsg::ReverseSimulation {
            ask_asset: ask_asset.clone(),
        },
    )
}

/// Sends a swap message to the given pool.
fn msg_to_dex(
    swap_pool: Addr,
    offer_asset: SendFundsMsg,
    offer_amount: Uint256,
) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
    let swap_msg = to_binary(&astroport::pair::ExecuteMsg::Swap {
        offer_asset: AssetAmount {
            info: offer_asset.clone(),
            amount: offer_amount,
        }
        .try_into()?,
        belief_price: None,
        max_spread: Some(Decimal::percent(50)),
        to: None,
    })?;
    let msg = send_assets(&swap_pool, offer_amount, offer_asset, swap_msg)?;
    Ok(vec![msg])
}

/// queries a pool and simulates a swap.
fn query_sim_pool(
    deps: Deps<QueryWrapper>,
    pool_addr: Addr,
    offer_asset: AssetInfo,
    offer_amount: Uint256,
) -> NeptuneResult<Uint256> {
    if offer_amount.is_zero() {
        return Ok(Uint256::zero());
    }

    let res: SimulationResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_addr.to_string(),
        msg: to_binary(&astroport::pair::QueryMsg::Simulation {
            offer_asset: astroport::asset::Asset {
                info: offer_asset.into(),
                amount: offer_amount.try_into()?,
            },
        })?,
    }))?;

    Ok(res.return_amount.into())
}
