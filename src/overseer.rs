use cosmwasm_std::{Addr, StdResult, DepsMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    execute_base::BaseExecuteMsg,
    base_config::{BaseSetConfigMsg, ConfigMsgTrait},
};

/// The external execute calls that can be performed on a Neptune registry.
/// These calls are private to be performed by depositors via the webapp.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    
    SetConfig { config_msg: SetConfigMsg },

    AddStrategy { strategy_info: StrategyInfo },

    RemoveStrategy { strategy_id: String },

    UpdateStatus { msg: UpdateStatus }
}

/// The public queries that can be called on a Neptune registry.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetStrategies {},
    GetStrategyInfo{ strategy_id: String } 
}

/// The instantiate message used to initialize a Neptune Registry and all it's parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SetConfigMsg {
    pub base: BaseSetConfigMsg
}

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, _deps: DepsMut) -> StdResult<()> { Ok(()) }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    Maintenance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentType {
    Custom,
    Production,
    Beta,
    Staging,
    Development,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateStatus {
    pub strategy_id: String,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StrategyInfo {
    pub strategy_id: String,
    pub deployment_profile: DeploymentType,
    pub collateral_profile: String,
    pub strategy_profile: String,
    pub sub_strategy_profile: Option<String>,
    pub status: Status,
    pub vault_addr: Addr,
    pub registry_addr: Addr,
    pub banker_addr: Addr,
    pub investment_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StrategyListResponse {
    pub strategy_list: Vec<StrategyInfo>,
}
