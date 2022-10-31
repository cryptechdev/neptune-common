use cosmwasm_std::MessageInfo;
use cw20::Cw20ReceiveMsg;

use crate::{
    asset::{AssetAmount, AssetInfo},
    error::{CommonError, CommonResult},
};

pub fn get_provided_asset_amount(
    info: &MessageInfo, cw20_receive_msg: &Option<Cw20ReceiveMsg>,
) -> CommonResult<AssetAmount> {
    if let Some(cw20_receive_msg) = cw20_receive_msg {
        Ok(AssetAmount {
            info:   AssetInfo::Token { contract_addr: info.sender.clone() },
            amount: cw20_receive_msg.amount.into(),
        })
    } else {
        let coin = info.funds.get(0).ok_or(CommonError::NoFundsReceived {})?;
        Ok(coin.into())
    }
}

pub fn assert_no_multiple_tx(last_tx_height: &mut u64, current_block_height: u64) -> CommonResult<()> {
    if *last_tx_height == current_block_height {
        return Err(CommonError::MultipleTx {});
    } else {
        *last_tx_height = current_block_height;
        Ok(())
    }
}
