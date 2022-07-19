use std::{str::FromStr};

use cosmwasm_std::{Decimal256, Uint256};
use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::NeptuneError;

/// Decimal256 with a sign
#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema)]
pub struct SignedDecimal {
    value: Decimal256,
    sign: bool
}

impl SignedDecimal {
    pub fn nan() -> Self {
        Self {
            value: Decimal256::zero(),
            sign: false
        }
    }

    pub fn is_nan(&self) -> bool {
        self.value.is_zero() && self.sign == false
    }

    pub fn value(&self) -> Decimal256 {
        assert!(self.sign, "SignedDecimal is negative!");
        self.value
    }

    pub fn from_uint256(val: Uint256) -> Result<SignedDecimal, NeptuneError> {
        Ok(SignedDecimal {
            value: Decimal256::from_atomics(val, 0u32).map_err(|e| NeptuneError::Decimal256RangeExceeded(e))?,
            sign: true
        })
    }
}

impl ToString for SignedDecimal {
    fn to_string(&self) -> String {
        if self.is_nan() { String::from("NaN") }
        else {
            let sign_str = if self.sign { "" } else { "-" }.to_owned();
            sign_str + self.value.to_string().as_str()
        }
    }
}

impl std::ops::Add<SignedDecimal> for SignedDecimal {
    type Output = Self;

    fn add(self, rhs: SignedDecimal) -> Self {
        let value;
        let sign;
        if self.sign == rhs.sign { 
            value = self.value + rhs.value;
            sign = self.sign;
        } else { 
            if self.value > rhs.value {
                value = self.value - rhs.value;
                sign = self.sign;
            } else if self.value < rhs.value {
                value = rhs.value - self.value;
                sign = rhs.sign
            } else {
                value = Decimal256::zero();
                sign = true;
            }
        }
        Self { sign, value }
    }
}

impl std::ops::Sub<SignedDecimal> for SignedDecimal {
    type Output = Self;

    fn sub(self, rhs: SignedDecimal) -> Self {
        self + Self{
            value: rhs.value,
            sign: !rhs.sign
        }
    }
}

impl std::ops::Mul<SignedDecimal> for SignedDecimal {
    type Output = Self;

    fn mul(self, rhs: SignedDecimal) -> Self {
        let value = self.value * rhs.value;
        Self {
            value,
            sign: self.sign == rhs.sign || value.is_zero()
        }
    }
}

impl std::ops::Div<SignedDecimal> for SignedDecimal {
    type Output = Self;

    fn div(self, rhs: SignedDecimal) -> Self {
        let value = if rhs.value.is_zero() { rhs.value } else { self.value / rhs.value };
        Self {
            value,
            sign: self.sign == rhs.sign || value.is_zero()
        }
    }
}

impl std::cmp::PartialEq for SignedDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.sign == other.sign
    }
}

impl std::cmp::PartialOrd for SignedDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.sign == other.sign {
            if self.sign { self.value.partial_cmp(&other.value) }
            else { other.value.partial_cmp(&self.value) }
        }
        else {
            if self.sign { Some(std::cmp::Ordering::Greater) }
            else { Some(std::cmp::Ordering::Less) }
        }
    }
}

impl From<Decimal256> for SignedDecimal {
    fn from(value: Decimal256) -> Self {
        Self {
            value,
            sign: true
        }
    }
}

impl FromStr for SignedDecimal {
    type Err = NeptuneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sign;
        let val_str;
        let mut chars = s.chars();
        if chars.next().unwrap() == '-' {
            sign = false;
            val_str = chars.as_str();
        } else {
            sign = true;
            val_str = s;
        }
        Ok(Self {
            value: Decimal256::from_str(val_str)?,
            sign: sign
        })
    }
}

impl Default for SignedDecimal {
    fn default() -> Self {
        Self {
            value: Decimal256::default(),
            sign: true
        }
    }
}

#[test]
fn signed_decimal_test() {
    let big_pos   = SignedDecimal::from_str("100").unwrap();
    let big_neg   = SignedDecimal::from_str("-100").unwrap();
    let small_pos = SignedDecimal::from_str("50").unwrap();
    let small_neg = SignedDecimal::from_str("-50").unwrap();

    let big_pos_f64   = f64::from_str("100").unwrap();
    let big_neg_f64   = f64::from_str("-100").unwrap();
    let small_pos_f64 = f64::from_str("50").unwrap();
    let small_neg_f64 = f64::from_str("-50").unwrap();

    // Test partial_cmp
    assert!(big_pos   > big_neg);
    assert!(big_pos   > small_neg);
    assert!(big_pos   > small_pos);
    assert!(big_pos   > big_neg);
    assert!(small_pos > small_neg);
    assert!(small_pos > big_neg);
    assert!(small_neg > big_neg);

    // Utility function
    fn f64_to_signed_decimal(val: f64) -> SignedDecimal {
        SignedDecimal::from_str(val.to_string().as_str()).unwrap()
    }

    // Test mul
    assert!(big_pos   * small_pos == f64_to_signed_decimal( big_pos_f64   * small_pos_f64 ));
    assert!(big_pos   * small_neg == f64_to_signed_decimal( big_pos_f64   * small_neg_f64 ));
    assert!(big_pos   * big_neg   == f64_to_signed_decimal( big_pos_f64   * big_neg_f64   ));
    assert!(small_pos * small_neg == f64_to_signed_decimal( small_pos_f64 * small_neg_f64 ));
    assert!(small_pos * big_neg   == f64_to_signed_decimal( small_pos_f64 * big_neg_f64   ));
    assert!(small_neg * big_neg   == f64_to_signed_decimal( small_neg_f64 * big_neg_f64   ));

    // Test div
    assert!(big_pos   / small_pos == f64_to_signed_decimal( big_pos_f64   / small_pos_f64 ));
    assert!(big_pos   / small_neg == f64_to_signed_decimal( big_pos_f64   / small_neg_f64 ));
    assert!(big_pos   / big_neg   == f64_to_signed_decimal( big_pos_f64   / big_neg_f64   ));
    assert!(small_pos / small_neg == f64_to_signed_decimal( small_pos_f64 / small_neg_f64 ));
    assert!(small_pos / big_neg   == f64_to_signed_decimal( small_pos_f64 / big_neg_f64   ));
    assert!(small_neg / big_neg   == f64_to_signed_decimal( small_neg_f64 / big_neg_f64   ));

    // Test add
    assert!(big_pos   + small_pos == f64_to_signed_decimal( big_pos_f64   + small_pos_f64 ));
    assert!(big_pos   + small_neg == f64_to_signed_decimal( big_pos_f64   + small_neg_f64 ));
    assert!(big_pos   + big_neg   == f64_to_signed_decimal( big_pos_f64   + big_neg_f64   ));
    assert!(small_pos + small_neg == f64_to_signed_decimal( small_pos_f64 + small_neg_f64 ));
    assert!(small_pos + big_neg   == f64_to_signed_decimal( small_pos_f64 + big_neg_f64   ));
    assert!(small_neg + big_neg   == f64_to_signed_decimal( small_neg_f64 + big_neg_f64   ));

    // Test sub
    assert!(big_pos   - small_pos == f64_to_signed_decimal( big_pos_f64   - small_pos_f64 ));
    assert!(big_pos   - small_neg == f64_to_signed_decimal( big_pos_f64   - small_neg_f64 ));
    assert!(big_pos   - big_neg   == f64_to_signed_decimal( big_pos_f64   - big_neg_f64   ));
    assert!(small_pos - small_neg == f64_to_signed_decimal( small_pos_f64 - small_neg_f64 ));
    assert!(small_pos - big_neg   == f64_to_signed_decimal( small_pos_f64 - big_neg_f64   ));
    assert!(small_neg - big_neg   == f64_to_signed_decimal( small_neg_f64 - big_neg_f64   ));
}