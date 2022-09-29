use crate::{asset::{AssetInfo}, map::Map};

pub type AssetMap<T> = Map<AssetInfo, T>;

impl<T> AssetMap<T>
{

}

// impl From<AssetMap<Uint256>> for AssetMap<Uint256> {
//     fn from(mut object: AssetMap<Uint256>) -> Self {
//         object.iter_mut().map(|x| {
//             (x.info.clone(), x.amount)
//         }).collect::<Vec<(AssetInfo, Uint256)>>().into()
//     }
// }

// impl<T> Into<AssetVec> for AssetMap<T> {
//     fn into(self) -> AssetVec {
//         todo!()
//     }
// }

#[derive(Clone, Debug)]
pub struct AssetVec(Vec<AssetInfo>);

impl IntoIterator for AssetVec {
    type Item = AssetInfo;

    type IntoIter = std::vec::IntoIter<AssetInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<AssetInfo>> for AssetVec {
    fn from(object: Vec<AssetInfo>) -> Self {
        AssetVec(object)
    }
}