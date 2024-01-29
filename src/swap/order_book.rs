use crate::{
    asset::AssetInfo,
    error::NeptuneResult,
    injective::{into_decimal_256, into_fp_decimal, into_uint_256},
    msg_wrapper::MsgWrapper,
    query_wrapper::QueryWrapper,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Decimal256, Deps, Env, Fraction, QueryRequest, Uint256};
use injective_cosmwasm::{
    exchange::response::QueryOrderbookResponse, get_default_subaccount_id_for_checked_address,
    InjectiveMsg, InjectiveQuery, InjectiveRoute, MarketId, MarketMidPriceAndTOBResponse,
    OrderInfo, OrderSide, OrderType, QueryMarketAtomicExecutionFeeMultiplierResponse, SpotMarket,
    SpotMarketResponse, SpotOrder,
};
use injective_math::FPDecimal;

use super::{error::SwapError, Swap};

#[cw_serde]
pub struct OrderBook {
    pub market_id: MarketId,
}

impl Swap for OrderBook {
    fn swap(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        offer_asset: &AssetInfo,
        _ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        if let Some(msg) =
            market_order_offer(deps, env, self.market_id.clone(), offer_asset, offer_amount)?
        {
            Ok(vec![msg])
        } else {
            Ok(vec![])
        }
    }

    /// Override the default impl for more accuracy
    fn swap_ask(
        &self,
        deps: Deps<QueryWrapper>,
        env: &Env,
        _offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Vec<CosmosMsg<MsgWrapper>>> {
        if let Some(msg) =
            market_order_ask(deps, env, self.market_id.clone(), ask_asset, ask_amount)?
        {
            Ok(vec![msg])
        } else {
            Ok(vec![])
        }
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
        let AssetInfo::NativeToken { denom: offer_denom } = offer_asset else {
            return Err(SwapError::InvalidAsset.into());
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;

        let fee_rate = query_total_fees(deps, &spot_market);
        let offer_amount = offer_amount.into();

        let ask_amount = if offer_denom == &spot_market.quote_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Sell,
                None,
                Some(offer_amount),
            )?;
            get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount)?.quantity
        } else if offer_denom == &spot_market.base_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Buy,
                Some(offer_amount),
                None,
            )?;
            get_sell_ask_amount(&spot_market, fee_rate, &order_book, offer_amount)?
        } else {
            return Err(SwapError::InvalidOfferAsset.into());
        };

        Ok(into_uint_256(ask_amount.int()))
    }

    fn query_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        _ask_asset: &AssetInfo,
        offer_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let AssetInfo::NativeToken { denom: offer_denom } = offer_asset else {
            return Err(SwapError::InvalidAsset.into());
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;
        let fee_rate = query_total_fees(deps, &spot_market);
        let mut offer_amount = offer_amount.into();

        let ask_amount = if offer_denom == &spot_market.quote_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Sell,
                None,
                Some(offer_amount),
            )?;
            let buy_quantity = get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount)?
                .quantity
                .max(spot_market.min_quantity_tick_size);
            offer_amount = get_buy_offer_amount(&spot_market, fee_rate, &order_book, buy_quantity)?
                .offer_amount;
            buy_quantity
        } else if offer_denom == &spot_market.base_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Buy,
                Some(offer_amount),
                None,
            )?;
            offer_amount = tick_round_down(offer_amount, spot_market.min_quantity_tick_size)
                .max(spot_market.min_quantity_tick_size);
            get_sell_ask_amount(&spot_market, fee_rate, &order_book, offer_amount)?
        } else {
            return Err(SwapError::InvalidAsset.into());
        };

        if ask_amount.is_zero() {
            return Err(SwapError::InsufficientLiquidity.into());
        }

        let swap_ratio = offer_amount / ask_amount;

        Ok(into_decimal_256(swap_ratio))
    }

    /// Returns the of the offer asset required to receive the given amount of the ask asset, rounded up.
    /// Errors on insufficient liquidity.
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
        let AssetInfo::NativeToken { denom: ask_denom } = ask_asset else {
            return Err(SwapError::InvalidAsset.into());
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;

        let fee_rate = query_total_fees(deps, &spot_market);

        let ask_amount = ask_amount.into();

        let offer_amount = if ask_denom == &spot_market.base_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Sell,
                Some(ask_amount),
                None,
            )?;
            get_buy_offer_amount(&spot_market, fee_rate, &order_book, ask_amount)?.offer_amount
        } else if ask_denom == &spot_market.quote_denom {
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Buy,
                None,
                Some(ask_amount),
            )?;
            get_sell_quantity(&spot_market, fee_rate, &order_book, ask_amount)?
        } else {
            return Err(SwapError::InvalidAsset.into());
        };

        Ok(into_uint_256(offer_amount.int()))
    }

    fn query_ask_amount_at_price(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        _ask_asset: &AssetInfo,
        max_ratio: Decimal256,
    ) -> NeptuneResult<Uint256> {
        let AssetInfo::NativeToken { denom: offer_denom } = offer_asset else {
            return Err(SwapError::InvalidAsset.into());
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;

        let fee_rate = query_total_fees(deps, &spot_market);

        let ask_amount = if offer_denom == &spot_market.quote_denom {
            let price = into_fp_decimal(max_ratio);
            // TODO: This quey is problematic as we cannot effectively limit
            // TODO: our query.
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Sell,
                None,
                None,
            )?;
            get_buy_quantity_at_price(&spot_market, &order_book, price)?
        } else if offer_denom == &spot_market.base_denom {
            let price = into_fp_decimal(max_ratio.inv().unwrap());
            let order_book = query_spot_market_order_book(
                deps,
                self.market_id.clone(),
                0,
                OrderSide::Buy,
                None,
                None,
            )?;
            get_sell_ask_amount_at_price(&spot_market, fee_rate, &order_book, price)?
        } else {
            return Err(SwapError::InvalidAsset.into());
        };

        Ok(into_uint_256(ask_amount.int()))
    }

    /// Uses a swap simulation to calculate the ratio of offer to ask.
    fn query_reverse_swap_ratio(
        &self,
        deps: Deps<QueryWrapper>,
        offer_asset: &AssetInfo,
        ask_asset: &AssetInfo,
        ask_amount: Uint256,
    ) -> NeptuneResult<Decimal256> {
        let offer_amount = self.query_reverse_sim(deps, offer_asset, ask_asset, ask_amount)?;
        self.query_swap_ratio(deps, offer_asset, ask_asset, offer_amount)
    }
}

pub fn market_order_offer(
    deps: Deps<QueryWrapper>,
    env: &Env,
    market_id: MarketId,
    offer_asset: &AssetInfo,
    offer_amount: Uint256,
) -> NeptuneResult<Option<CosmosMsg<MsgWrapper>>> {
    let offer_amount = FPDecimal::from(offer_amount);

    if offer_amount.is_zero() {
        return Ok(None);
    }

    let AssetInfo::NativeToken { denom: offer_denom } = offer_asset else {
        return Err(SwapError::InvalidAsset.into());
    };

    let spot_market = query_spot_market(deps, market_id.clone())?;

    let fee_rate = query_total_fees(deps, &spot_market);

    if &spot_market.quote_denom == offer_denom {
        let order_book = query_spot_market_order_book(
            deps,
            market_id,
            0,
            OrderSide::Sell,
            None,
            Some(offer_amount),
        )?;
        let buy_quantity = get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount)?;
        buy(
            env,
            &spot_market,
            buy_quantity.worst_order_price,
            buy_quantity.quantity,
        )
    } else if &spot_market.base_denom == offer_denom {
        sell(env, &spot_market, offer_amount)
    } else {
        return Err(SwapError::InvalidAsset.into());
    }
}

pub fn market_order_ask(
    deps: Deps<QueryWrapper>,
    env: &Env,
    market_id: MarketId,
    ask_asset: &AssetInfo,
    ask_amount: Uint256,
) -> NeptuneResult<Option<CosmosMsg<MsgWrapper>>> {
    let ask_amount = FPDecimal::from(ask_amount);

    if ask_amount.is_zero() {
        return Ok(None);
    }

    let AssetInfo::NativeToken { denom: ask_denom } = ask_asset else {
        return Err(SwapError::InvalidAsset.into());
    };

    let spot_market = query_spot_market(deps, market_id.clone())?;

    let fee_rate = query_total_fees(deps, &spot_market);

    if &spot_market.base_denom == ask_denom {
        let order_book = query_spot_market_order_book(
            deps,
            market_id,
            0,
            OrderSide::Sell,
            Some(ask_amount),
            None,
        )?;
        let worst_order_price =
            get_buy_offer_amount(&spot_market, fee_rate, &order_book, ask_amount)?
                .worst_order_price;
        buy(env, &spot_market, worst_order_price, ask_amount)
    } else if &spot_market.quote_denom == ask_denom {
        let order_book = query_spot_market_order_book(
            deps,
            market_id,
            0,
            OrderSide::Buy,
            None,
            Some(ask_amount),
        )?;
        let quantity = get_sell_quantity(&spot_market, fee_rate, &order_book, ask_amount)?.int();
        sell(env, &spot_market, quantity)
    } else {
        return Err(SwapError::InvalidAsset.into());
    }
}

struct GetBuyQuantity {
    quantity: FPDecimal,
    worst_order_price: Option<FPDecimal>,
}

/// returns the quantity of the ask asset (rounded down)
/// that can be bought with the given offer amount
fn get_buy_quantity(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    offer_amount: FPDecimal, // quote
) -> NeptuneResult<GetBuyQuantity> {
    let mut remaining_offer_amount = offer_amount; // quote
    let mut quantity = FPDecimal::ZERO; // base
    let mut worst_order_price = None;
    for sell_order in &order_book.sells_price_level {
        let sell_order_quantity = sell_order.q;
        let sell_order_price = sell_order.p;
        worst_order_price = Some(sell_order_price);
        let sell_order_base_amount = apply_fee(sell_order_quantity * sell_order_price, fee_rate);
        if remaining_offer_amount > sell_order_base_amount {
            quantity += sell_order_quantity;
            remaining_offer_amount -= sell_order_base_amount;
        } else {
            // `sell_order_price` cannot be zero, no need to check.
            quantity +=
                (remaining_offer_amount / ((FPDecimal::ONE + fee_rate) * sell_order_price)).int();
            break;
        }
    }
    quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
    Ok(GetBuyQuantity {
        quantity,
        worst_order_price,
    })
}

/// returns the quantity of the ask asset (rounded down)
/// that can be bought for less that `price`.
fn get_buy_quantity_at_price(
    spot_market: &SpotMarket,
    order_book: &QueryOrderbookResponse,
    price: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let mut quantity = FPDecimal::ZERO; // base
    for sell_order in &order_book.sells_price_level {
        let sell_order_quantity = sell_order.q;
        let sell_order_price = sell_order.p;
        if sell_order_price > price {
            break;
        }
        quantity += sell_order_quantity;
    }
    quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
    Ok(quantity)
}

/// Returns the quantity of the offer asset (rounded up)
/// that can be sold for more than `price`.
fn get_sell_ask_amount_at_price(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    price: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let mut quantity = FPDecimal::ZERO; // base
    for buy_order in &order_book.buys_price_level {
        let buy_order_quantity = buy_order.q;
        let buy_order_price = buy_order.p;
        if buy_order_price < price {
            break;
        }
        let buy_order_base_amount = apply_fee(buy_order_quantity * buy_order_price, fee_rate);
        quantity += buy_order_base_amount;
    }
    let quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
    Ok(quantity)
}

/// Returns the quantity of the offer asset (rounded up)
/// that is required to receive the ask amount.
/// Will throw an error on insufficient liquidity.
fn get_sell_quantity(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    ask_amount: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let mut remaining_ask_amount = ask_amount; // quote
    let mut quantity = FPDecimal::ZERO; // base
    for buy_order in &order_book.buys_price_level {
        let buy_order_quantity = buy_order.q;
        let buy_order_price = buy_order.p;
        let buy_order_quote_amount =
            ((buy_order_quantity * buy_order_price) * (FPDecimal::ONE - fee_rate)).int();
        if remaining_ask_amount > buy_order_quote_amount {
            quantity += buy_order_quantity;
            remaining_ask_amount -= buy_order_quote_amount;
        } else {
            // `buy_order_price` cannot be zero, no need to check.
            quantity += tick_round_up(
                apply_fee(remaining_ask_amount / buy_order_price, fee_rate),
                spot_market.min_quantity_tick_size,
            );
            remaining_ask_amount = FPDecimal::ZERO;
            break;
        }
    }
    if !remaining_ask_amount.is_zero() {
        return Err(SwapError::InsufficientLiquidity.into());
    }
    quantity *= FPDecimal::must_from_str("1.00001");
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    Ok(quantity)
}

struct GetBuyOfferAmount {
    offer_amount: FPDecimal,
    worst_order_price: Option<FPDecimal>,
}

/// returns the offer amount amount_required to purchase
/// a given quantity of the ask asset
fn get_buy_offer_amount(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    quantity: FPDecimal, // quote
) -> NeptuneResult<GetBuyOfferAmount> {
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    let mut offer_amount = FPDecimal::ZERO;
    let mut worst_order_price = None;
    let mut remaining_quantity = quantity;
    for sell_order in &order_book.sells_price_level {
        let sell_order_quantity = sell_order.q;
        let sell_order_price = sell_order.p;
        worst_order_price = Some(sell_order_price);
        if sell_order_quantity > remaining_quantity {
            offer_amount += apply_fee(remaining_quantity * sell_order_price, fee_rate);
            remaining_quantity = FPDecimal::ZERO;
            break;
        } else {
            offer_amount += apply_fee(sell_order_quantity * sell_order_price, fee_rate);
            remaining_quantity -= sell_order_quantity;
        }
    }
    if !remaining_quantity.is_zero() {
        return Err(SwapError::InsufficientLiquidity.into());
    }
    Ok(GetBuyOfferAmount {
        offer_amount,
        worst_order_price,
    })
}

/// returns the ask amount received from selling
/// a given quantity of the offer asset
fn get_sell_ask_amount(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    quantity: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
    let mut ask_amount = FPDecimal::ZERO;
    let mut remaining_quantity = quantity;
    for buy_order in &order_book.buys_price_level {
        let buy_order_quantity = buy_order.q;
        let buy_order_price = buy_order.p;
        if buy_order_quantity > remaining_quantity {
            ask_amount +=
                ((remaining_quantity * buy_order_price) / (FPDecimal::ONE + fee_rate)).int();
            break;
        } else {
            ask_amount +=
                ((buy_order_quantity * buy_order_price) / (FPDecimal::ONE + fee_rate)).int();
            remaining_quantity -= buy_order_quantity;
        }
    }
    // TODO: why does this work?
    ask_amount = tick_round_down(ask_amount, FPDecimal::from(10_000_000_000_000u128));

    Ok(ask_amount)
}

/// Buys the given quantity rounded up, erroring on insufficient funds
/// `worst_order_price` is the worst price that can be accepted
/// It must be specified accurately or the module will attempt to withdraw
/// more funds than are available.j
fn buy(
    env: &Env,
    spot_market: &SpotMarket,
    worst_order_price: Option<FPDecimal>,
    quantity: FPDecimal,
) -> NeptuneResult<Option<CosmosMsg<MsgWrapper>>> {
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    if quantity.is_zero() {
        return Ok(None);
    }
    let worst_order_price = worst_order_price.ok_or(SwapError::InsufficientLiquidity)?;
    let price = tick_round_down(worst_order_price, spot_market.min_price_tick_size);
    let subaccount_id = get_default_subaccount_id_for_checked_address(&env.contract.address);

    let order_info = OrderInfo {
        subaccount_id,
        fee_recipient: Some(env.contract.address.clone()),
        price,
        quantity,
        // TODO
        cid: None,
    };

    let order = SpotOrder {
        market_id: spot_market.market_id.clone(),
        order_info,
        order_type: OrderType::BuyAtomic,
        trigger_price: None,
    };

    let wrapper = MsgWrapper {
        route: InjectiveRoute::Exchange,
        msg_data: InjectiveMsg::CreateSpotMarketOrder {
            sender: env.contract.address.clone(),
            order,
        },
    };

    Ok(Some(CosmosMsg::Custom(wrapper)))
}

/// Sells the given quantity rounded down, erroring on insufficient funds
fn sell(
    env: &Env,
    spot_market: &SpotMarket,
    quantity: FPDecimal,
) -> NeptuneResult<Option<CosmosMsg<MsgWrapper>>> {
    let quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
    if quantity.is_zero() {
        return Ok(None);
    }
    let price = spot_market.min_price_tick_size;
    let subaccount_id = get_default_subaccount_id_for_checked_address(&env.contract.address);

    let order_info = OrderInfo {
        subaccount_id,
        fee_recipient: Some(env.contract.address.clone()),
        price,
        quantity,
        cid: None,
    };

    let order = SpotOrder {
        market_id: spot_market.market_id.clone(),
        order_info,
        order_type: OrderType::SellAtomic,
        trigger_price: None,
    };

    let wrapper = MsgWrapper {
        route: InjectiveRoute::Exchange,
        msg_data: InjectiveMsg::CreateSpotMarketOrder {
            sender: env.contract.address.clone(),
            order,
        },
    };

    Ok(Some(CosmosMsg::Custom(wrapper)))
}

pub fn query_spot_market_mid_price_and_tob(
    deps: Deps<QueryWrapper>,
    market_id: MarketId,
) -> NeptuneResult<MarketMidPriceAndTOBResponse> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::SpotMarketMidPriceAndTob { market_id },
    };

    let query_request = QueryRequest::Custom(wrapper);

    Ok(deps.querier.query(&query_request)?)
}

pub fn query_spot_market(
    deps: Deps<QueryWrapper>,
    market_id: MarketId,
) -> NeptuneResult<SpotMarket> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::SpotMarket { market_id },
    };

    let query_request = QueryRequest::Custom(wrapper);

    let res: SpotMarketResponse = deps.querier.query(&query_request)?;

    let spot_market = res.market.ok_or(SwapError::SpotMarketNotFound)?;

    Ok(spot_market)
}

fn query_spot_market_order_book(
    deps: Deps<QueryWrapper>,
    market_id: MarketId,
    limit: u64,
    order_side: OrderSide,
    limit_cumulative_quantity: Option<FPDecimal>,
    limit_cumulative_notional: Option<FPDecimal>,
) -> NeptuneResult<QueryOrderbookResponse> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::SpotOrderbook {
            market_id,
            limit,
            order_side,
            limit_cumulative_quantity,
            limit_cumulative_notional,
        },
    };

    let query_request = QueryRequest::Custom(wrapper);

    Ok(deps.querier.query(&query_request)?)
}

fn query_atomic_fee_execution_multiplier(
    deps: Deps<QueryWrapper>,
    market_id: MarketId,
) -> NeptuneResult<FPDecimal> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::MarketAtomicExecutionFeeMultiplier { market_id },
    };

    let query_request = QueryRequest::Custom(wrapper);

    let res: QueryMarketAtomicExecutionFeeMultiplierResponse =
        deps.querier.query(&query_request)?;

    Ok(res.multiplier)
}

fn query_total_fees(deps: Deps<QueryWrapper>, spot_market: &SpotMarket) -> FPDecimal {
    let multiplier =
        query_atomic_fee_execution_multiplier(deps, spot_market.market_id.clone()).unwrap();
    multiplier * spot_market.taker_fee_rate * (FPDecimal::ONE - spot_market.relayer_fee_share_rate)
}

fn apply_fee(value: FPDecimal, fee: FPDecimal) -> FPDecimal {
    let res = value.int() * (FPDecimal::ONE + fee);
    if res.is_int() {
        res
    } else {
        (res + FPDecimal::ONE).int()
    }
}

pub fn tick_round_up(value: FPDecimal, tick_size: FPDecimal) -> FPDecimal {
    let tick_num = value / tick_size;
    let tick_num = if tick_num.is_int() {
        tick_num
    } else {
        (tick_num + FPDecimal::ONE).int() // no ceiling function
    };
    tick_num * tick_size
}

pub fn tick_round_down(value: FPDecimal, tick_size: FPDecimal) -> FPDecimal {
    let tick_num = (value / tick_size).int();
    tick_num * tick_size
}
