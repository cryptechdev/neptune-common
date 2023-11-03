use cosmwasm_std::{to_json_binary, CosmosMsg, Env, WasmMsg};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{NeptuneError, NeptuneResult};

/// Asserts that the current block height is not the same as the last transaction height.
/// This exists to prevent multiple transactions from being sent in the same block
/// thereby preventing common flash loan attacks.
///
/// ```
/// # use neptune_common::utilities::{assert_no_multiple_tx};
/// let mut last_tx_height: u64 = 0;
/// assert!(assert_no_multiple_tx(&mut last_tx_height, 1).is_ok());
/// assert!(assert_no_multiple_tx(&mut last_tx_height, 1).is_err());
/// ```
pub fn assert_no_multiple_tx(
    last_tx_height: &mut u64,
    current_block_height: u64,
) -> NeptuneResult<()> {
    if *last_tx_height == current_block_height {
        Err(NeptuneError::MultipleTx {})
    } else {
        *last_tx_height = current_block_height;
        Ok(())
    }
}

/// Sends a message to the contract itself.
pub fn msg_to_self<ExecuteMsg: Serialize + DeserializeOwned, M>(
    env: &Env,
    msg: &ExecuteMsg,
) -> NeptuneResult<CosmosMsg<M>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_json_binary(&msg)?,
    }))
}
