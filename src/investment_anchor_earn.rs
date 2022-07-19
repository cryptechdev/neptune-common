use cosmwasm_std::{DepsMut, StdResult};
use schemars::JsonSchema;

use serde::{
    Deserialize,
    Serialize
};

//use crate::investment_base::InvestmentSetConfigMsg;
use crate::{base_config::{BaseSetConfigMsg, ConfigMsgTrait}, investment::InvestmentBaseExecuteMsg, execute_base::BaseExecuteMsg};

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
}

/// Config variables for the base investment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
}

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SetConfigMsg {
    /// The InstantiateMsg inherited from the base
    pub base: BaseSetConfigMsg,
}

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, _deps: DepsMut) -> StdResult<()> { Ok(()) }
}

/// The external execute calls that can be performed on an investment_anchor_earn.
/// These calls are private to be performed only by the vault that created it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InvestmentBase(InvestmentBaseExecuteMsg),

    /// Transaction to set the contract's config as well as the base config
    SetConfig { config_msg: SetConfigMsg }
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::InvestmentBase(base.into())
    }
}