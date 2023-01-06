use cosmwasm_std::{Decimal256, Uint256};

/// Division that returns zero if the denominator is zero.
/// ```
/// # use cosmwasm_std::{attr, Response, StdResult, Uint256, Decimal256};
/// # use neptune_common::math::div_or_zero;
/// # fn test_div_or_zero() {
/// assert!(div_or_zero(Uint256::one(), Uint256::zero()).is_zero());
/// # }
/// ```
pub fn div_or_zero(num: Uint256, denom: Uint256) -> Decimal256 {
    if denom.is_zero() {
        Decimal256::zero()
    } else {
        Decimal256::from_ratio(num, denom)
    }
}
