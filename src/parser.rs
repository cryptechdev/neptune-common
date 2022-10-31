use std::{fmt::Debug, str::FromStr};

use cosmwasm_std::{Addr, Binary, Timestamp, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::{asset::AssetInfo, error::CommonResult};

pub fn addr_parser(s: &str) -> CommonResult<Addr> { Ok(Addr::unchecked(s.to_string())) }

pub fn binary_parser(s: &str) -> CommonResult<Binary> { Ok(Binary::from_base64(s)?) }

pub fn time_stamp_parser(s: &str) -> CommonResult<Timestamp> { Ok(serde_json::from_str::<Timestamp>(s).unwrap()) }

pub fn cw20_receive_parser(_: &str) -> CommonResult<Cw20ReceiveMsg> {
    Ok(Cw20ReceiveMsg { sender: String::default(), amount: Uint128::default(), msg: Binary::default() })
}

pub fn tuple_parser<T, U>(s: &str) -> CommonResult<(T, U)>
where
    T: FromStr,
    U: FromStr,
    <T as FromStr>::Err: Debug,
    <U as FromStr>::Err: Debug,
{
    let vec: Vec<&str> = s.split(',').collect();
    Ok((T::from_str(vec[0]).unwrap(), U::from_str(vec[1]).unwrap()))
}

pub fn asset_info_iter_parser(s: &str) -> CommonResult<impl IntoIterator<Item = AssetInfo> + Clone> {
    let mut asset_infos = vec![];
    for str in s.split(',') {
        asset_infos.push(AssetInfo::from_str(str)?);
    }
    Ok(asset_infos)
}
