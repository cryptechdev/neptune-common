use cosmwasm_std::{ConversionOverflowError, Decimal256RangeExceeded, StdError};
use neptune_auth::error::NeptAuthError;
use thiserror::Error;

pub type CommonResult<T> = core::result::Result<T, CommonError>;

#[derive(Error, Debug, PartialEq)]
pub enum CommonError {
    #[error("{0}")]
    Generic(String),

    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Auth(#[from] NeptAuthError),

    #[error(transparent)]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error(transparent)]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Only one tx is allowed per block")]
    MultipleTx,

    #[error("Missing Cw20HookMg")]
    MissingHookMsg,
}
