use cosmwasm_std::{ConversionOverflowError, Decimal256RangeExceeded, OverflowError, StdError};
use neptune_authorization::error::NeptAuthError;
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
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Insufficient liquidity to send funds")]
    InsufficientLiquidity {},

    #[error("Asset not found")]
    AssetNotFound {},

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("No stable funds were attached")]
    NoFundsReceived {},

    #[error("Amount received is less than minimum receive")]
    MinimumReceive(),

    #[error("Missing address for {0}")]
    MissingAddress(String),

    #[error("Missing config variable")]
    MissingConfigVariable {},

    #[error("Missing admin addresses")]
    MissingAdminAddresses {},

    #[error("Missing admin double sig address")]
    MissingAdminDoubleSigAddress {},

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

impl From<CommonError> for NeptAuthError {
    fn from(val: CommonError) -> Self { Self::Error(val.to_string()) }
}

impl From<Box<dyn std::error::Error>> for CommonError {
    fn from(error: Box<dyn std::error::Error>) -> Self { Self::Error(error.to_string()) }
}
