use crate::{
    asset::{AssetAmount, AssetInfo},
    error::NeptuneError,
};

impl From<AssetInfo> for astroport::asset::AssetInfo {
    fn from(value: AssetInfo) -> Self {
        match value {
            AssetInfo::Token { contract_addr } => {
                astroport::asset::AssetInfo::Token { contract_addr }
            }
            AssetInfo::NativeToken { denom } => astroport::asset::AssetInfo::NativeToken { denom },
        }
    }
}

impl From<astroport::asset::AssetInfo> for AssetInfo {
    fn from(value: astroport::asset::AssetInfo) -> Self {
        match value {
            astroport::asset::AssetInfo::Token { contract_addr } => {
                AssetInfo::Token { contract_addr }
            }
            astroport::asset::AssetInfo::NativeToken { denom } => AssetInfo::NativeToken { denom },
        }
    }
}

impl TryFrom<AssetAmount> for astroport::asset::Asset {
    type Error = NeptuneError;

    fn try_from(value: AssetAmount) -> Result<Self, Self::Error> {
        Ok(Self {
            info: value.info.into(),
            amount: value.amount.try_into()?,
        })
    }
}
