use cosmwasm_std::{CanonicalAddr, DepsMut, StdResult};
use cw_storage_plus::Item;
use schemars::JsonSchema;

use serde::{
    Deserialize,
    Serialize
};
use terraswap::asset::AssetInfo;

//use crate::investment_base::InvestmentSetConfigMsg;
use crate::{
    base_config::{BaseSetConfigMsg, ConfigMsgTrait}, 
    investment::InvestmentBaseExecuteMsg, 
    execute_base::BaseExecuteMsg, 
    storage::{CONFIG_KEY, PARAMS_KEY}
};

/// Parameters for terraswap lp.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub asset_1: AssetInfo,
    pub asset_2: AssetInfo,
}

pub const PARAMS: Item<Params> = Item::new(PARAMS_KEY);

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub asset_1: AssetInfo,
    pub asset_2: AssetInfo,
}

/// Config variables for the base investment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The address for the liquidity pool contract
    pub lp_contract: Option<CanonicalAddr>,

    pub lp_token: Option<CanonicalAddr>,

    pub asset_1_stable_pool: Option<CanonicalAddr>,

    pub asset_2_stable_pool: Option<CanonicalAddr>,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SetConfigMsg {
    /// The InstantiateMsg inherited from the base
    pub base: BaseSetConfigMsg,

    /// The address for the liquidity pool contract
    pub lp_contract: String,

    pub lp_token: String,

    pub asset_1_stable_pool: String,

    pub asset_2_stable_pool: String,
}

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, deps: DepsMut) -> StdResult<()> {
        let config = Config {
            lp_contract: Some(deps.api.addr_canonicalize(self.lp_contract.as_str())?),
            lp_token: Some(deps.api.addr_canonicalize(self.lp_token.as_str())?),
            asset_1_stable_pool: Some(deps.api.addr_canonicalize(self.asset_1_stable_pool.as_str())?),
            asset_2_stable_pool: Some(deps.api.addr_canonicalize(self.asset_2_stable_pool.as_str())?),
        };
    
        CONFIG.save(deps.storage, &config)
    }
}

/// The external execute calls that can be performed on an investment_terraswap_lp.
/// These calls are private to be performed only by the vault that created it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InvestmentBase(InvestmentBaseExecuteMsg),
    ProvideLiquidity{ },
    ConvertAssetsToUst{ },

    /// Transaction to set the contract's config as well as the base config
    SetConfig { config_msg: SetConfigMsg }
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::InvestmentBase(base.into())
    }
}