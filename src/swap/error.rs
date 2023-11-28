use crate::asset::AssetInfo;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SwapError {
    #[error("liquidity pool not found {0:?}")]
    PoolNotFound([AssetInfo; 2]),

    #[error("Insufficient liquidity to execute swap")]
    InsufficientLiquidity,

    #[error("Invalid asset")]
    InvalidAsset,

    #[error("Invalid pool")]
    InvalidPool,

    #[error("Spot market not found")]
    SpotMarketNotFound,

    #[error("Invalid offer asset")]
    InvalidOfferAsset,
}
