use cosmwasm_std::{Addr, StdResult, DepsMut, CanonicalAddr};
use cosmwasm_std::{ Uint256 };
use cw20::Cw20ReceiveMsg;
use cw_storage_plus::{Item, PrimaryKey, Key, Prefixer, KeyDeserialize, Bound};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    execute_base::BaseExecuteMsg,
    base_config::{BaseSetConfigMsg, ConfigMsgTrait}, storage::STATE_KEY,
};

/// The hook messages sent with a CW20 token transfer. Used to verify the intention of the
/// sender is to deposit Basset tokens as collateral.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20ReceiveHookMsg {
    /// Deposit basset collateral tokens
    Deposit {},

    /// Message called by the Vault after Withdrawing to transfer bAsset back to the investor.
    TransferBassetToInvestor { investor_addr: Addr }
}

/// The external execute calls that can be performed on a Neptune registry.
/// These calls are private to be performed by depositors via the webapp.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    
    /// Message for sending CW20 tokens to the vault.
    Receive(Cw20ReceiveMsg),

    /// Message for removing a specific amount of shares from the vault.
    Withdraw { shares_amount: Uint256 },

    Freeze { },

    Unfreeze { },

    /// Liquidates the Vault fully and returns all funds to its investors
    RefundInvestors { },

    /// Transaction to set the contract's config as well as the base config
    SetConfig { config_msg: SetConfigMsg }
}

impl From<BaseExecuteMsg> for ExecuteMsg {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

/// The public queries that can be called on a Neptune registry.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetInvestorDetails { address: Addr },
    GetInvestorList { start_after: Option<Addr>, limit: Option<u32> },
    GetSharePrice {}
}

/// The instantiate message used to initialize a Neptune Registry and all it's parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

/// The SetConfig message used to initialize a Neptune Registry's config and all it's dependencies.
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

/// State variables for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub outstanding_shares: Uint256,
    pub outstanding_basset_principal: Uint256,
    pub is_frozen: bool,
}

pub const STATE: Item<State> = Item::new(STATE_KEY);

impl Default for State {
    fn default() -> Self {
        State {
            outstanding_shares: Uint256::zero(),
            outstanding_basset_principal: Uint256::zero(),
            is_frozen: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestorInfo {
    pub shares: Uint256,
    pub basset_principal: Uint256,
    pub last_tx_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestorDetailsResponse {
    pub investor: Addr,
    pub shares: Uint256,
    pub basset_principal: Uint256,
    pub last_tx_height: u64,
    pub basset_equity: Uint256,
    pub net_value: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestorListResponse {
    pub investor_list: Vec<InvestorListData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestorListData {
    pub investor: Addr,
    pub shares: Uint256,
    pub basset_principal: Uint256,
    pub last_tx_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, JsonSchema)]
pub struct InvestorAddr(pub CanonicalAddr);

/// type safe version to ensure address was validated before use.
/*
impl<'a> PrimaryKey<'a> for &'a InvestorAddr {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // this is simple, we don't add more prefixes
        vec![Key::Ref(self.as_ref().as_bytes())]
    }
}

impl<'a> Prefixer<'a> for &'a InvestorAddr {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl KeyDeserialize for &InvestorAddr {
    type Output = Addr;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Self::Output::from_vec(value)
    }
}*/

/// owned variant.
impl<'a> PrimaryKey<'a> for InvestorAddr {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // this is simple, we don't add more prefixes
        vec![Key::Ref(self.0.as_slice())]
    }
}

impl<'a> Prefixer<'a> for InvestorAddr {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.0.as_slice())]
    }
}

impl KeyDeserialize for InvestorAddr {
    type Output = InvestorAddr;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(CanonicalAddr::from(value).into())
    }
}

// conversions back to CanonicalAddr
impl From<CanonicalAddr> for InvestorAddr {
    fn from(addr: CanonicalAddr) -> Self {
        InvestorAddr(addr)
    }
}

impl Into<CanonicalAddr> for InvestorAddr {
    fn into(self) -> CanonicalAddr {
        self.0
    }
}

impl From<Vec<u8>> for InvestorAddr {
    fn from(array: Vec<u8>) -> Self {
        InvestorAddr(CanonicalAddr::from(array))
    }
}

impl<'a> Into<Bound<'a, InvestorAddr>> for InvestorAddr {
    fn into(self) -> Bound<'a, InvestorAddr> {
        Bound::exclusive(self.0.as_slice().to_vec())
    }
}
