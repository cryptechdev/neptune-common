use std::convert::TryFrom;

use cosmwasm_std::{Decimal256, Uint128, Uint256};

use crate::error::{CommonError, CommonResult};

pub fn div_or_zero(num: Uint256, denom: Uint256) -> Decimal256 {
    if denom.is_zero() {
        Decimal256::zero()
    } else {
        Decimal256::from_ratio(num, denom)
    }
}

pub fn to_uint128(other: Uint256) -> CommonResult<Uint128> {
    Uint128::try_from(other).map_err(CommonError::ConversionOverflowError)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_division_or_zero_test() {
        assert!(div_or_zero(Uint256::one(), Uint256::zero()).is_zero());
    }
}
