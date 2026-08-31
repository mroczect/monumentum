#![allow(clippy::float_cmp)]

use monumentum_db::error::DbError;
use monumentum_db::types::Float;
use std::cmp::Ordering;

#[test]
fn try_new_finite() {
    let f = Float::try_new(2.5).unwrap();
    assert_eq!(f.as_f64(), 2.5);
    assert!(Float::try_new(0.0).is_ok());
    assert!(Float::try_new(-0.0).is_ok());
    assert!(Float::try_new(f64::MAX).is_ok());
    assert!(Float::try_new(f64::MIN_POSITIVE).is_ok());
}

#[test]
fn try_new_rejects_non_finite() {
    assert!(matches!(
        Float::try_new(f64::NAN),
        Err(DbError::TypeMismatch(_))
    ));
    assert!(matches!(
        Float::try_new(f64::INFINITY),
        Err(DbError::TypeMismatch(_))
    ));
    assert!(matches!(
        Float::try_new(f64::NEG_INFINITY),
        Err(DbError::TypeMismatch(_))
    ));
    if let Err(e) = Float::try_new(f64::NAN) {
        assert!(e.to_string().contains("float must be finite"));
    }
}

#[test]
fn as_f64() {
    let f = Float::try_new(2.5).unwrap();
    assert_eq!(f.as_f64(), 2.5);
    let f = Float::try_new(-1.25).unwrap();
    assert_eq!(f.as_f64(), -1.25);
    let f = Float::try_new(0.0).unwrap();
    assert_eq!(f.as_f64(), 0.0);
}

#[test]
fn total_cmp_equal() {
    let a = Float::try_new(1.5).unwrap();
    let b = Float::try_new(1.5).unwrap();
    assert_eq!(a.total_cmp(&b), Ordering::Equal);
}

#[test]
fn total_cmp_less_greater() {
    let a = Float::try_new(1.0).unwrap();
    let b = Float::try_new(2.0).unwrap();
    assert_eq!(a.total_cmp(&b), Ordering::Less);
    assert_eq!(b.total_cmp(&a), Ordering::Greater);
    let neg = Float::try_new(-1.0).unwrap();
    assert_eq!(neg.total_cmp(&a), Ordering::Less);
}

#[test]
fn total_cmp_negative_zero() {
    let neg_zero = Float::try_new(-0.0).unwrap();
    let pos_zero = Float::try_new(0.0).unwrap();
    assert_eq!(neg_zero.total_cmp(&pos_zero), Ordering::Less);
}

#[test]
fn total_cmp_close_values() {
    let a = Float::try_new(1.0).unwrap();
    let b = Float::try_new(1.000_000_000_000_000_2).unwrap();
    assert!(a.total_cmp(&b) != Ordering::Equal);
}

#[test]
fn to_le_bytes_roundtrip() {
    let vals = [
        0.0,
        -0.0,
        1.5,
        -3.25,
        f64::MAX,
        f64::MIN_POSITIVE,
        12_345.678_9,
    ];
    for v in vals {
        let f = Float::try_new(v).unwrap();
        let bytes = f.to_le_bytes();
        let f2 = Float::try_from_le_bytes(bytes).unwrap();
        assert_eq!(f.total_cmp(&f2), Ordering::Equal);
        assert_eq!(bytes, v.to_le_bytes());
    }
}

#[test]
fn try_from_le_bytes_rejects_non_finite() {
    let nan_bytes = f64::NAN.to_le_bytes();
    assert!(matches!(
        Float::try_from_le_bytes(nan_bytes),
        Err(DbError::TypeMismatch(_))
    ));
    let inf_bytes = f64::INFINITY.to_le_bytes();
    assert!(matches!(
        Float::try_from_le_bytes(inf_bytes),
        Err(DbError::TypeMismatch(_))
    ));
    let neg_inf = f64::NEG_INFINITY.to_le_bytes();
    assert!(matches!(
        Float::try_from_le_bytes(neg_inf),
        Err(DbError::TypeMismatch(_))
    ));
}

#[test]
fn try_from_f64() {
    assert!(Float::try_from(1.0).is_ok());
    assert!(Float::try_from(f64::NAN).is_err());
}

#[test]
fn try_from_str() {
    let ok_cases = ["1.5", "-2.0", "0", "2.5", "1e10"];
    for s in ok_cases {
        let f = Float::try_from(s).unwrap();
        assert!(f.as_f64().is_finite());
    }
    let err_cases = ["abc", "", "1.2.3", "NaN", "inf", "-inf"];
    for s in err_cases {
        assert!(Float::try_from(s).is_err());
    }
}

#[test]
fn display() {
    let f = Float::try_new(2.5).unwrap();
    assert_eq!(f.as_f64(), 2.5);
    let f = Float::try_new(-2.5).unwrap();
    assert_eq!(format!("{f}"), "-2.5");
    let f = Float::try_new(0.0).unwrap();
    assert_eq!(format!("{f}"), "0");
}

#[test]
fn partial_eq() {
    let a = Float::try_new(1.0).unwrap();
    let b = Float::try_new(1.0).unwrap();
    assert_eq!(a, b);
    let c = Float::try_new(2.0).unwrap();
    assert_ne!(a, c);
    let neg_zero = Float::try_new(-0.0).unwrap();
    let pos_zero = Float::try_new(0.0).unwrap();
    assert_eq!(neg_zero, pos_zero);
}

#[test]
fn copy_trait() {
    let a = Float::try_new(5.5).unwrap();
    let b = a;
    assert_eq!(a.as_f64(), 5.5);
    assert_eq!(b.as_f64(), 5.5);
}

#[test]
fn debug_format() {
    let f = Float::try_new(2.0).unwrap();
    let s = format!("{f:?}");
    assert!(s.contains('2'));
}
