use crate::{asset::AssetInfo, map::Map};

pub type AssetMap<T> = Map<AssetInfo, T>;

#[derive(Clone, Debug)]
pub struct AssetVec(Vec<AssetInfo>);

impl IntoIterator for AssetVec {
    type IntoIter = std::vec::IntoIter<AssetInfo>;
    type Item = AssetInfo;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl From<Vec<AssetInfo>> for AssetVec {
    fn from(object: Vec<AssetInfo>) -> Self { AssetVec(object) }
}
