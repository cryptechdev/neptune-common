// pub fn router_swap<E: Serialize+DeserializeOwned+From<BaseExecuteMsg>>(
//     deps: Deps,
//     env: &Env,
//     offer_asset_info: AssetInfo,
//     ask_asset_info: AssetInfo,
//     offer_amount: Uint256
// ) -> MoneyMarketResult<Vec<CosmosMsg>> {

//     let mut msgs = vec![];

//     if offer_amount.is_zero(){ return Ok(msgs); }

//     // let receive_amount = query_lp_coin_simulation(deps, &swap_pool,
// offer_asset.clone().into(), offer_amount)?;

//     // if receive_amount.is_zero(){ return Ok(msgs); }

//     let binary_msg =
//         match offer_asset_info {
//             AssetInfo::Token { .. } => {
//                 to_binary(&astroport::router::ExecuteMsg::ExecuteSwapOperations {
//                     operations: vec![astroport::router::SwapOperation::AstroSwap {
//                         offer_asset_info: offer_asset_info.clone(),
//                         ask_asset_info
//                     }],
//                     minimum_receive: None,
//                     max_spread: Some(Decimal::percent(50)),
//                     to: Option::None,
//                 })?
//             },
//             AssetInfo::NativeToken { .. } => {
//                 to_binary(&astroport::router::Cw20HookMsg::ExecuteSwapOperations {
//                     operations: vec![astroport::router::SwapOperation::AstroSwap {
//                         offer_asset_info: offer_asset_info.clone(),
//                         ask_asset_info
//                     }],
//                     minimum_receive: None,
//                     max_spread: Some(Decimal::percent(50)),
//                     to: Option::None,
//                 })?
//             },
//     };

//     msgs.push(msg_to_self(env, &E::from(BaseExecuteMsg::SendFunds{
//         recipient: get_router_addr(deps)?,
//         amount: offer_amount,
//         send_msg: offer_asset_info.into(),
//         exec_msg: Some(binary_msg)
//     }))?);
//     Ok(msgs)
// }

// // pub fn query_best_route(
// //     deps: Deps,
// //     offer_asset_info: AssetInfo,
// //     ask_asset_info: AssetInfo,
// //     offer_amount: Uint256,
// //     hub_assets: Vec<AssetInfo>,
// // ) -> MoneyMarketResult<Vec<SwapOperation>> {

// //     let direct = query_router_sim(deps, offer_asset_info.clone(), ask_asset_info.clone(),
// // offer_amount)?;     let mut result = vec![];
// //     for hub_asset in hub_assets.clone() {
// //         let intermediate = query_router_sim(deps, offer_asset_info.clone(), hub_asset.clone(),
// // offer_amount)?;         let end = query_router_sim(deps, hub_asset.clone(),
// // ask_asset_info.clone(), intermediate)?;         result.push(end);
// //     }
// //     let largest = result.iter().max().unwrap();
// //     if &direct >= largest {
// //         return Ok(vec![
// //             SwapOperation::AstroSwap {
// //                 offer_asset_info: offer_asset_info,
// //                 ask_asset_info: ask_asset_info
// //             }
// //         ]);
// //     } else {
// //         let index = result.iter().position(|x| x == largest).unwrap();
// //         let asset = &hub_assets[index];
// //         return Ok(vec![
// //             SwapOperation::AstroSwap {
// //                 offer_asset_info: offer_asset_info,
// //                 ask_asset_info: asset.clone(),
// //             },
// //             SwapOperation::AstroSwap {
// //                 offer_asset_info: asset.clone(),
// //                 ask_asset_info: ask_asset_info
// //             }
// //         ]);
// //     }
// // }

// pub fn query_router_sim(
//     deps: Deps,
//     offer_asset_info: AssetInfo,
//     ask_asset_info: AssetInfo,
//     offer_amount: Uint256
// ) -> MoneyMarketResult<Uint256> {

//     if offer_amount.is_zero() { return Ok(Uint256::zero()) }

//     let swap_operation = SwapOperation::AstroSwap {
//         offer_asset_info,
//         ask_asset_info,
//     };

//     Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: get_router_addr(deps)?.into(),
//         msg: to_binary(&astroport::router::QueryMsg::SimulateSwapOperations {
//             offer_amount: offer_amount.try_into()?,
//             operations: vec![swap_operation],
//         })?,
//     }))?)

// }

// pub fn query_lp_token_simulation(
//     deps: Deps,
//     pool_addr: &Addr,
//     token_addr: &Addr,
//     amount: Uint256
// ) -> MoneyMarketResult<Uint256> {

//     if amount.is_zero() { return Ok(Uint256::zero()) }

//     Ok(astroport::querier::simulate(
//         &deps.querier,
//         pool_addr.clone(),
//         &AssetAmount {
//             info: AssetInfo::Token {
//                 contract_addr: token_addr.clone()
//             },
//             amount: amount,
//         }
//     )?.return_amount.into())
// }

// pub fn query_lp_coin_simulation(
//     deps: Deps,
//     pool_addr: &Addr,
//     offer_asset: AssetInfo,
//     amount: Uint256
// ) -> MoneyMarketResult<Uint256> {

//     if amount.is_zero() { return Ok(Uint256::zero()) }

//     Ok(astroport::querier::simulate(
//         &deps.querier,
//         pool_addr.clone(),
//         &AssetAmount {
//             info: offer_asset,
//             amount: amount,
//         }
//     )?.return_amount.into())
// }

// pub fn query_reverse_token_sim(
//     deps: Deps,
//     pool_addr: Addr,
//     token_addr: Addr,
//     ask_amount: Uint256
// ) -> MoneyMarketResult<Uint256> {

//     if ask_amount.is_zero() { return Ok(Uint256::zero()) }

//     Ok(match astroport::querier::reverse_simulate(
//         &deps.querier,
//         &pool_addr,
//         &AssetAmount {
//             info:  AssetInfo::Token {
//                 contract_addr: token_addr.clone(),
//             },
//             amount: ask_amount,
//         }
//     ) {
//         Ok(response) => response.offer_amount.into(),
//         Err(_) => {
//             let token_price = query_lp_token_simulation(
//                 deps, &pool_addr, &token_addr, Uint256::from(1000000u128)
//             )?;
//             if token_price.is_zero() { return Err(CommonError::ZeroDenominator {})}
//             // include a 1% extra to account for slippage and protocol fees (1000000/990099 =
// ~1.01)             ask_amount.multiply_ratio(token_price,Uint256::from(990099u128))
//         },
//     })
// }

// pub fn query_reverse_coin_sim(
//     deps: Deps,
//     pool_addr: Addr,
//     ask_asset: AssetInfo,
//     ask_amount: Uint256
// ) -> MoneyMarketResult<Uint256> {

//     if ask_amount.is_zero() { return Ok(Uint256::zero()) }

//     Ok(match astroport::querier::reverse_simulate(
//         &deps.querier,
//         &pool_addr,
//         &AssetAmount {
//             info:  ask_asset.clone(),
//             amount: ask_amount,
//         }
//     ) {
//         Ok(response) => response.offer_amount.into(),
//         Err(_) => {
//             let coin_price = query_lp_coin_simulation(
//                 deps, &pool_addr, ask_asset, Uint256::from(1000000u128)
//             )?;
//             if coin_price.is_zero() { return Err(CommonError::ZeroDenominator {})}
//             // include a 1% extra to account for slippage and protocol fees (1000000/990099 =
// ~1.01)             ask_amount.multiply_ratio(coin_price,Uint256::from(990099u128))
//         },
//     })
// }
