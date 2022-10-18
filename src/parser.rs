use crate::error::CommonResult;
use cosmwasm_std::{Addr, Binary};

pub fn addr_parser(s: &str) -> CommonResult<Addr> {
    Ok(Addr::unchecked(s.to_string()))
}

pub fn binary_parser(s: &str) -> CommonResult<Binary> {
    Ok(Binary::from_base64(s)?)
}