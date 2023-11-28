use cosmwasm_std::{CheckedFromRatioError, ConversionOverflowError, StdError};
use neptune_auth::error::NeptAuthError;
use thiserror::Error;

pub type NeptuneResult<T> = core::result::Result<T, NeptuneError>;

#[derive(Error, Debug, PartialEq)]
pub enum NeptuneError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Auth(#[from] NeptAuthError),

    #[error(transparent)]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[error(transparent)]
    CheckedFromRatio(#[from] CheckedFromRatioError),

    #[cfg(feature = "swap")]
    #[error(transparent)]
    SwapError(#[from] crate::swap::error::SwapError),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Only one tx is allowed per block")]
    MultipleTx,

    #[error("Missing Cw20HookMg")]
    MissingHookMsg,

    #[error("{0}")]
    Conversion(String),

    #[error("{0}")]
    Generic(String),
}
