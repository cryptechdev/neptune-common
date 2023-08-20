#[cfg(feature = "injective")]
pub mod order_book;
pub mod liquidity_pool;


use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, Uint256, CosmosMsg, Env, Decimal256};

use crate::{asset::AssetInfo, query_wrapper::QueryWrapper, error::{NeptuneResult, NeptuneError}, msg_wrapper::MsgWrapper};

use self::{liquidity_pool::LiquidityPool, order_book::OrderBook};

pub const EXCHANGES: cw_storage_plus::Map<(&AssetInfo, &AssetInfo), Exchange> =
    cw_storage_plus::Map::new("exchanges");

#[cw_serde]
pub enum Exchange {
    LiquidityPool(LiquidityPool),
    #[cfg(feature = "injective")]
    OrderBook(OrderBook)
}

fn get_exchange_type(
    deps: Deps<QueryWrapper>, 
    exchanges: &cw_storage_plus::Map<(&AssetInfo, &AssetInfo), Exchange>, 
    mut assets: [&AssetInfo; 2]
) -> NeptuneResult<Exchange> {
    assets.sort_unstable();
    exchanges
        .may_load(deps.storage, (assets[0], assets[1]))?
        .ok_or_else(|| NeptuneError::PoolNotFound([assets[0].clone(), assets[1].clone()]))
}

// pub fn get_exchange(
//     deps: Deps<QueryWrapper>, 
//     exchanges: &cw_storage_plus::Map<(&AssetInfo, &AssetInfo), Exchange>, 
//     hub_asset: AssetInfo, 
//     assets: Vec<AssetInfo>
// ) -> NeptuneMap<[AssetInfo; 2], Exchange> {
//     assets
//         .iter()
//         .filter_map(|x| {
//             if x != &hub_asset {
//                 let mut key = [hub_asset.clone(), x.clone()];
//                 key.sort_unstable();
//                 let addr = exchanges.load(deps.storage, (&key[0], &key[1])).unwrap();
//                 Some((key, addr))
//             } else {
//                 None
//             }
//         })
//         .collect::<NeptuneMap<_, _>>()
// }

pub trait Swap {
    /// Creates a message to swap assets
    fn swap(
        &self, deps: Deps<QueryWrapper>, env: &Env, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>>;

    /// Creates a message to swap assets
    fn swap_ask(
        &self, deps: Deps<QueryWrapper>, env: &Env, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        let offer_amount = self.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)?;
        self.swap(deps, env, offer_asset, ask_asset, offer_amount)
    }

    fn query_sim(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Uint256>;

    fn query_reverse_sim(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Uint256>;

    /// Uses a swap simulation to calculate the ratio of offer to ask.
    fn query_swap_ratio(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let ask_amount = self.query_sim(deps, offer_asset, ask_asset, offer_amount)?;
        Ok(Decimal256::checked_from_ratio(offer_amount, ask_amount)?)
    }

    fn query_reverse_swap_ratio(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let offer_amount = self.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)?;
        Ok(Decimal256::checked_from_ratio(offer_amount, ask_amount)?)
    }
}

impl Swap for cw_storage_plus::Map<'static, (&AssetInfo, &AssetInfo), Exchange> {
    fn swap(
        &self, deps: Deps<QueryWrapper>, env: &Env, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.swap(deps, env, offer_asset, ask_asset, offer_amount),
            Exchange::OrderBook(order_book) => order_book.swap(deps, env, offer_asset, ask_asset, offer_amount),
        }
    }

    fn swap_ask(
        &self, deps: Deps<QueryWrapper>, env: &Env, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.swap_ask(deps, env, offer_asset, ask_asset, ask_amount),
            Exchange::OrderBook(order_book) => order_book.swap_ask(deps, env, offer_asset, ask_asset, ask_amount),
        }
    }

    fn query_sim(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.query_sim(deps, offer_asset, ask_asset, offer_amount),
            Exchange::OrderBook(order_book) => order_book.query_sim(deps, offer_asset, ask_asset, offer_amount)
        }
    }

    fn query_reverse_sim(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Uint256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount),
            Exchange::OrderBook(order_book) => order_book.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount),
        }
    }

    fn query_swap_ratio(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, offer_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.query_swap_ratio(deps, offer_asset, ask_asset, offer_amount),
            Exchange::OrderBook(order_book) => order_book.query_swap_ratio(deps, offer_asset, ask_asset, offer_amount),
        }
    }

    fn query_reverse_swap_ratio(
        &self, deps: Deps<QueryWrapper>, offer_asset: &AssetInfo, ask_asset: &AssetInfo, ask_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        match get_exchange_type(deps, self, [offer_asset, ask_asset])? {
            Exchange::LiquidityPool(liquidity_pool) => liquidity_pool.query_reverse_swap_ratio(deps, offer_asset, ask_asset, ask_amount),
            Exchange::OrderBook(order_book) => order_book.query_reverse_swap_ratio(deps, offer_asset, ask_asset, ask_amount),
        }
    }
}