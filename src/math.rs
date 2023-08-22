use cosmwasm_std::{Decimal256, StdResult, Uint256, Uint512};

/// Division that returns zero if the denominator is zero.
/// ```
/// # use cosmwasm_std::Uint256;
/// # use neptune_common::math::div_or_zero;
/// # fn test_div_or_zero() {
/// assert!(div_or_zero(Uint256::one(), Uint256::zero()).is_zero());
/// # }
/// ```
pub fn div_or_zero(num: Uint256, denom: Uint256) -> Decimal256 {
    Decimal256::checked_from_ratio(num, denom).unwrap_or_default()
}

/// Divide a `Uint256` by a `Decimal256`.
/// ```
/// # use cosmwasm_std::{Uint256, Decimal256};
/// # use neptune_common::math::checked_div;
/// # fn test_checked_div() {
/// assert_eq!(
///     checked_div(
///         Uint256::from(1500u64),
///         Decimal256::from_ratio(3u64, 2u64)
///     ),
///     Ok(Uint256::from(1000u64))
/// );
/// # }
/// ```
pub fn checked_div(numerator: Uint256, denominator: Decimal256) -> StdResult<Uint256> {
    Ok(numerator
        .full_mul(Decimal256::one().atomics())
        .checked_div(Uint512::from(denominator.atomics()))?
        .try_into()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Decimal256, Uint256};
    use std::str::FromStr;

    #[test]
    fn test_checked_div_precision() {
        assert_eq!(
            checked_div(
                Uint256::from(10_000_000_000_000_000_000_000u128),
                Decimal256::from_str("10000000000000000000.0").unwrap()
            ),
            Ok(Uint256::from(1000u64))
        )
    }
}
