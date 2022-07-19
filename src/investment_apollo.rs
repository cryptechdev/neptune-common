use cosmwasm_std::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, StdResult, DepsMut};
use cw_storage_plus::Item;
use schemars::JsonSchema;

use serde::{
    Deserialize,
    Serialize
};
use terraswap::asset::AssetInfo;

//use crate::investment_base::InvestmentSetConfigMsg;
use crate::{
    base_config::{
        BaseSetConfigMsg, 
        ConfigMsgTrait
    }, 
    investment::InvestmentBaseExecuteMsg, 
    execute_base::{BaseExecuteMsg}, storage::{CONFIG_KEY, PARAMS_KEY}
};

/// Parameters for Apollo.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub strategy_id: u64,
    pub asset_infos: [AssetInfo; 2],
    pub earn_ratio: Decimal256,
}

pub const PARAMS: Item<Params> = Item::new(PARAMS_KEY);

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub strategy_id: u64,
    pub asset_infos: [AssetInfo; 2],
    pub earn_ratio: Decimal256,
}

/// Config variables for the base investment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub apollo_factory: Option<CanonicalAddr>,

    pub apollo_oracle: Option<CanonicalAddr>,

    pub lp_contract: Option<CanonicalAddr>,

    pub lp_token: Option<CanonicalAddr>,

    pub asset_0_stable_pool: Option<CanonicalAddr>,

    pub asset_1_stable_pool: Option<CanonicalAddr>,

    pub apollo_stable_pool: Option<CanonicalAddr>,

    pub apollo_token: Option<CanonicalAddr>,
}

/// Instantiate message for the anchor earn investment contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SetConfigMsg {
    /// The InstantiateMsg inherited from the base
    pub base: BaseSetConfigMsg,

    pub apollo_factory: String,

    pub apollo_oracle: String,

    pub lp_contract: String,

    pub lp_token: String,

    pub asset_0_stable_pool: String,

    pub asset_1_stable_pool: String,

    pub apollo_stable_pool: String,

    pub apollo_token: String,

}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, deps: DepsMut) -> StdResult<()> {
        let config = Config {
            apollo_oracle: Some(deps.api.addr_canonicalize(self.apollo_oracle.as_str())?),
            apollo_factory: Some(deps.api.addr_canonicalize(self.apollo_factory.as_str())?),
            lp_contract: Some(deps.api.addr_canonicalize(self.lp_contract.as_str())?),
            lp_token: Some(deps.api.addr_canonicalize(self.lp_token.as_str())?),
            asset_0_stable_pool: Some(deps.api.addr_canonicalize(self.asset_0_stable_pool.as_str())?),
            asset_1_stable_pool: Some(deps.api.addr_canonicalize(self.asset_1_stable_pool.as_str())?),
            apollo_stable_pool: Some(deps.api.addr_canonicalize(self.apollo_stable_pool.as_str())?),
            apollo_token: Some(deps.api.addr_canonicalize(self.apollo_token.as_str())?),
        };
        CONFIG.save(deps.storage, &config)
    }
}

/// The external execute calls that can be performed on an investment_apollo.
/// These calls are private to be performed only by the vault that created it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InvestmentBase(InvestmentBaseExecuteMsg),
    
    /// Transaction to set the contract's config as well as the base config
    SetConfig { config_msg: SetConfigMsg },
    SetParams { params_msg: InstantiateMsg },
    ProvideLiquidity{ },
    WithdrawLiquidity{ },
    DepositLpTokens{ },
    WithdrawLpTokens{ amount: Uint256 },
    ConvertUstToAssets{ },
    ConvertAssetsToUst{ },
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::InvestmentBase(base.into())
    }
}