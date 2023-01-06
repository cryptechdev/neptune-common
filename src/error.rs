use cosmwasm_std::{ConversionOverflowError, Decimal256RangeExceeded, StdError};
use neptune_auth::error::NeptAuthError;
use thiserror::Error;

pub type CommonResult<T> = core::result::Result<T, CommonError>;

#[derive(Error, Debug, PartialEq)]
pub enum CommonError {
    #[error("{0}")]
    Error(String),

    #[error("{0}")]
    Generic(String),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Auth(#[from] NeptAuthError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Only one tx is allowed per block")]
    MultipleTx {},

    #[error("Denominator was zero")]
    ZeroDenominator {},

    #[error("Basset price was returned as zero")]
    BassetPriceIsZero {},

    #[error("Argument is out of range")]
    ArgOutOfRange,

    #[error("This function has not yet been implemented")]
    Unimplemented {},

    #[error("Missing Cw20HookMg")]
    MissingHookMsg {},
}
