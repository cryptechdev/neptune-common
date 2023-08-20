use cosmwasm_std::{ConversionOverflowError, Decimal256RangeExceeded, StdError, CheckedFromRatioError};
use neptune_auth::error::NeptAuthError;
use thiserror::Error;

use crate::asset::AssetInfo;

pub type NeptuneResult<T> = core::result::Result<T, NeptuneError>;

#[derive(Error, Debug, PartialEq)]
pub enum NeptuneError {
    #[error("{0}")]
    Generic(String),

    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Auth(#[from] NeptAuthError),

    #[error(transparent)]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[error(transparent)]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error(transparent)]
    CheckedFromRatio(#[from] CheckedFromRatioError),
    
    #[error("liquidity pool not found {0:?}")]
    PoolNotFound([AssetInfo; 2]),

    #[error("Insufficient liquidity to execute swap")]
    InsufficientLiquidity,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Only one tx is allowed per block")]
    MultipleTx,

    #[error("Missing Cw20HookMg")]
    MissingHookMsg,
}
