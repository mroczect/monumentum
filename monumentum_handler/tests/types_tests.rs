use monumentum_handler::error::DbError;
use monumentum_handler::types::{Blob, Float, Integer, Text};

#[test]
fn text_new_and_accessors() {
    let text = Text::new("hello".to_string());
    assert_eq!(text.as_str(), "hello");
    assert_eq!(text.len(), 5);
    assert!(!text.is_empty());
    assert_eq!(text.to_uppercase().as_str(), "HELLO");
    assert_eq!(text.to_lowercase().as_str(), "hello");
    assert!(text.contains_ignore_case("ELL"));
    assert_eq!(text.as_bytes(), b"hello");
}

#[test]
fn text_try_new_enforces_max_size() {
    let big = "x".repeat(monumentum_handler::constants::MAX_TEXT_SIZE + 1);
    let result = Text::try_new(big);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
    }
}

#[test]
fn blob_new_and_accessors() {
    let data = vec![1, 2, 3, 4];
    let blob = Blob::new(data.clone());
    assert_eq!(blob.as_slice(), &data[..]);
    assert_eq!(blob.len(), 4);
    assert!(!blob.is_empty());
}

#[test]
fn blob_try_new_enforces_max_size() {
    let big = vec![0u8; monumentum_handler::constants::MAX_BLOB_SIZE + 1];
    let result = Blob::try_new(big);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
    }
}

#[test]
fn float_try_new_rejects_non_finite() {
    assert!(Float::try_new(f64::NAN).is_err());
    assert!(Float::try_new(f64::INFINITY).is_err());
    assert!(Float::try_new(f64::NEG_INFINITY).is_err());
    assert!(Float::try_new(0.0).is_ok());
    assert!(Float::try_new(-1.5).is_ok());
}

#[test]
fn integer_checked_operations() {
    let a = Integer::new(10);
    let b = Integer::new(3);
    assert_eq!(a.checked_add(b), Some(Integer::new(13)));
    assert_eq!(a.checked_sub(b), Some(Integer::new(7)));
    assert_eq!(a.checked_mul(b), Some(Integer::new(30)));
    assert_eq!(a.checked_div(b), Some(Integer::new(3)));
    assert_eq!(Integer::new(i64::MAX).checked_add(Integer::new(1)), None);
}
