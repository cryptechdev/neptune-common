use cosmwasm_std::{Deps, Order};
use cw_storage_plus::{Bounder, KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::CommonError;

pub const PARAMS_KEY: &str = "params";
pub const STATE_KEY: &str = "state";

/// Reads a map from storage is ascending order.
///
/// TODO: Doc Test Here
pub fn read_map<
    'a,
    K: 'static,
    R: Bounder<'a> + PrimaryKey<'a> + KeyDeserialize<Output = K> + 'a,
    V: Serialize + DeserializeOwned,
>(
    deps: Deps, start_after: Option<R>, limit: Option<u32>, map: Map<'a, R, V>,
) -> Result<Vec<(K, V)>, CommonError> {
    let start = start_after.map(|key| key.inclusive_bound().unwrap());
    let vec = match limit {
        Some(limit) => map
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit as usize)
            .collect::<Result<Vec<_>, _>>()?,
        None => map.range(deps.storage, start, None, Order::Ascending).collect::<Result<Vec<_>, _>>()?,
    };
    Ok(vec)
}
