use cosmwasm_std::{CanonicalAddr, Timestamp, Env, DepsMut, StdResult};
use cosmwasm_std::{Decimal256, Uint256};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    execute_base::BaseExecuteMsg,
    base_config::{BaseSetConfigMsg, ConfigMsgTrait}, 
    storage::{canonicalize_addresses, CONFIG_KEY, PARAMS_KEY, STATE_KEY}, 
    signed_decimal::SignedDecimal,
};

/// The external execute calls that can be performed on a Neptune banker.
/// These calls are private to be performed only by the loopers and owners.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    
    /// Message for claiming the latest rewards from the vault's anchor loans.
    ClaimRewardsAndFees { },

    /// Message for re-balancing the investments, debts and collaterals in the vault.
    Rebalance { },

    /// Message for replenishing the looper with some liquid stable to perform the transactions above.
    ReplenishLooper { amount: Uint256 },

    /// Callback from the Vault every time there's a deposit or withdrawal
    FundsUpdateCallback { amount_to_deposit: Uint256, fraction_to_withdraw: Decimal256 },

    /// Transaction to set the contract's config as well as the base config
    SetConfig { config_msg: SetConfigMsg },
    SetParams { params_msg: InstantiateMsg },
    AddLooper { address: String },
    RemoveLooper { address: String }
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

/// The public queries that can be called on a Neptune banker.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetState {},
    GetParams {},
    GetBalances {},
    GetBalanceValues {},
    GetTvl { include_unclaimed_rewards: bool },
    GetLtvRatio { anchor_pricing: bool },
    GetRequiresRebalancing {},
    GetBlocksSinceLastRebalance {},
    GetProfitSinceLastClaim {},
    GetApr {},
    GetApyDetails { },
    GetPendingRewardsValue {},
    GetPendingBassetRewardsValue {},
    GetPendingAncRewardsValue {},
    GetPendingInvestmentRewardsValue {},
    GetInvestmentMetrics {},
}

/// Parameters for a Neptune banker.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub fee_profit_rate: Decimal256,
    pub fee_min_rate: Decimal256,
    pub fee_avg_attenuation: u64,
    pub staking_ratio_min: Decimal256,
    pub staking_ratio_max: Decimal256,
    pub staking_ratio_delta: Decimal256,
    pub max_replenish_looper: Uint256,
    pub replenish_looper_period: Uint256,
}

pub const PARAMS: Item<Params> = Item::new(PARAMS_KEY);

/// The instantiate message used to initialize a Neptune Banker and all it's parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fee_profit_rate: Decimal256,
    pub fee_min_rate: Decimal256,
    pub fee_avg_attenuation: u64,
    pub staking_ratio_min: Decimal256,
    pub staking_ratio_max: Decimal256,
    pub staking_ratio_delta: Decimal256,
    pub max_replenish_looper: Uint256,
    pub replenish_looper_period: Uint256,
}

/// Config variables for a Neptune banker.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The looper addresses
    pub loopers: Option<Vec<CanonicalAddr>>,

    /// The looper manager address
    pub looper_manager: Option<CanonicalAddr>,

    /// The list of addresses that are authorized to claim rewards/fees and re-balance.
    pub fee_wallet: Option<CanonicalAddr>,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

/// The instantiate message used to initialize a Neptune Banker's config and all it's dependencies.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SetConfigMsg {
    pub base: BaseSetConfigMsg,

    /// The address for the wallet to which the fees need to go.
    pub fee_wallet: String,

    /// The list of addresses that are authorized to claim rewards/fees and re-balance.
    pub loopers: Vec<String>,

    /// The looper manager address
    pub looper_manager: String,
}

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, deps: DepsMut) -> StdResult<()> {
        let config = Config {
            loopers: Some(canonicalize_addresses(deps.as_ref(), &self.loopers)?),
            looper_manager: Some(deps.api.addr_canonicalize(&self.looper_manager.as_str())?),
            fee_wallet: Some(deps.api.addr_canonicalize(&self.fee_wallet.as_str())?),
        };
        CONFIG.save(deps.storage, &config)
    }
}

/// State variables for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub previous_basset_net_worth: Uint256,
    pub profit_apr_moving_avg: SignedDecimal,
    pub previous_basset_staking_apr: Decimal256,

    pub time_last_claim: Timestamp,

    pub time_last_replenish_looper: Timestamp,
    pub amount_replenished_to_looper: Uint256,
}

pub const STATE: Item<State> = Item::new(STATE_KEY);

impl State {
    pub fn default(env: &Env) -> Self {
        State {
            previous_basset_net_worth: Uint256::zero(),
            profit_apr_moving_avg: SignedDecimal::nan(),
            previous_basset_staking_apr: Decimal256::zero(),
            
            time_last_claim: env.block.time,

            time_last_replenish_looper: env.block.time,
            amount_replenished_to_looper: Uint256::zero(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApyDetails {
    pub terraswap_ltv_ratio: Decimal256,
    pub anchor_ltv_ratio: Decimal256,
    pub staking_ratio: Decimal256,
    pub borrow_rate: Decimal256,
    pub rewards_rate: Decimal256,
    pub raw_strategy_rate: Decimal256,
    pub net_strategy_rate: Decimal256,
    pub basset_staking_rate: Decimal256,
    pub combined_rate: Decimal256,
    pub fee_rate: Decimal256,
    pub vault_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingRewardsValue {
    pub total_pending_rewards_value: Uint256,
    pub anc_pending_rewards_value: Uint256,
    pub investment_pending_rewards_value: Uint256,
    pub staking_pending_rewards_value: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProfitSinceLastClaim {
    pub anchor_basset_price: Decimal256,
    pub terraswap_basset_price: Decimal256,
    pub profit_since_last_claim: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestmentMetrics {
    pub investment_asset_price: Decimal256,
    pub investment_pending_rewards: Uint256,
    pub investment_redeemable: Uint256,
}

