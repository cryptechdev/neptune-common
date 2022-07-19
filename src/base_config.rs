use std::fmt::Debug;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
};
use cosmwasm_std::{
    DepsMut, Deps, Addr, StdResult,
    CanonicalAddr, Storage,
};
use terraswap::asset::{AssetInfo};


// Neptune Package crate imports
use crate::{
    error::{NeptuneError}, 
    storage::{
        BASE_OWNER_KEY, 
        BASE_CONFIG_KEY,
        canonicalize_addresses, 
        get_contract_addr, 
        humanize_addresses, 
        get_config_string, 
        canonicalize_address,
    }
};

/// Struct for all the external contract addresses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExternalContracts {
    /// The addresses for the different anchor contracts
    pub anchor_market: Option<CanonicalAddr>,
    pub anchor_overseer: Option<CanonicalAddr>,
    pub anchor_oracle: Option<CanonicalAddr>,
    pub anchor_custody: Option<CanonicalAddr>,
    pub anchor_interest_model: Option<CanonicalAddr>,
    pub anchor_aust: Option<CanonicalAddr>,
    pub basset_rewards_contract: Option<CanonicalAddr>,

    /// The addresses for the different token contracts
    pub anc_token: Option<CanonicalAddr>,
    pub basset_token: Option<CanonicalAddr>,
    pub stable_asset_info: Option<AssetInfo>,

    /// The addresses for the different token bools
    pub anc_pool: Option<CanonicalAddr>,
    pub stable_asset_pool: Option<CanonicalAddr>,
    pub asset_basset_pool: Option<CanonicalAddr>,
    pub stable_basset_pool: Option<CanonicalAddr>,

    /// The name of the asset
    pub asset_denom: Option<String>,
}

/// Struct for all the external contract addresses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExternalContractsMsg {
    /// The addresses for the different anchor contracts
    pub anchor_market: String,
    pub anchor_overseer: String,
    pub anchor_oracle: String,
    pub anchor_custody: String,
    pub anchor_interest_model: String,
    pub anchor_aust: String,
    pub basset_rewards_contract: String,

    /// The addresses for the different token contracts
    pub anc_token: String,
    pub basset_token: String,
    pub stable_asset_info: AssetInfo,


    /// The addresses for the different token bools
    pub anc_pool: String,
    pub stable_asset_pool: String,
    pub asset_basset_pool: String,
    pub stable_basset_pool: String,

    /// The name of the asset
    pub asset_denom: String,
}

impl ExternalContracts {
    pub fn from(deps: Deps, ecm: &ExternalContractsMsg) -> Self {
        Self {
            anchor_market:            deps.api.addr_canonicalize(ecm.anchor_market.as_str()).ok(),
            anchor_overseer:          deps.api.addr_canonicalize(ecm.anchor_overseer.as_str()).ok(),
            anchor_oracle:            deps.api.addr_canonicalize(ecm.anchor_oracle.as_str()).ok(),
            anchor_custody:           deps.api.addr_canonicalize(ecm.anchor_custody.as_str()).ok(),
            anchor_interest_model:    deps.api.addr_canonicalize(ecm.anchor_interest_model.as_str()).ok(),
            anchor_aust:              deps.api.addr_canonicalize(ecm.anchor_aust.as_str()).ok(),
            basset_rewards_contract:  deps.api.addr_canonicalize(ecm.basset_rewards_contract.as_str()).ok(),
            anc_token:                deps.api.addr_canonicalize(ecm.anc_token.as_str()).ok(),
            basset_token:             deps.api.addr_canonicalize(ecm.basset_token.as_str()).ok(),
            stable_asset_info:        Some(ecm.stable_asset_info.clone()),
            anc_pool:                 deps.api.addr_canonicalize(ecm.anc_pool.as_str()).ok(),
            stable_asset_pool:           deps.api.addr_canonicalize(ecm.stable_asset_pool.as_str()).ok(),
            asset_basset_pool:        deps.api.addr_canonicalize(ecm.asset_basset_pool.as_str()).ok(),
            stable_basset_pool:          deps.api.addr_canonicalize(ecm.stable_basset_pool.as_str()).ok(),
            asset_denom:              Some(ecm.asset_denom.clone()),
        }
    }
}

impl  Default for ExternalContracts {
    fn default() -> Self {
        Self { 
            anchor_market: None,
            anchor_overseer: None,
            anchor_oracle: None,
            anchor_custody: None,
            anchor_interest_model: None,
            anchor_aust: None,
            basset_rewards_contract: None,
            anc_token: None,
            basset_token: None,
            stable_asset_info: None,
            anc_pool: None,
            stable_asset_pool: None,
            asset_basset_pool: None,
            stable_basset_pool: None,
            asset_denom: None 
        }
    }
}

/// Config variables for a Neptune vault.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseConfig {
    /// The hash for the commit at the time of instantiation or migration
    pub revision: String,

    /// Address of the vault
    pub vault: Option<CanonicalAddr>,

    /// The list of addresses that are authorized to access admin functionality.
    pub admins: Option<Vec<CanonicalAddr>>,

    /// Double sig admin address
    pub admin_double_sig: Option<CanonicalAddr>,

    /// The set of external contracts
    pub external_contracts: ExternalContracts,
}

impl BaseConfig {
    pub fn from_msg(deps: Deps, msg: &BaseSetConfigMsg) -> StdResult<Self> {
        Ok(BaseConfig {
            revision: msg.revision.clone(),
            vault: canonicalize_address(deps, &msg.vault)?,
            admins: Some(canonicalize_addresses(deps, &msg.admins)?),
            admin_double_sig: canonicalize_address(deps, &msg.admin_double_sig)?,
            external_contracts: ExternalContracts::from(deps,&msg.external_contracts)
        })
    }

    pub fn default(deps: Deps) -> StdResult<Self> {
        Ok(BaseConfig {  
            revision: String::default(),
            vault: None,
            admins: Some(vec![BASE_OWNER.load(deps.storage)?]),
            admin_double_sig: None,
            external_contracts: ExternalContracts::default() 
        })
    }
}

pub const BASE_OWNER: Item<CanonicalAddr> = Item::new(BASE_OWNER_KEY);
pub const BASE_CONFIG: Item<BaseConfig> = Item::new(BASE_CONFIG_KEY);

pub trait ConfigMsgTrait {
    fn get_base_config_msg(&self) -> &BaseSetConfigMsg;
    fn set_config(&self, deps: DepsMut) -> StdResult<()>;
}

/// Instantiate message common to all contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BaseSetConfigMsg {
    /// The hash for the commit at the time of instantiation or migration
    pub revision: String,

    /// Address of the vault
    pub vault: String,

    /// The list of addresses that are authorized to access admin functionality.
    pub admins: Vec<String>,

    pub admin_double_sig: String,

    /// The set of external contracts
    pub external_contracts: ExternalContractsMsg,
}

pub fn stringify_optional_addr(deps: Deps, option: Option<CanonicalAddr>) -> StdResult<String> {
    Ok(if let Some(canon_addr) = option{
        deps.api.addr_humanize(&canon_addr)?.to_string()
    } else { String::from("None") })
}

impl BaseSetConfigMsg {
    pub fn from_config(deps: Deps, config: BaseConfig) -> StdResult<Self> {
        let admins = if let Some(a) = config.admins {
            humanize_addresses(deps, &a)?.iter().map(|a| a.to_string()).collect()
        } else { vec![] };

        let ecm = config.external_contracts;

        Ok(BaseSetConfigMsg {
            revision: config.revision,
            vault: stringify_optional_addr(deps, config.vault)?,
            admins,
            admin_double_sig: stringify_optional_addr(deps, config.admin_double_sig)?,
            external_contracts: ExternalContractsMsg {
                anchor_market:            stringify_optional_addr(deps, ecm.anchor_market        )?,
                anchor_overseer:          stringify_optional_addr(deps, ecm.anchor_overseer      )?,
                anchor_oracle:            stringify_optional_addr(deps, ecm.anchor_oracle        )?,
                anchor_custody:           stringify_optional_addr(deps, ecm.anchor_custody       )?,
                anchor_interest_model:    stringify_optional_addr(deps, ecm.anchor_interest_model)?,
                anchor_aust:              stringify_optional_addr(deps, ecm.anchor_aust          )?,
                basset_rewards_contract:  stringify_optional_addr(deps, ecm.basset_rewards_contract)?,
                anc_token:                stringify_optional_addr(deps, ecm.anc_token            )?,
                basset_token:             stringify_optional_addr(deps, ecm.basset_token         )?,
                stable_asset_info:        ecm.stable_asset_info.unwrap(),
                anc_pool:                 stringify_optional_addr(deps, ecm.anc_pool             )?,
                stable_asset_pool:           stringify_optional_addr(deps, ecm.stable_asset_pool       )?,
                asset_basset_pool:        stringify_optional_addr(deps, ecm.asset_basset_pool    )?,
                stable_basset_pool:          stringify_optional_addr(deps, ecm.stable_basset_pool      )?,
                asset_denom:              ecm.asset_denom.or(Some(String::from("None"))).unwrap(),
            }
        })
    }
}

/// A code sharing function to set the values of all the config variables during either
/// contract instantiation or migration.
pub fn set_config_from_msg<M: ConfigMsgTrait>(deps: DepsMut, msg: M) -> StdResult<()> {
    let config = BaseConfig::from_msg(deps.as_ref(), msg.get_base_config_msg())?;
    store_base_config(deps.storage, &config)?;
    msg.set_config(deps)
}

pub fn set_default_base_config(deps: DepsMut) -> StdResult<()> {
    let config = BaseConfig::default(deps.as_ref())?;
    store_base_config(deps.storage, &config)
}

pub fn store_base_config(storage: &mut dyn Storage, data: &BaseConfig) -> StdResult<()> {
    BASE_CONFIG.save(storage, &data)
}

pub fn read_base_config(storage: &dyn Storage) -> StdResult<BaseConfig> {
    BASE_CONFIG.load(storage)
}

pub fn set_owner_address(deps: DepsMut, addr: Addr) -> StdResult<()> {
    let canon_addr = deps.api.addr_canonicalize(addr.as_str())?;
    BASE_OWNER.save(deps.storage, &canon_addr)
}

pub fn get_owner_address(deps: Deps) -> StdResult<Addr> {
    let canon_addr = BASE_OWNER.load(deps.storage)?;
    deps.api.addr_humanize(&canon_addr)
}

pub fn get_admin_double_sig_address(deps: Deps) -> Result<Option<Addr>, NeptuneError> {
    let config = read_base_config(deps.storage)?;
    let admin_double_sig = &config.admin_double_sig;
    if let Some(addr) = admin_double_sig {
        Ok(Some(deps.api.addr_humanize(addr)?))
    } else {
        Ok(None)
    }
}

pub fn get_vault_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Neptune Vault", &read_base_config(deps.storage)?.vault)
}

pub fn get_admin_addresses(deps: Deps) -> Result<Vec<Addr>, NeptuneError> {
    let config = read_base_config(deps.storage)?;
    let admin_list = &config.admins.ok_or(NeptuneError::MissingAdminAddresses{})?;
    Ok(humanize_addresses(deps, admin_list)?)
}

pub fn get_anchor_market_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor Market", &read_base_config(deps.storage)?.external_contracts.anchor_market)
}

pub fn get_anchor_overseer_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor Overseer", &read_base_config(deps.storage)?.external_contracts.anchor_overseer)
}

pub fn get_anchor_oracle_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor Oracle", &read_base_config(deps.storage)?.external_contracts.anchor_oracle)
}

pub fn get_anchor_custody_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor Custody", &read_base_config(deps.storage)?.external_contracts.anchor_custody)
}

pub fn get_anchor_interest_model_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor Interest Model", &read_base_config(deps.storage)?.external_contracts.anchor_interest_model)
}

pub fn get_anchor_aust_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Anchor aUST", &read_base_config(deps.storage)?.external_contracts.anchor_aust)
}

pub fn get_anc_token_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "ANC Token", &read_base_config(deps.storage)?.external_contracts.anc_token)
}

pub fn get_basset_token_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "bAsset Token", &read_base_config(deps.storage)?.external_contracts.basset_token)
}

pub fn get_anc_pool(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "ANC Pool", &read_base_config(deps.storage)?.external_contracts.anc_pool)
}

pub fn get_stable_asset_pool(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Asset Pool", &read_base_config(deps.storage)?.external_contracts.stable_asset_pool)
}

pub fn get_asset_basset_pool(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "bAsset Pool", &read_base_config(deps.storage)?.external_contracts.asset_basset_pool)
}

pub fn get_stable_basset_pool(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "bAsset Pool", &read_base_config(deps.storage)?.external_contracts.stable_basset_pool)
}

pub fn get_basset_rewards_contract(deps: Deps) -> Result<Addr, NeptuneError> {
    get_contract_addr(deps, "Basset Rewards Contract", &read_base_config(deps.storage)?.external_contracts.basset_rewards_contract)
}

pub fn get_asset_denom(deps: Deps) -> Result<String, NeptuneError> {
    get_config_string(read_base_config(deps.storage)?.external_contracts.asset_denom)
}

pub fn get_stable_asset(deps: Deps) -> Result<AssetInfo, NeptuneError> {
    Ok(read_base_config(deps.storage)?.external_contracts.stable_asset_info.unwrap())
}