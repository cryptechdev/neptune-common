use cosmwasm_std::{OverflowError, StdError, ConversionOverflowError, Decimal256RangeExceeded};
use neptune_authorization::error::NeptuneAuthorizationError;
use thiserror::Error;

pub type CommonResult<T> = core::result::Result<T, CommonError>;

const NEPT_ERR: &str = "🔱 Neptune Error -";

#[derive(Error, Debug, PartialEq)]
pub enum CommonError {
    #[error("{} {0}", NEPT_ERR)]
    Error(String),

    #[error("{} Generic: {0}", NEPT_ERR)]
    Generic(String),

    #[error("{} StdError: {0}", NEPT_ERR)]
    Std(#[from] StdError),

    #[error("{} AuthError: {0}", NEPT_ERR)]
    Auth(#[from] NeptuneAuthorizationError),

    #[error("{} OverflowError: {0}", NEPT_ERR)]
    OverflowError(#[from] OverflowError),

    #[error("{} ConversionOverflowError: {0}", NEPT_ERR)]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{} Decimal256RangeExceeded: {0}", NEPT_ERR)]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error("{} Unauthorized: {0}", NEPT_ERR)]
    Unauthorized (String),

    #[error("{} Insufficient liquidity to send funds", NEPT_ERR)]
    InsufficientLiquidity {},

    #[error("{} No stable funds were attached", NEPT_ERR)]
    NoFundsReceived {},

    #[error("{} Missing address for {0}", NEPT_ERR)]
    MissingAddress ( String ),

    #[error("{} Missing config variable", NEPT_ERR)]
    MissingConfigVariable {},

    #[error("{} Missing admin addresses", NEPT_ERR)]
    MissingAdminAddresses {},

    #[error("{} Missing admin double sig address", NEPT_ERR)]
    MissingAdminDoubleSigAddress {},

    #[error("{} Denominator was zero", NEPT_ERR)]
    ZeroDenominator {},
    
    #[error("{} Basset price was returned as zero", NEPT_ERR)]
    BassetPriceIsZero {},

    #[error("{} Argument is out of range", NEPT_ERR)]
    ArgOutOfRange,

    #[error("{} This function has not yet been implemented", NEPT_ERR)]
    Unimplemented {},

    #[error("{} Missing Cw20HookMg", NEPT_ERR)]
    MissingHookMsg {},
}

impl Into<NeptuneAuthorizationError> for CommonError {
    fn into(self) -> NeptuneAuthorizationError {
        NeptuneAuthorizationError::Error(self.to_string())
    }
}

impl From<Box<dyn std::error::Error>>for CommonError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Error(error.to_string())
    }
}