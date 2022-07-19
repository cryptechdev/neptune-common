use std::fmt::Debug;
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize, 
};
use cosmwasm_std::{
    Env, MessageInfo,
    DepsMut, Response,
    Binary, to_binary,
    Deps, Addr, CosmosMsg, 
    WasmMsg, Coin, BankMsg, attr, Attribute,
};
use cosmwasm_std::Uint256;

use terraswap::asset::AssetInfo;

use crate::math::to_uint128;
// Neptune Package crate imports
use crate::{
    authorization::{
        BaseAuthorization,
        neptune_execute_authorize, NeptuneContractAuthorization
    },
    base_config::{
        ExternalContractsMsg, 
        ExternalContracts, 
        BaseConfig,
        store_base_config, 
        read_base_config, 
    },
    error::{NeptuneResult, NeptuneError},
    storage::{
        canonicalize_addresses,
    }, 
    querier::{query_balance, query_token_balance}, 
    warning::NeptuneWarning, 
    warn
};

/// The external execute calls that can be performed on any Neptune contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseExecuteMsg {
    SendFunds{ recipient: Addr, amount: Uint256, send_msg: SendFundsMsg, exec_msg: Option<Binary> },
    UpdateConfig { 
        revision: Option<String>, 
        vault: Option<String>, 
        admins: Option<Vec<String>>, 
        admin_double_sig: Option<String>,
        external_contracts: Option<ExternalContractsMsg> 
    },
}

/// Execute mutable operations on a Neptune vault.
pub fn base_execute<A: NeptuneContractAuthorization<SendFundsMsg>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseExecuteMsg,
) -> Result<Response, NeptuneError> {
    neptune_execute_authorize::<BaseExecuteMsg, BaseAuthorization>(
        deps.as_ref(), &env, &info.sender, &msg
    )?;

    match msg {
        BaseExecuteMsg::SendFunds{ recipient, amount, send_msg, exec_msg }=> {
            send_funds::<A>(
                deps.as_ref(), &env,
                &recipient,
                amount,
                send_msg,
                exec_msg
            )
        },
        BaseExecuteMsg::UpdateConfig { revision, vault, admins, admin_double_sig, external_contracts } => {
            update_base_config(
                deps,
                revision,
                vault, 
                admins, 
                admin_double_sig,
                external_contracts
            )?;
            Ok(Response::default())
        }
    }
}

/// The private messages for sending funds out of a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SendFundsMsg {
    SendCoins(String),
    SendTokens(Addr),
}

impl From<AssetInfo> for SendFundsMsg {
    fn from(asset: AssetInfo) -> Self {
        match asset {
            AssetInfo::NativeToken{denom} => SendFundsMsg::SendCoins(denom),
            AssetInfo::Token{contract_addr} => SendFundsMsg::SendTokens(Addr::unchecked(contract_addr)),
        }
    }
}

impl Into<AssetInfo> for SendFundsMsg {
    fn into(self) -> AssetInfo {
        match self {
            SendFundsMsg::SendCoins(denom) => AssetInfo::NativeToken{denom},
            SendFundsMsg::SendTokens(contract_addr) => AssetInfo::Token{contract_addr: contract_addr.into()},
        }
    }
}

fn send_funds<A: NeptuneContractAuthorization<SendFundsMsg>>(
    deps: Deps,
    env: &Env,
    recipient: &Addr,
    mut amount: Uint256,
    send_msg: SendFundsMsg,
    exec_msg: Option<Binary>
) -> Result<Response, NeptuneError> {
    neptune_execute_authorize::<SendFundsMsg,A>(deps, &env, &recipient, &send_msg)?;

    let mut attrs: Vec<Attribute> = vec![];

    let cosmos_msg = match send_msg {
        SendFundsMsg::SendCoins(denom) => {
            // Cap by our balance
            let coin_balance = query_balance(deps, &env.contract.address, denom.to_string())?;
            if amount > coin_balance {
                warn!(attrs, NeptuneWarning::InsuffBalance);
                amount = coin_balance;
            }
            if amount.is_zero() {return warn!(NeptuneWarning::AmountWasZero);}
            
            // Create the Coin array and either send coins or attach to a message
            let coins = vec![Coin {
                denom: denom.to_string(),
                amount: to_uint128(amount)?,
            }];
            match exec_msg {
                Some(binary) => attach_coins(coins, recipient, binary),
                None => send_coins(coins, recipient)
            }
        }
        SendFundsMsg::SendTokens(token_addr) => 
        {
            // Cap by our balance
            let token_balance = query_token_balance(deps, &token_addr, &env.contract.address)?;
            if amount > token_balance {
                warn!(attrs, NeptuneWarning::InsuffBalance);
                amount = token_balance;
            }
            if amount.is_zero() {return warn!(NeptuneWarning::AmountWasZero);}

            send_tokens(&token_addr, amount, exec_msg, recipient)?
        }
    };

    Ok(Response::new().add_message(cosmos_msg).add_attributes(attrs))
}

fn update_base_config(
    deps: DepsMut,
    revision: Option<String>,
    vault: Option<String>,
    admins: Option<Vec<String>>,
    admin_double_sig: Option<String>,
    external_contracts: Option<ExternalContractsMsg>
) -> NeptuneResult<()> {
    let mut config: BaseConfig = read_base_config(deps.storage)?;

    if let Some(r) = revision {
        config.revision = r;
    }

    if let Some(v) = vault {
        config.vault = Some(deps.api.addr_canonicalize(v.as_str())?);
    }

    if let Some(a) = admins {
        config.admins = Some(canonicalize_addresses(deps.as_ref(), &a)?);
    }

    if let Some(ads) = admin_double_sig {
        config.admin_double_sig = Some(deps.api.addr_canonicalize(ads.as_str())?);
    }

    if let Some(ec) = external_contracts {
        config.external_contracts = ExternalContracts::from(deps.as_ref(),&ec);
    }

    Ok(store_base_config(deps.storage, &config)?)
}

pub fn attach_coins(
    coins: Vec<Coin>,
    recipient_addr: &Addr,
    exec_msg: Binary,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: recipient_addr.to_string(),
        msg: exec_msg,
        funds: coins,
    })
}

pub fn send_coins(
    coins: Vec<Coin>,
    recipient_addr: &Addr,
) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: coins
    })
}

pub fn send_tokens(
    token_addr: &Addr,
    token_amount: Uint256,
    exec_msg: Option<Binary>,
    recipient_addr: &Addr,
) -> Result<CosmosMsg, NeptuneError> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        funds: vec![],
        msg: to_binary(
            &match exec_msg {
                Some(binary) => Cw20ExecuteMsg::Send {
                    contract: recipient_addr.to_string(),
                    amount: to_uint128(token_amount)?,
                    msg: binary,
                },
                None => Cw20ExecuteMsg::Transfer {
                    recipient: recipient_addr.to_string(),
                    amount: to_uint128(token_amount)?,
                },
            }
        )?
    }))
}