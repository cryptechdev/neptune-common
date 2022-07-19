use std::{fmt::Debug};
use cosmwasm_std::{Deps, Addr, Env,};
use crate::{
    error::NeptuneError,
    execute_base::BaseExecuteMsg,
    base_config::{
        get_owner_address,
        get_vault_contract,
        get_admin_addresses, 
        get_anchor_custody_contract, 
        get_anchor_market_contract, 
        get_anc_pool, 
        get_stable_asset_pool, 
        get_asset_basset_pool,
        get_stable_basset_pool, get_admin_double_sig_address,
    },
};

pub type PermissionGroup = Vec<Addr>;

pub trait GetPermissionGroup: Debug {
    fn get_permission_group(&self, deps: Deps, env: &Env) -> Result<PermissionGroup, NeptuneError>;
}

pub type PermissionGroupList<'a> = Vec<&'a dyn GetPermissionGroup>;

#[derive(Clone, Debug)]
pub enum BasePermissionGroups {
    Internal,
    Vault,
    Admins,
    AdminDoubleSig,
    AdminTripleSig,
    Public,
    Anchor,
    TerraSwap
}

impl GetPermissionGroup for BasePermissionGroups {
    fn get_permission_group(&self, deps: Deps, env: &Env) -> Result<PermissionGroup, NeptuneError> {

        Ok(match self {
            Self::Internal          => vec![env.contract.address.clone()],
            Self::Vault             => vec![get_vault_contract(deps)?],
            Self::AdminTripleSig    => vec![get_owner_address(deps)?],
            Self::AdminDoubleSig    => {
                let mut vec = vec![get_owner_address(deps)?];
                if let Some(addr) = get_admin_double_sig_address(deps)? {vec.push(addr)}
                vec
            },
            Self::Admins            => {
                let mut admins = get_admin_addresses(deps)?;
                if let Some(addr) = get_admin_double_sig_address(deps)? { admins.push(addr) };
                admins.push(get_owner_address(deps)?);
                admins
            },
            Self::Public            => vec![],
            Self::Anchor => vec![
                get_anchor_custody_contract(deps)?,
                get_anchor_market_contract(deps)?
            ],
            Self::TerraSwap => vec![
                get_anc_pool(deps)?,
                get_stable_asset_pool(deps).or(get_stable_basset_pool(deps))?,
                get_asset_basset_pool(deps).or(get_stable_basset_pool(deps))?,
            ]
        })
    }
}

pub trait NeptuneContractAuthorization<M> {
    fn permissions(msg: &M) -> Result<PermissionGroupList, NeptuneError>;
}

/// Structure to pass the base authorization levels for a global permissions check on all executes
#[derive(Copy, Clone)]
pub struct BaseAuthorization {}

impl NeptuneContractAuthorization<BaseExecuteMsg> for BaseAuthorization {

    /// Schema for the execute permissions of the vault contract
    fn permissions(msg: &BaseExecuteMsg) -> Result<PermissionGroupList, NeptuneError> {

        use BasePermissionGroups::*;
        Ok(match msg {
            BaseExecuteMsg::SendFunds{ .. } => vec![&Internal],
            BaseExecuteMsg::UpdateConfig { .. } => vec![&AdminTripleSig],
        })
    }
}

pub fn neptune_execute_authorize<M, A: NeptuneContractAuthorization<M>>(
    deps: Deps,
    env: &Env,
    address: &Addr,
    message: &M,
) -> Result<(), NeptuneError> {

    #[cfg(neptune_test)] {
        return Ok(());
    }
    let permission_result = A::permissions( message);

    match permission_result {
        Ok(p) => authorize_permissions(deps.clone(), env, address, &p),
        Err(e) => panic!("Authorization error: {:?}", e),
    }
}

pub fn authorize_permissions(
    deps: Deps,
    env: &Env,
    addr: &Addr,
    permissions: &PermissionGroupList,
) -> Result<(), NeptuneError> {
    let collected_permissions: Result<Vec<PermissionGroup>, NeptuneError> = permissions.iter()
    .map(|x| x.get_permission_group(deps, env))
    .collect();

    let flattened : PermissionGroup = collected_permissions?.into_iter().flatten().collect();
    
    let authorized = flattened.is_empty() || flattened.iter().any(|i| *i == *addr);
    if authorized { 
        Ok(()) 
    }
    else { 
        Err(NeptuneError::Unauthorized(format!("Unauthorized execution: {} is not {:?}", *addr, permissions))) 
    }
}
