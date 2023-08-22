use crate::{
    asset::AssetInfo,
    error::{NeptuneError, NeptuneResult},
    injective::{into_decimal_256, into_fp_decimal, into_uint_256},
    msg_wrapper::MsgWrapper,
    query_wrapper::QueryWrapper,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Decimal256, Deps, Env, QueryRequest, Uint256};
use injective_cosmwasm::{
    exchange::response::QueryOrderbookResponse, get_default_subaccount_id_for_checked_address,
    InjectiveMsg, InjectiveQuery, InjectiveRoute, MarketId, MarketMidPriceAndTOBResponse,
    OrderInfo, OrderSide, OrderType, QueryMarketAtomicExecutionFeeMultiplierResponse, SpotMarket,
    SpotMarketResponse, SpotOrder,
};
use injective_math::FPDecimal;

use super::Swap;

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
            return Err(NeptuneError::Generic(
                "Only native tokens are supported".to_string(),
            ));
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;
        let order_book = query_spot_market_order_book(
            deps,
            self.market_id.clone(),
            0,
            OrderSide::Unspecified,
            None,
            None,
        )?;
        let fee_rate = query_total_fees(deps, &spot_market);

        let ask_amount = if offer_denom == &spot_market.quote_denom {
            get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount.into())?
        } else if offer_denom == &spot_market.base_denom {
            get_sell_ask_amount(&spot_market, fee_rate, &order_book, offer_amount.into())?
        } else {
            return Err(NeptuneError::Generic("Invalid offer asset".to_string()));
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
            return Err(NeptuneError::Generic(
                "Only native tokens are supported".to_string(),
            ));
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;
        let order_book = query_spot_market_order_book(
            deps,
            self.market_id.clone(),
            0,
            OrderSide::Unspecified,
            None,
            None,
        )?;
        let fee_rate = query_total_fees(deps, &spot_market);
        let mut offer_amount = offer_amount.into();

        let ask_amount = if offer_denom == &spot_market.quote_denom {
            let buy_quantity = get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount)?
                .max(spot_market.min_quantity_tick_size);
            offer_amount = get_buy_offer_amount(&spot_market, fee_rate, &order_book, buy_quantity)?;
            buy_quantity
        } else if offer_denom == &spot_market.base_denom {
            offer_amount = tick_round_down(offer_amount, spot_market.min_quantity_tick_size)
                .max(spot_market.min_quantity_tick_size);
            get_sell_ask_amount(&spot_market, fee_rate, &order_book, offer_amount)?
        } else {
            return Err(NeptuneError::Generic("Invalid offer asset".to_string()));
        };

        if ask_amount.is_zero() {
            return Err(NeptuneError::Generic(
                "Market is empty, could not calculate swap ratio".to_string(),
            ));
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
            return Err(NeptuneError::Generic(
                "Only native tokens are supported".to_string(),
            ));
        };
        let spot_market = query_spot_market(deps, self.market_id.clone())?;
        let order_book = query_spot_market_order_book(
            deps,
            self.market_id.clone(),
            0,
            OrderSide::Unspecified,
            None,
            None,
        )?;
        let fee_rate = query_total_fees(deps, &spot_market);

        let offer_amount = if ask_denom == &spot_market.base_denom {
            get_buy_offer_amount(&spot_market, fee_rate, &order_book, ask_amount.into())?
        } else if ask_denom == &spot_market.quote_denom {
            get_sell_quantity(&spot_market, fee_rate, &order_book, ask_amount.into())?
        } else {
            return Err(NeptuneError::Generic("Invalid offer asset".to_string()));
        };

        Ok(into_uint_256(offer_amount.int()))
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
        return Err(NeptuneError::Generic(
            "Only native tokens are supported".to_string(),
        ));
    };

    let spot_market = query_spot_market(deps, market_id.clone())?;
    let order_book =
        query_spot_market_order_book(deps, market_id, 0, OrderSide::Unspecified, None, None)?;
    let fee_rate = query_total_fees(deps, &spot_market);

    if &spot_market.quote_denom == offer_denom {
        let quantity = get_buy_quantity(&spot_market, fee_rate, &order_book, offer_amount)?;
        buy(env, &spot_market, fee_rate, offer_amount, quantity)
    } else if &spot_market.base_denom == offer_denom {
        sell(env, &spot_market, offer_amount)
    } else {
        return Err(NeptuneError::Generic("Invalid offer asset".to_string()));
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
        return Err(NeptuneError::Generic(
            "Only native tokens are supported".to_string(),
        ));
    };

    let spot_market = query_spot_market(deps, market_id.clone())?;
    let order_book =
        query_spot_market_order_book(deps, market_id, 0, OrderSide::Unspecified, None, None)?;
    let fee_rate = query_total_fees(deps, &spot_market);

    if &spot_market.base_denom == ask_denom {
        let expected_offer_amount =
            get_buy_offer_amount(&spot_market, fee_rate, &order_book, ask_amount)?;
        buy(
            env,
            &spot_market,
            fee_rate,
            expected_offer_amount,
            ask_amount,
        )
    } else if &spot_market.quote_denom == ask_denom {
        let quantity = get_sell_quantity(&spot_market, fee_rate, &order_book, ask_amount)?;
        sell(env, &spot_market, quantity)
    } else {
        return Err(NeptuneError::Generic("Invalid offer asset".to_string()));
    }
}

/// returns the quantity of the ask asset (rounded down)
/// that can be bought with the given offer amount
fn get_buy_quantity(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    offer_amount: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let mut remaining_offer_amount = offer_amount; // quote
    let mut quantity = FPDecimal::zero(); // base
    for sell_order in &order_book.sells_price_level {
        let sell_order_quantity = sell_order.q;
        let sell_order_price = sell_order.p;
        let sell_order_base_amount = apply_fee(sell_order_quantity * sell_order_price, fee_rate);
        if remaining_offer_amount > sell_order_base_amount {
            quantity += sell_order_quantity;
            remaining_offer_amount -= sell_order_base_amount;
        } else {
            // `sell_order_price` cannot be zero, no need to check.
            quantity +=
                (remaining_offer_amount / ((FPDecimal::one() + fee_rate) * sell_order_price)).int();
            break;
        }
    }
    quantity = tick_round_down(quantity, spot_market.min_quantity_tick_size);
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
    let mut quantity = FPDecimal::zero(); // base
    for buy_order in &order_book.buys_price_level {
        let buy_order_quantity = buy_order.q;
        let buy_order_price = buy_order.p;
        let buy_order_base_amount = apply_fee(buy_order_quantity * buy_order_price, fee_rate);
        if remaining_ask_amount > buy_order_base_amount {
            quantity += buy_order_quantity;
            remaining_ask_amount -= buy_order_base_amount;
        } else {
            // `buy_order_price` cannot be zero, no need to check.
            quantity +=
                (remaining_ask_amount / ((FPDecimal::one() + fee_rate) * buy_order_price)).int();
            remaining_ask_amount = FPDecimal::zero();
            break;
        }
    }
    if !remaining_ask_amount.is_zero() {
        return Err(NeptuneError::InsufficientLiquidity);
    }
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    Ok(quantity)
}

/// returns the offer amount amount_required to purchase
/// a given quantity of the ask asset
fn get_buy_offer_amount(
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    order_book: &QueryOrderbookResponse,
    quantity: FPDecimal, // quote
) -> NeptuneResult<FPDecimal> {
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    let mut offer_amount = FPDecimal::zero();
    let mut remaining_quantity = quantity;
    for sell_order in &order_book.sells_price_level {
        let sell_order_quantity = sell_order.q;
        let sell_order_price = sell_order.p;
        if sell_order_quantity > remaining_quantity {
            offer_amount += apply_fee(remaining_quantity * sell_order_price, fee_rate);
            remaining_quantity = FPDecimal::zero();
            break;
        } else {
            offer_amount += apply_fee(sell_order_quantity * sell_order_price, fee_rate);
            remaining_quantity -= sell_order_quantity;
        }
    }
    if !remaining_quantity.is_zero() {
        return Err(NeptuneError::InsufficientLiquidity);
    }
    Ok(offer_amount)
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
    let mut ask_amount = FPDecimal::zero();
    let mut remaining_quantity = quantity;
    for buy_order in &order_book.buys_price_level {
        let buy_order_quantity = buy_order.q;
        let buy_order_price = buy_order.p;
        if buy_order_quantity > remaining_quantity {
            ask_amount +=
                ((remaining_quantity * buy_order_price) / (FPDecimal::one() + fee_rate)).int();
            break;
        } else {
            ask_amount +=
                ((buy_order_quantity * buy_order_price) / (FPDecimal::one() + fee_rate)).int();
            remaining_quantity -= buy_order_quantity;
        }
    }
    Ok(ask_amount)
}

/// Buys the given quantity rounded up, erroring on insufficient funds
fn buy(
    env: &Env,
    spot_market: &SpotMarket,
    fee_rate: FPDecimal,
    expected_offer_amount: FPDecimal,
    quantity: FPDecimal,
) -> NeptuneResult<Option<CosmosMsg<MsgWrapper>>> {
    let quantity = tick_round_up(quantity, spot_market.min_quantity_tick_size);
    if quantity.is_zero() {
        return Ok(None);
    }
    if expected_offer_amount.is_zero() {
        return Ok(None);
    }

    let expected_offer_amount_less_fees =
        (expected_offer_amount / (FPDecimal::one() + fee_rate)).int();
    let price = expected_offer_amount_less_fees / quantity;
    let price = tick_round_up(price, spot_market.min_price_tick_size);

    let subaccount_id = get_default_subaccount_id_for_checked_address(&env.contract.address);

    let order_info = OrderInfo {
        subaccount_id,
        fee_recipient: Some(env.contract.address.clone()),
        price,
        quantity,
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

fn query_spot_market(deps: Deps<QueryWrapper>, market_id: MarketId) -> NeptuneResult<SpotMarket> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::SpotMarket { market_id },
    };

    let query_request = QueryRequest::Custom(wrapper);

    let res: SpotMarketResponse = deps.querier.query(&query_request)?;

    let spot_market = res
        .market
        .ok_or(NeptuneError::Generic("Spot market not found".to_string()))?;

    Ok(spot_market)
}

fn query_spot_market_order_book(
    deps: Deps<QueryWrapper>,
    market_id: MarketId,
    limit: u64,
    order_side: OrderSide,
    limit_cumulative_quantity: Option<Decimal256>,
    limit_cumulative_notional: Option<Decimal256>,
) -> NeptuneResult<QueryOrderbookResponse> {
    let wrapper = QueryWrapper {
        route: InjectiveRoute::Exchange,
        query_data: InjectiveQuery::SpotOrderbook {
            market_id,
            limit,
            order_side,
            limit_cumulative_quantity: limit_cumulative_quantity.map(into_fp_decimal),
            limit_cumulative_notional: limit_cumulative_notional.map(into_fp_decimal),
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
    let res = value.int() * (FPDecimal::one() + fee);
    if res.is_int() {
        res
    } else {
        (res + FPDecimal::one()).int()
    }
}

pub fn tick_round_up(value: FPDecimal, tick_size: FPDecimal) -> FPDecimal {
    let tick_num = value / tick_size;
    let tick_num = if tick_num.is_int() {
        tick_num
    } else {
        (tick_num + FPDecimal::one()).int() // no ceiling function
    };
    tick_num * tick_size
}

pub fn tick_round_down(value: FPDecimal, tick_size: FPDecimal) -> FPDecimal {
    let tick_num = (value / tick_size).int();
    tick_num * tick_size
}
