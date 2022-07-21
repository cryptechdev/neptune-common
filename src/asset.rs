use cosmwasm_std::{Uint256, Addr};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, JsonSchema)]
pub enum AssetInfo {
    Token{ addr: Addr },
    NativeToken{ denom: String },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Asset {
    pub asset_info: AssetInfo,
    pub amount: Uint256,
}