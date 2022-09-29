use cosmwasm_std::Uint256;
use crate::{asset::{AssetInfo, AssetVec, AssetAmount}, map::Map};
pub type AssetMap<T> = Map<AssetInfo, T>;

impl<T> AssetMap<T>
{

}

impl From<Vec<AssetAmount>> for AssetMap<Uint256> {
    fn from(mut object: Vec<AssetAmount>) -> Self {
        object.iter_mut().map(|x| {
            (x.info.clone(), x.amount)
        }).collect::<Vec<(AssetInfo, Uint256)>>().into()
    }
}

impl<T> Into<AssetVec> for AssetMap<T> {
    fn into(self) -> AssetVec {
        todo!()
    }
}