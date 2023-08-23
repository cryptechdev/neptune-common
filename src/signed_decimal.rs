use std::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::{Mul, Neg},
    str::FromStr,
};

use cosmwasm_std::Decimal256;
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::error::NeptuneError;

/// Decimal256 with a sign
#[derive(Clone, Copy, Debug, Eq)]
pub struct SignedDecimal {
    value: Decimal256,
    is_positive: bool,
}

impl SignedDecimal {
    pub const fn abs(&self) -> Self {
        Self {
            value: self.value,
            is_positive: true,
        }
    }

    pub const fn signum(&self) -> Self {
        match self.is_positive {
            true => Self::one(),
            false => Self {
                value: Decimal256::one(),
                is_positive: false,
            },
        }
    }

    pub const fn is_positive(&self) -> bool {
        self.is_positive
    }

    pub const fn is_negative(&self) -> bool {
        !self.is_positive
    }

    pub const fn one() -> Self {
        Self {
            value: Decimal256::one(),
            is_positive: true,
        }
    }

    pub const fn zero() -> Self {
        Self {
            value: Decimal256::zero(),
            is_positive: true,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl Mul<Decimal256> for SignedDecimal {
    type Output = SignedDecimal;

    fn mul(mut self, rhs: Decimal256) -> Self::Output {
        self.value *= rhs;
        self
    }
}

impl Neg for SignedDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_zero() {
            return self;
        }
        Self {
            value: self.value,
            is_positive: !self.is_positive,
        }
    }
}

impl ToString for SignedDecimal {
    fn to_string(&self) -> String {
        if self.is_zero() {
            String::from("0.0")
        } else {
            let sign_str = if self.is_positive { "" } else { "-" }.to_owned();
            sign_str + self.value.to_string().as_str()
        }
    }
}

impl std::ops::Add<Self> for SignedDecimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let value;
        let is_positive;
        if self.is_positive == rhs.is_positive {
            value = self.value + rhs.value;
            is_positive = self.is_positive;
        } else if self.value > rhs.value {
            value = self.value - rhs.value;
            is_positive = self.is_positive;
        } else if self.value < rhs.value {
            value = rhs.value - self.value;
            is_positive = rhs.is_positive
        } else {
            value = Decimal256::zero();
            is_positive = true;
        }
        Self { is_positive, value }
    }
}

impl std::ops::AddAssign<Self> for SignedDecimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<Self> for SignedDecimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + Self {
            value: rhs.value,
            is_positive: !rhs.is_positive,
        }
    }
}

impl std::ops::Mul<Self> for SignedDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let value = self.value * rhs.value;
        Self {
            value,
            is_positive: self.is_positive == rhs.is_positive || value.is_zero(),
        }
    }
}

impl std::ops::Div<Self> for SignedDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let value = if rhs.value.is_zero() {
            Decimal256::zero()
        } else {
            self.value / rhs.value
        };
        Self {
            value,
            is_positive: self.is_positive == rhs.is_positive || value.is_zero(),
        }
    }
}

impl std::cmp::PartialEq for SignedDecimal {
    fn eq(&self, other: &Self) -> bool {
        if self.is_zero() {
            return other.is_zero();
        }
        self.value == other.value && self.is_positive == other.is_positive
    }
}

impl std::cmp::PartialOrd for SignedDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for SignedDecimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.is_positive == other.is_positive {
            if self.is_positive {
                self.value.cmp(&other.value)
            } else {
                other.value.cmp(&self.value)
            }
        } else if self.is_positive {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    }
}

impl From<Decimal256> for SignedDecimal {
    fn from(value: Decimal256) -> Self {
        Self {
            value,
            is_positive: true,
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
            is_positive: sign,
        })
    }
}

/// Serializes as a decimal string
impl Serialize for SignedDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for SignedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SignedDecimalVisitor)
    }
}

struct SignedDecimalVisitor;

impl<'de> de::Visitor<'de> for SignedDecimalVisitor {
    type Value = SignedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded signed_decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Self::Value::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!(
                "Error parsing signed_decimal '{v}': {e}"
            ))),
        }
    }
}

impl JsonSchema for SignedDecimal {
    fn schema_name() -> String {
        "SignedDecimal".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }

    fn is_referenceable() -> bool {
        true
    }
}

impl TryFrom<&str> for SignedDecimal {
    type Error = NeptuneError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryInto<Decimal256> for SignedDecimal {
    type Error = NeptuneError;

    fn try_into(self) -> Result<Decimal256, Self::Error> {
        if !self.is_positive && !self.value.is_zero() {
            return Err(NeptuneError::Generic(
                "Cannot convert negative SignedDecimal to Decimal256".into(),
            ));
        }
        Ok(self.value)
    }
}

impl Default for SignedDecimal {
    fn default() -> Self {
        Self {
            value: Decimal256::default(),
            is_positive: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, ops::Div};

    use super::*;
    #[test]
    fn signed_decimal_test() {
        let big_pos = SignedDecimal::from_str("100").unwrap();
        let big_neg = SignedDecimal::from_str("-100").unwrap();
        let small_pos = SignedDecimal::from_str("50").unwrap();
        let small_neg = SignedDecimal::from_str("-50").unwrap();
        let dec_neg = SignedDecimal::from_str("-50.50").unwrap();

        let big_pos_f64 = f64::from_str("100").unwrap();
        let big_neg_f64 = f64::from_str("-100").unwrap();
        let small_pos_f64 = f64::from_str("50").unwrap();
        let small_neg_f64 = f64::from_str("-50").unwrap();
        let dec_neg_f64 = f64::from_str("-50.50").unwrap();

        // Test partial_cmp
        assert!(big_pos > big_neg);
        assert!(big_pos > small_neg);
        assert!(big_pos > small_pos);
        assert!(big_pos > big_neg);
        assert!(small_pos > small_neg);
        assert!(small_pos > big_neg);
        assert!(small_neg > big_neg);

        // Utility function
        fn f64_to_signed_decimal(val: f64) -> SignedDecimal {
            SignedDecimal::from_str(val.to_string().as_str()).unwrap()
        }

        // Test mul
        assert!(big_pos * small_pos == f64_to_signed_decimal(big_pos_f64 * small_pos_f64));
        assert!(big_pos * small_neg == f64_to_signed_decimal(big_pos_f64 * small_neg_f64));
        assert!(big_pos * big_neg == f64_to_signed_decimal(big_pos_f64 * big_neg_f64));
        assert!(small_pos * small_neg == f64_to_signed_decimal(small_pos_f64 * small_neg_f64));
        assert!(small_pos * big_neg == f64_to_signed_decimal(small_pos_f64 * big_neg_f64));
        assert!(small_neg * big_neg == f64_to_signed_decimal(small_neg_f64 * big_neg_f64));

        // Test div
        assert!(big_pos / small_pos == f64_to_signed_decimal(big_pos_f64 / small_pos_f64));
        assert!(big_pos / small_neg == f64_to_signed_decimal(big_pos_f64 / small_neg_f64));
        assert!(big_pos / big_neg == f64_to_signed_decimal(big_pos_f64 / big_neg_f64));
        assert!(small_pos / small_neg == f64_to_signed_decimal(small_pos_f64 / small_neg_f64));
        assert!(small_pos / big_neg == f64_to_signed_decimal(small_pos_f64 / big_neg_f64));
        assert!(small_neg / big_neg == f64_to_signed_decimal(small_neg_f64 / big_neg_f64));

        // Test add
        assert!(big_pos + small_pos == f64_to_signed_decimal(big_pos_f64 + small_pos_f64));
        assert!(big_pos + small_neg == f64_to_signed_decimal(big_pos_f64 + small_neg_f64));
        assert!(big_pos + big_neg == f64_to_signed_decimal(big_pos_f64 + big_neg_f64));
        assert!(small_pos + small_neg == f64_to_signed_decimal(small_pos_f64 + small_neg_f64));
        assert!(small_pos + big_neg == f64_to_signed_decimal(small_pos_f64 + big_neg_f64));
        assert!(small_neg + big_neg == f64_to_signed_decimal(small_neg_f64 + big_neg_f64));

        // Test sub
        assert!(big_pos - small_pos == f64_to_signed_decimal(big_pos_f64 - small_pos_f64));
        assert!(big_pos - small_neg == f64_to_signed_decimal(big_pos_f64 - small_neg_f64));
        assert!(big_pos - big_neg == f64_to_signed_decimal(big_pos_f64 - big_neg_f64));
        assert!(small_pos - small_neg == f64_to_signed_decimal(small_pos_f64 - small_neg_f64));
        assert!(small_pos - big_neg == f64_to_signed_decimal(small_pos_f64 - big_neg_f64));
        assert!(small_neg - big_neg == f64_to_signed_decimal(small_neg_f64 - big_neg_f64));

        // Test conversion
        assert!(big_pos == f64_to_signed_decimal(big_pos_f64));
        assert!(big_neg == f64_to_signed_decimal(big_neg_f64));
        assert!(small_pos == f64_to_signed_decimal(small_pos_f64));
        assert!(small_neg == f64_to_signed_decimal(small_neg_f64));
        assert!(dec_neg == f64_to_signed_decimal(dec_neg_f64));

        // Test division by zero
        let num = SignedDecimal::one();
        let denom = SignedDecimal::zero();
        assert_eq!(num.div(denom), SignedDecimal::zero());

        // Test try_into
        {
            let x = SignedDecimal::one().neg();
            TryInto::<Decimal256>::try_into(x).expect_err("Should throw error for negatives");

            let y = SignedDecimal::one();
            let _: Decimal256 = y.try_into().expect("Should be able to convert");
        }

        // Test cmp
        {
            let lhs = SignedDecimal::one().neg();
            let rhs = SignedDecimal::one();
            assert_eq!(lhs.cmp(&rhs), Ordering::Less);
            assert_eq!(rhs.cmp(&lhs), Ordering::Greater);
            assert_eq!(rhs.cmp(&rhs), Ordering::Equal);

            let x = SignedDecimal::from_str("50").unwrap();
            let y = SignedDecimal::from_str("10").unwrap();
            assert_eq!(x.cmp(&y), Ordering::Greater);
            assert_eq!(y.cmp(&x), Ordering::Less);
            let z = SignedDecimal::from_str("50").unwrap();
            assert_eq!(x.cmp(&z), Ordering::Equal);
        }

        {
            let x = Decimal256::default();
            let z = SignedDecimal::from(x);
            let y = SignedDecimal::default();
            assert_eq!(z, y);
        }
    }

    #[test]
    fn test_sign_fns() {
        let mut sd =
            SignedDecimal::from_str("4.1243").expect("have to be able to SignedDecimal::from_str");
        assert!(sd.is_positive());
        assert!(!sd.is_negative());

        sd = sd.neg();
        assert!(!sd.is_positive());
        assert!(sd.is_negative());

        sd = sd.abs();
        assert!(sd.is_positive());
        assert!(!sd.is_negative());
    }

    #[test]
    fn test_zero_is_positive() {
        {
            let mut x = SignedDecimal::zero();
            let y = SignedDecimal::one().neg();

            x = x * y;
            assert!(x.is_positive);

            x = y * x;
            assert!(x.is_positive);

            x = x / y;
            assert!(x.is_positive);

            x += y;
            x = x - y;
            assert!(x.is_positive);

            x = x - y;
            x += y;
            assert!(x.is_positive);
        }
        {
            let x = SignedDecimal::one() * SignedDecimal::from_str("5.0").unwrap();
            let y = SignedDecimal::one() * SignedDecimal::from_str("-5.0").unwrap();

            let z = x + y;
            assert!(z.is_positive);

            let z = -x - y;
            assert!(z.is_positive);
        }
        {
            let x = -SignedDecimal::zero();
            assert!(x.is_positive);
        }
        {
            let x = SignedDecimal::zero().neg();
            assert!(x.is_positive);
        }
        {
            let x = SignedDecimal::zero().neg();
            let y = SignedDecimal::from_str("5.0").unwrap();

            let z = x * y;
            assert!(z.is_positive);

            let z = y * x;
            assert!(z.is_positive);
        }
    }
}
