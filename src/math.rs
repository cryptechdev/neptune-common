use std::convert::TryFrom;

use cosmwasm_std::{Decimal, Decimal256, Uint128, Uint256};

use crate::error::{CommonError, CommonResult};

pub const UINT256_ONE: Uint256 = Uint256::from_u128(1u128);

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

pub fn from_decimal(other: Decimal) -> Decimal256 {
    // Unwrap is safe because Decimal256 and Decimal have the same decimal places.
    // Every Decimal value can be stored in Decimal256.
    Decimal256::from_atomics(other.atomics(), other.decimal_places()).unwrap()
}

#[test]
fn assert_serialize() {
    use cosmwasm_std::to_binary;
    use rand::Rng;

    for _ in 1..1000 {
        let mut rng = rand::thread_rng();
        let num_1: u128 = rng.gen();
        let num_2: u128 = rng.gen();

        let big_num_1 = cosmwasm_std::Uint256::from(num_1);
        let std_num_1 = cosmwasm_std::Uint256::from(num_1);

        let big_num_2 = cosmwasm_std::Uint256::from(num_2);
        let std_num_2 = cosmwasm_std::Uint256::from(num_2);

        let big_dec = cosmwasm_std::Decimal256::from_ratio(big_num_1, big_num_2);
        let std_dec = cosmwasm_std::Decimal256::from_ratio(std_num_1, std_num_2);

        assert_eq!(to_binary(&big_dec).unwrap(), to_binary(&std_dec).unwrap());
        assert_eq!(to_binary(&big_num_1).unwrap(), to_binary(&std_num_1).unwrap());
    }
}

#[test]
fn get_division_or_zero_test() {
    assert!(div_or_zero(UINT256_ONE, Uint256::zero()).is_zero());
}
