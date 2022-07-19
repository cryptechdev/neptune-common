use std::ops::Mul;

// Cosmos and Terra imports
use cosmwasm_std::{Addr, CanonicalAddr, DepsMut, StdResult, Env, Fraction };
use cosmwasm_std::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Neptune Package crate imports
use crate::{
    error::{
        NeptuneError, 
        NeptuneResult
    },
    execute_base::{BaseExecuteMsg},
    math::get_difference_or_error,
    base_config::{
        BaseSetConfigMsg, ConfigMsgTrait,
    }, 
    storage::{CONFIG_KEY, PARAMS_KEY, STATE_KEY},
};

/// The external execute calls that can be performed on a Neptune vault.
/// These calls are private to be performed either by the Banker contract for Rebalance and
/// ClaimAncRewards, or the Registry contract for Receive and RemoveBassets.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),

    /// Message for claiming the latest rewards from the vault's anchor loans.
    ClaimRewardsAndFees { stable_fee: Uint256 },

    SendFee { amount: Uint256 },

    /// Message for re-balancing the investments, debts and collaterals in the vault.
    Rebalance { },

    AssertRebalance { current_balances: Balances, target_balances: Balances },

    /// Message for sending CW20 tokens to the vault.
    Receive(Cw20ReceiveMsg),

    /// Message for removing a specific amount of bassets from the vault.
    Withdraw { investor_address: Addr, fraction: Decimal256 },

    // Admin tx
    SetStakingRatio { staking_ratio: Decimal256 },
    SetConfig { config_msg: SetConfigMsg },
    SetParams { params_msg: InstantiateMsg },
    Liquidate { fraction: Decimal256 },
    SuspendAndLiquidate { },
    Suspend { },
    Resume { },

    // Private tx
    ResetState { },
    IncreaseCollateral { amount: Uint256 },
    DecreaseCollateral { amount: Uint256 },
    IncreaseInvestment { amount: Uint256 },
    DecreaseInvestment { amount: Uint256 },
    IncreaseDebt { target_debt: Uint256 },
    DecreaseDebt { amount: Uint256 },
    ConvertUstToBasset { amount: Uint256 },
    ConvertAllUstToBasset { reserve_stable: Uint256 },
    ConvertBassetToUst { amount: Uint256 },
    RepayDebtFromInvestment { target_debt: Uint256, target_investment: Uint256, target_liquid_stable: Uint256, stable_to_convert: Uint256, max_loops: u32 },
    RepayDebtFromCollateral { target_debt: Uint256, target_collateral: Uint256, target_liquid_basset: Uint256, convert_max_basset_to_stable: bool, max_loops: u32 },
    SendBAssetToRegistry { investor_addr: Addr, reserve_basset: Uint256 }
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

/// The hook messages sent with a CW20 token transfer. Used to verify the intention of the
/// sender is to deposit Basset tokens as collateral.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20ReceiveHookMsg {
    /// Deposit basset collateral tokens
    Deposit {},
}

/// The public queries that can be called on a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetState {},
    GetParams {},
    GetBalances {},
    GetTvl { include_unclaimed_rewards: bool },
    GetLtvRatio { anchor_pricing: bool },
    GetRequiresRebalancing {},
    GetBlocksSinceLastRebalance {},
    GetIsFrozen {},
    GetInvestmentValue {},
    GetInvestmentPendingRewardsValue {},
}

/// The balances that are held by the vault
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balances {
    pub collateral_basset : Uint256,
    pub debt_stable :          Uint256,
    pub investment_stable :    Uint256,
    pub liquid_stable :        Uint256,
    pub liquid_basset :     Uint256
}

pub type BalanceValues = Balances;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct TvlResponse {
    pub tvl_basset:   Uint256,
    pub tvl_stable:      Uint256,
    pub basset_price: Decimal256
}

impl Mul<Decimal256> for Balances {
    type Output = Self;
    fn mul(self, rhs: Decimal256) -> Self::Output {
        Balances {
            collateral_basset : self.collateral_basset  * rhs,
            debt_stable :          self.debt_stable           * rhs,
            investment_stable :    self.investment_stable     * rhs,
            liquid_stable :        self.liquid_stable         * rhs,
            liquid_basset :     self.liquid_basset      * rhs,
        }
    }
}

impl Balances {

    pub fn get_total_net_worth_as_basset(&self, basset_price: Decimal256) -> NeptuneResult<Uint256> {
        match basset_price.inv() {
            Some(basset_price_inv) => get_difference_or_error(
                self.collateral_basset + self.liquid_basset
                    + (self.investment_stable + self.liquid_stable) * basset_price_inv
                ,
                self.debt_stable * basset_price_inv,
                "ERROR: total_net_worth is negative".to_string()
            ),
            None => Err(NeptuneError::BassetPriceIsZero {}),
        }
    }

    pub fn get_total_net_worth_as_stable(&self, basset_price: Decimal256) -> NeptuneResult<Uint256> {
        get_difference_or_error(
            (self.collateral_basset + self.liquid_basset) * basset_price
                + self.investment_stable + self.liquid_stable
            ,
            self.debt_stable,
            "ERROR: total_net_worth is negative".to_string()
        )
    }

    pub fn get_balance_values(&self, basset_price: Decimal256) -> NeptuneResult<BalanceValues> {
        Ok(BalanceValues {
            collateral_basset : self.collateral_basset * basset_price,
            debt_stable :          self.debt_stable,
            investment_stable :    self.investment_stable,
            liquid_stable :        self.liquid_stable,
            liquid_basset :     self.liquid_basset * basset_price,
        })
    }
}

/// Parameters for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub max_price_drop             : Decimal256,
    pub buffer_debt_upper          : Decimal256,
    pub buffer_debt_lower          : Decimal256,
    pub buffer_investment_upper    : Decimal256,
    pub buffer_investment_lower    : Decimal256,
    pub buffer_basset              : Decimal256,
    pub buffer_liquidity           : Decimal256,
    pub max_loops                  : u32,
}

pub const PARAMS: Item<Params> = Item::new(PARAMS_KEY);

/// The instantiate message used to initialize a Neptune Vault and all it's parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub max_price_drop             : Decimal256,
    pub buffer_debt_upper          : Decimal256,
    pub buffer_debt_lower          : Decimal256,
    pub buffer_investment_upper    : Decimal256,
    pub buffer_investment_lower    : Decimal256,
    pub buffer_basset              : Decimal256,
    pub buffer_liquidity           : Decimal256,
    pub max_loops                  : u32,
}

/// Config variables for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The address of the Neptune investment contract,
    pub investment_contract: Option<CanonicalAddr>,

    /// The address that is authorized to deposit and withdraw from the vault directly.
    pub registry_contract: Option<CanonicalAddr>,

    /// The address that is authorized to claim rewards/fees and re-balance.
    pub banker_contract: Option<CanonicalAddr>,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

/// The SetConfig message used to initialize a Neptune Vault's config and all it's dependencies.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SetConfigMsg {
    pub base: BaseSetConfigMsg,

    /// The address of the Neptune investment contract,
    pub investment_contract: String,

    /// The list of addresses that are authorized to deposit and withdraw from the vault directly.
    pub registry_contract: String,

    /// The list of addresses that are authorized to claim rewards/fees and re-balance.
    pub banker_contract: String,
}

impl ConfigMsgTrait for SetConfigMsg {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg {
        &self.base
    }

    fn set_config(&self, deps: DepsMut) -> StdResult<()> {
        let config = Config {
            investment_contract: Some(deps.api.addr_canonicalize(self.investment_contract.as_str())?),
            registry_contract: Some(deps.api.addr_canonicalize(self.registry_contract.as_str())?),
            banker_contract: Some(deps.api.addr_canonicalize(self.banker_contract.as_str())?),
        };
    
        CONFIG.save(deps.storage, &config)
    }
}

/// State variables for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub staking_ratio: Decimal256,
    pub last_rebalance_block_height: u64,
    pub is_frozen: bool,
}

pub const STATE: Item<State> = Item::new(STATE_KEY);

impl State {
    pub fn default(env: &Env) -> Self {
        State {
            staking_ratio: Decimal256::zero(),
            last_rebalance_block_height: env.block.height,
            is_frozen: false,
        }
    }
}

