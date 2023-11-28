pub mod error;
pub mod liquidity_pool;
#[cfg(feature = "injective")]
pub mod order_book;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Decimal256, Deps, Env, Uint256};

use crate::{
    asset::AssetInfo, error::NeptuneResult, msg_wrapper::MsgWrapper, query_wrapper::QueryWrapper,
};

use self::{error::SwapError, liquidity_pool::LiquidityPool};

pub const EXCHANGES: cw_storage_plus::Map<(&AssetInfo, &AssetInfo), Exchange> =
    cw_storage_plus::Map::new("exchanges");

#[cw_serde]
pub enum Exchange {
    LiquidityPool(LiquidityPool),
    #[cfg(feature = "injective")]
    OrderBook(order_book::OrderBook),
}

fn get_exchange_type(
    deps: Deps<QueryWrapper>,
    exchanges: &cw_storage_plus::Map<(&AssetInfo, &AssetInfo), Exchange>,
    mut assets: [&AssetInfo; 2],
) -> NeptuneResult<Exchange> {
    assets.sort_unstable();
    Ok(exchanges
        .may_load(deps.storage, (assets[0], assets[1]))?
        .ok_or_else(|| SwapError::PoolNotFound([assets[0].clone(), assets[1].clone()]))?)
}

pub trait Swap {
    /// Creates a message to swap assets
    fn swap(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>>;

    /// Creates a message to swap assets
    fn swap_ask(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        let offer_amount = self.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)?;
        self.swap(deps, env, offer_asset, ask_asset, offer_amount)
    }

    fn query_sim(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Uint256>;

    fn query_reverse_sim(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Uint256>;

    /// Returns the volume of the largest swap where the ratio
    /// of offer to ask is less than ratio.
    fn query_ask_amount_at_price(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        max_ratio: Decimal256,
    ) -> NeptuneResult<Uint256>;

    /// Uses a swap simulation to calculate the ratio of offer to ask.
    fn query_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let ask_amount = self.query_sim(deps, offer_asset, ask_asset, offer_amount)?;
        Ok(Decimal256::checked_from_ratio(offer_amount, ask_amount)?)
    }

    fn query_reverse_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let offer_amount = self.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)?;
        Ok(Decimal256::checked_from_ratio(offer_amount, ask_amount)?)
    }
}

impl Swap for cw_storage_plus::Map<'static, (&AssetInfo, &AssetInfo), Exchange> {
    fn swap(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.swap(deps, env, offer_asset, ask_asset, offer_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.swap(deps, env, offer_asset, ask_asset, offer_amount)
            }
        }
    }

    fn swap_ask(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.swap_ask(deps, env, offer_asset, ask_asset, ask_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.swap_ask(deps, env, offer_asset, ask_asset, ask_amount)
            }
        }
    }

    fn query_sim(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.query_sim(deps, offer_asset, ask_asset, offer_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.query_sim(deps, offer_asset, ask_asset, offer_amount)
            }
        }
    }

    fn query_reverse_sim(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)
            }
        }
    }

    fn query_ask_amount_at_price(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        max_ratio: Decimal256,
    ) -> NeptuneResult<Uint256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.query_ask_amount_at_price(deps, offer_asset, ask_asset, max_ratio)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.query_ask_amount_at_price(deps, offer_asset, ask_asset, max_ratio)
            }
        }
    }

    fn query_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.query_swap_ratio(deps, offer_asset, ask_asset, offer_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.query_swap_ratio(deps, offer_asset, ask_asset, offer_amount)
            }
        }
    }

    fn query_reverse_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => {
                liquidity_pool.query_reverse_swap_ratio(deps, offer_asset, ask_asset, ask_amount)
            }
            #[cfg(feature = "injective")]
            Exchange::OrderBook(order_book) => {
                order_book.query_reverse_swap_ratio(deps, offer_asset, ask_asset, ask_amount)
            }
        }
    }
}
