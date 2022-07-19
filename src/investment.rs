use cosmwasm_std::Uint256;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};

use crate::execute_base::BaseExecuteMsg;

/// The external execute calls that can be performed on an investment.
/// These calls are private to be performed only by the vault that created it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvestmentBaseExecuteMsg {
    Base(BaseExecuteMsg),

    Invest { cw20_receive_msg: Option<Cw20ReceiveMsg> },

    Divest { amount: Uint256 },

    SendFundsToInvestment { amount: Uint256 },

    SendFundsToVaultForDivestment { amount: Uint256 },

    WithdrawFundsFromInvestment { amount: Uint256 },
    
    ClaimRewards { },
}

impl From<BaseExecuteMsg> for InvestmentBaseExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InvestmentBase(InvestmentBaseExecuteMsg)
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::InvestmentBase(base.into())
    }
}

/// The public queries that can be called on an investment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Gets the value of the investment as measured in UST
    GetInvestmentValue { },

    /// Gets the value of the investment that can actually be withdrawn as measured in UST
    GetInvestmentRedeemable { },

    GetAssetPrice { },

    GetPendingRewards { },

    GetApy { },

    GetParams { },

    GetConfig { },

    GetState { },
}