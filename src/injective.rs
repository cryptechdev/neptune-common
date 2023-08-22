use cosmwasm_std::{Decimal256, Uint256};
use injective_math::FPDecimal;
use std::str::FromStr;

pub fn into_fp_decimal(value: Decimal256) -> FPDecimal {
    let atomics = value.atomics().to_be_bytes();
    FPDecimal {
        num: atomics.into(),
        sign: 1,
    }
}

pub fn into_decimal_256(value: FPDecimal) -> Decimal256 {
    if value.sign.is_negative() {
        panic!("Negative value can't be converted")
    }
    let atomics: [u8; 32] = value.num.into();
    Decimal256::new(Uint256::from_be_bytes(atomics))
}

pub fn into_uint_256(value: FPDecimal) -> Uint256 {
    // Error for negative values handled implicitly here.
    Uint256::from_str(&value.to_string()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_fp_decimal() {
        let string = "23423498725.1238476198263".to_string();
        let dec_256 = Decimal256::from_str(string.as_str()).unwrap();
        let fp_dec: FPDecimal = into_fp_decimal(dec_256);
        assert_eq!(fp_dec, FPDecimal::from_str(string.as_str()).unwrap());
    }

    #[test]
    fn test_into_decimal_256() {
        let string = "23423498725.1238476198263".to_string();
        let fp_dec = FPDecimal::from_str(string.as_str()).unwrap();
        let dec_256: Decimal256 = into_decimal_256(fp_dec);
        assert_eq!(dec_256, Decimal256::from_str(string.as_str()).unwrap());
    }
}
