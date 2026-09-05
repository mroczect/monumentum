use monumentum_handler::constants::{MAX_BLOB_SIZE, MAX_TEXT_SIZE};
use monumentum_handler::types::{Blob, Float, Integer, Text};
use proptest as _;

fn ok_blob(data: Vec<u8>) -> Blob {
    Blob::try_new(data).unwrap_or_else(|_| unreachable!())
}

fn ok_float(v: f64) -> Float {
    Float::try_new(v).unwrap_or_else(|_| unreachable!())
}

fn ok_text(s: &str) -> Text {
    Text::try_new(s.to_string()).unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_blob_try_new() {
    assert!(Blob::try_new(Vec::new()).is_ok());
    assert!(Blob::try_new(vec![0; MAX_BLOB_SIZE]).is_ok());
    assert!(Blob::try_new(vec![0; MAX_BLOB_SIZE + 1]).is_err());
}

#[test]
fn test_blob_methods() {
    let data = vec![1, 2, 3];
    let blob = ok_blob(data.clone());
    assert_eq!(blob.as_slice(), &data[..]);
    assert_eq!(blob.len(), 3);
    assert!(!blob.is_empty());
    assert_eq!(blob.to_string(), "Blob(3 bytes)");
}

#[test]
fn test_blob_tryfrom() {
    let data = vec![4, 5];
    let result = Blob::try_from(data.clone());
    assert!(result.is_ok());
    if let Ok(blob) = result {
        assert_eq!(blob.as_slice(), &data[..]);
    }
    let slice: &[u8] = &[6, 7];
    let result = Blob::try_from(slice);
    assert!(result.is_ok());
    if let Ok(blob2) = result {
        assert_eq!(blob2.as_slice(), slice);
    }
    assert!(Blob::try_from(vec![0; MAX_BLOB_SIZE + 1]).is_err());
}

#[test]
fn test_blob_as_ref() {
    let blob = ok_blob(vec![9]);
    let s: &[u8] = blob.as_ref();
    assert_eq!(s, &[9]);
}

#[test]
fn test_float_try_new() {
    assert!(Float::try_new(0.0).is_ok());
    assert!(Float::try_new(-1.5).is_ok());
    assert!(Float::try_new(f64::MAX).is_ok());
    assert!(Float::try_new(f64::MIN_POSITIVE).is_ok());
    assert!(Float::try_new(f64::NAN).is_err());
    assert!(Float::try_new(f64::INFINITY).is_err());
    assert!(Float::try_new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_float_as_f64() {
    let f = ok_float(3.5);
    assert!((f.as_f64() - 3.5).abs() < f64::EPSILON);
}

#[test]
fn test_float_total_cmp() {
    let a = ok_float(1.0);
    let b = ok_float(2.0);
    assert_eq!(a.total_cmp(&b), core::cmp::Ordering::Less);
    assert_eq!(b.total_cmp(&a), core::cmp::Ordering::Greater);
    let neg_zero = ok_float(-0.0);
    let pos_zero = ok_float(0.0);
    assert_eq!(neg_zero.total_cmp(&pos_zero), core::cmp::Ordering::Less);
}

#[test]
fn test_float_le_bytes_roundtrip() {
    let f = ok_float(-123.456);
    let bytes = f.to_le_bytes();
    let result = Float::try_from_le_bytes(bytes);
    assert!(result.is_ok());
    if let Ok(restored) = result {
        assert_eq!(f, restored);
    }
}

#[test]
fn test_float_display() {
    let f = ok_float(2.5);
    assert_eq!(f.to_string(), "2.5");
}

#[test]
fn test_float_tryfrom_f64() {
    assert!(Float::try_from(1.0).is_ok());
    assert!(Float::try_from(f64::NAN).is_err());
}

#[test]
fn test_float_tryfrom_str() {
    assert!(Float::try_from("1.23").is_ok());
    assert!(Float::try_from("abc").is_err());
    assert!(Float::try_from("NaN").is_err());
}

#[test]
fn test_integer_new_and_as() {
    let i = Integer::new(42);
    assert_eq!(i.as_i64(), 42);
}

#[test]
fn test_integer_checked_ops() {
    let a = Integer::new(10);
    let b = Integer::new(3);
    assert_eq!(a.checked_add(b).map(Integer::as_i64), Some(13));
    assert_eq!(a.checked_sub(b).map(Integer::as_i64), Some(7));
    assert_eq!(a.checked_mul(b).map(Integer::as_i64), Some(30));
    assert_eq!(a.checked_div(b).map(Integer::as_i64), Some(3));

    let max = Integer::new(i64::MAX);
    let one = Integer::new(1);
    assert_eq!(max.checked_add(one), None);

    let min = Integer::new(i64::MIN);
    assert_eq!(min.checked_sub(one), None);

    assert_eq!(Integer::new(1).checked_div(Integer::new(0)), None);
}

#[test]
fn test_integer_le_bytes_roundtrip() {
    let i = Integer::new(-123_456_789);
    let bytes = i.to_le_bytes();
    let restored = Integer::from_le_bytes(bytes);
    assert_eq!(i, restored);
}

#[test]
fn test_integer_display() {
    assert_eq!(Integer::new(-5).to_string(), "-5");
    assert_eq!(Integer::new(0).to_string(), "0");
}

#[test]
fn test_integer_from_i64() {
    let i: Integer = 123i64.into();
    assert_eq!(i.as_i64(), 123);
}

#[test]
fn test_integer_tryfrom_str() {
    assert!(Integer::try_from("42").is_ok());
    assert!(Integer::try_from("-42").is_ok());
    assert!(Integer::try_from("abc").is_err());
    assert!(Integer::try_from("999999999999999999999999").is_err());
}

#[test]
fn test_text_try_new() {
    assert!(Text::try_new(String::new()).is_ok());
    assert!(Text::try_new("a".repeat(MAX_TEXT_SIZE)).is_ok());
    assert!(Text::try_new("a".repeat(MAX_TEXT_SIZE + 1)).is_err());
}

#[test]
fn test_text_methods() {
    let t = ok_text("Hello World");
    assert_eq!(t.as_str(), "Hello World");
    assert_eq!(t.len(), 11);
    assert!(!t.is_empty());
    assert_eq!(t.as_bytes(), b"Hello World");

    let lower = t.to_lowercase();
    assert_eq!(lower.as_str(), "hello world");
    let upper = t.to_uppercase();
    assert_eq!(upper.as_str(), "HELLO WORLD");

    assert!(t.contains_ignore_case("world"));
    assert!(t.contains_ignore_case("WORLD"));
    assert!(!t.contains_ignore_case("xyz"));
}

#[test]
fn test_text_display() {
    let t = ok_text("hello");
    assert_eq!(t.to_string(), "hello");
}

#[test]
fn test_text_tryfrom() {
    assert!(Text::try_from("ok".to_string()).is_ok());
    assert!(Text::try_from("ok").is_ok());
    assert!(Text::try_from("a".repeat(MAX_TEXT_SIZE + 1)).is_err());
}

#[test]
fn test_text_as_ref() {
    let t = ok_text("ref");
    let s: &str = t.as_ref();
    assert_eq!(s, "ref");
}
