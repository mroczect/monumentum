#![allow(unused_crate_dependencies)]

//! Audit pengurutan & serialisasi `IndexKey`.

use monumentum_core::index::key::IndexKey;
use monumentum_handler::Value;

#[test]
fn test_integer_key_ord_natural() {
    assert!(IndexKey::Integer(-10) < IndexKey::Integer(0));
    assert!(IndexKey::Integer(0) < IndexKey::Integer(10));
    assert!(IndexKey::Integer(-100) < IndexKey::Integer(-1));
}

#[test]
fn test_float_key_ord_positive() {
    let mut keys: Vec<_> = [1.0f64, 2.0, 3.0, 100.0]
        .into_iter()
        .map(|f| IndexKey::Float(f.to_bits()))
        .collect();
    let sorted_before = keys.clone();
    keys.sort();
    assert_eq!(keys, sorted_before);
}

#[test]
fn test_float_key_ord_negative() {
    let a = IndexKey::Float((-1.0f64).to_bits());
    let b = IndexKey::Float((-2.0f64).to_bits());
    assert!(
        b < a,
        "H1 CONFIRMED: -2.0 harus < -1.0, tapi bit-order rusak"
    );
}

#[test]
fn test_float_key_ord_mixed_sign() {
    let neg = IndexKey::Float((-1.0f64).to_bits());
    let pos = IndexKey::Float((1.0f64).to_bits());
    assert!(neg < pos, "H1 CONFIRMED: -1.0 harus < 1.0");
}

#[test]
fn test_key_roundtrip_integer() -> Result<(), monumentum_handler::DbError> {
    for i in [-1_000_000i64, -1, 0, 1, 1_000_000] {
        let k = IndexKey::Integer(i);
        assert_eq!(IndexKey::from_bytes(&k.to_bytes()?)?, k);
    }
    Ok(())
}

#[test]
fn test_key_roundtrip_text() -> Result<(), monumentum_handler::DbError> {
    for s in ["", "a", "hello", "héllo", "日本語"] {
        let k = IndexKey::Text(s.to_string());
        assert_eq!(IndexKey::from_bytes(&k.to_bytes()?)?, k);
    }
    Ok(())
}

#[test]
fn test_key_roundtrip_blob() -> Result<(), monumentum_handler::DbError> {
    for b in [vec![], vec![0u8], vec![0u8; 100], vec![0xFFu8; 1000]] {
        let k = IndexKey::Blob(b);
        assert_eq!(IndexKey::from_bytes(&k.to_bytes()?)?, k);
    }
    Ok(())
}

#[test]
fn test_key_from_value_null() {
    assert!(IndexKey::from_value(&Value::Null).is_none());
}

#[test]
fn test_key_from_value_neg_zero_normalized() {
    let pos = IndexKey::from_value(&Value::try_from(0.0).unwrap_or_else(|_| unreachable!()));
    let neg = IndexKey::from_value(&Value::try_from(-0.0).unwrap_or_else(|_| unreachable!()));
    assert_eq!(pos, neg);
}

#[test]
fn test_key_decode_rejects_invalid_tag() {
    assert!(IndexKey::from_bytes(&[0xFF]).is_err());
}

#[test]
fn test_key_decode_rejects_truncated() {
    assert!(IndexKey::from_bytes(&[0]).is_err());
    assert!(IndexKey::from_bytes(&[2, 0x10, 0, 0, 0]).is_err());
}
