use cosmwasm_std::MessageInfo;
use cw20::Cw20ReceiveMsg;

use crate::{asset::{AssetAmount, AssetInfo}, error::{CommonResult, CommonError}};

pub fn get_provided_asset_amount(
    info: &MessageInfo,
    cw20_receive_msg: &Option<Cw20ReceiveMsg>
) -> CommonResult<AssetAmount> {
    if let Some(cw20_receive_msg) = cw20_receive_msg {
        Ok(AssetAmount {
            info: AssetInfo::Token {
                contract_addr: info.sender.clone(),
            },
            amount: cw20_receive_msg.amount.into(),
        })
    } else {
        let coin = info.funds.get(0).ok_or(
            CommonError::NoFundsReceived {  }
        )?;
        Ok(coin.into())
    }
}