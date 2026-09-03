use monumentum_db::error::DbError;
use monumentum_db::types::Float;
use proptest::prelude::*;
use std::cmp::Ordering;

fn finite_float_strategy() -> impl Strategy<Value = f64> {
    proptest::prelude::any::<f64>().prop_filter("must be finite", |f| f.is_finite())
}

#[test]
fn try_new_accepts_finite_positive() -> Result<(), DbError> {
    let f = Float::try_new(2.5)?;
    assert_eq!(f.as_f64(), 2.5);
    Ok(())
}

#[test]
fn try_new_accepts_finite_negative() -> Result<(), DbError> {
    let f = Float::try_new(-0.5)?;
    assert_eq!(f.as_f64(), -0.5);
    Ok(())
}

#[test]
fn try_new_accepts_zero() -> Result<(), DbError> {
    let f1 = Float::try_new(0.0)?;
    let f2 = Float::try_new(-0.0)?;
    assert_eq!(f1.as_f64(), 0.0);
    assert_eq!(f2.as_f64(), -0.0);
    Ok(())
}

#[test]
fn try_new_rejects_nan() {
    let result = Float::try_new(f64::NAN);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: float must be finite (no NaN or infinity)"
        );
    }
}

#[test]
fn try_new_rejects_positive_infinity() {
    let result = Float::try_new(f64::INFINITY);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: float must be finite (no NaN or infinity)"
        );
    }
}

#[test]
fn try_new_rejects_negative_infinity() {
    let result = Float::try_new(f64::NEG_INFINITY);
    assert!(result.is_err());
}

#[test]
fn as_f64_returns_value() -> Result<(), DbError> {
    let f = Float::try_new(1.5)?;
    assert_eq!(f.as_f64(), 1.5);
    let g = Float::try_new(-2.0)?;
    assert_eq!(g.as_f64(), -2.0);
    Ok(())
}

#[test]
fn total_cmp_standard_ordering() -> Result<(), DbError> {
    let a = Float::try_new(1.0)?;
    let b = Float::try_new(2.0)?;
    let c = Float::try_new(1.5)?;
    let d = Float::try_new(1.5)?;

    assert_eq!(a.total_cmp(&b), Ordering::Less);
    assert_eq!(b.total_cmp(&a), Ordering::Greater);
    assert_eq!(c.total_cmp(&d), Ordering::Equal);
    Ok(())
}

#[test]
fn total_cmp_negative_and_zero() -> Result<(), DbError> {
    let neg = Float::try_new(-1.0)?;
    let zero = Float::try_new(0.0)?;
    assert_eq!(neg.total_cmp(&zero), Ordering::Less);

    let neg_zero = Float::try_new(-0.0)?;
    let pos_zero = Float::try_new(0.0)?;
    assert_eq!(neg_zero.total_cmp(&pos_zero), Ordering::Less);
    Ok(())
}

#[test]
fn try_from_le_bytes_roundtrip() -> Result<(), DbError> {
    let original = 123.456_f64;
    let bytes = original.to_le_bytes();
    let f = Float::try_from_le_bytes(bytes)?;
    assert_eq!(f.as_f64(), original);
    Ok(())
}

#[test]
fn try_from_le_bytes_rejects_non_finite() {
    let nan_bytes = f64::NAN.to_le_bytes();
    let inf_bytes = f64::INFINITY.to_le_bytes();
    assert!(Float::try_from_le_bytes(nan_bytes).is_err());
    assert!(Float::try_from_le_bytes(inf_bytes).is_err());
}

#[test]
fn to_le_bytes_matches_f64() -> Result<(), DbError> {
    let value = 0.1_f64;
    let f = Float::try_new(value)?;
    assert_eq!(f.to_le_bytes(), value.to_le_bytes());
    Ok(())
}

#[test]
fn display_uses_debug_format() -> Result<(), DbError> {
    let f = Float::try_new(1.5)?;
    let expected = format!("{:?}", 1.5_f64);
    assert_eq!(format!("{}", f), expected);
    Ok(())
}

#[test]
fn try_from_f64_finite_success() -> Result<(), DbError> {
    let f = Float::try_from(2.0)?;
    assert_eq!(f.as_f64(), 2.0);
    Ok(())
}

#[test]
fn try_from_f64_non_finite_error() {
    assert!(Float::try_from(f64::NAN).is_err());
    assert!(Float::try_from(f64::INFINITY).is_err());
}

#[test]
fn try_from_str_valid() -> Result<(), DbError> {
    let f = Float::try_from("2.5")?;
    assert_eq!(f.as_f64(), 2.5);
    let g = Float::try_from("-0.5")?;
    assert_eq!(g.as_f64(), -0.5);
    Ok(())
}

#[test]
fn try_from_str_invalid() {
    assert!(Float::try_from("").is_err());
    assert!(Float::try_from("abc").is_err());
}

#[test]
fn try_from_str_non_finite() {
    assert!(Float::try_from("NaN").is_err());
    assert!(Float::try_from("inf").is_err());
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    #[test]
    fn try_new_finite_roundtrip(f in finite_float_strategy()) {
        let result = Float::try_new(f);
        prop_assert!(result.is_ok());
        if let Ok(float) = result {
            prop_assert_eq!(float.as_f64(), f);
        }
    }

    #[test]
    fn total_cmp_antisymmetric(
        a in finite_float_strategy(),
        b in finite_float_strategy(),
    ) {
        let fa_res = Float::try_new(a);
        let fb_res = Float::try_new(b);
        prop_assert!(fa_res.is_ok());
        prop_assert!(fb_res.is_ok());

        if let (Ok(fa), Ok(fb)) = (fa_res, fb_res) {
            let ord = fa.total_cmp(&fb);
            let rev = fb.total_cmp(&fa);
            prop_assert_eq!(ord.reverse(), rev);
        }
    }

    #[test]
    fn to_le_bytes_roundtrip(f in finite_float_strategy()) {
        let float_res = Float::try_new(f);
        prop_assert!(float_res.is_ok());
        if let Ok(float) = float_res {
            let bytes = float.to_le_bytes();
            let restored = Float::try_from_le_bytes(bytes);
            prop_assert!(restored.is_ok());
            if let Ok(restored_float) = restored {
                prop_assert_eq!(float, restored_float);
            }
        }
    }

    #[test]
    fn try_from_str_roundtrip(f in finite_float_strategy()) {
        let s = format!("{}", f);
        let result = Float::try_from(s.as_str());
        prop_assert!(result.is_ok());
        if let Ok(float) = result {
            prop_assert_eq!(float.as_f64(), f);
        }
    }
}
