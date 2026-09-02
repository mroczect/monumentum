use monumentum_db::error::DbError;
use monumentum_db::types::Integer;

#[test]
fn new_basic_values() {
    assert_eq!(Integer::new(0).as_i64(), 0);
    assert_eq!(Integer::new(42).as_i64(), 42);
    assert_eq!(Integer::new(-7).as_i64(), -7);
}

#[test]
fn as_i64_returns_value() {
    assert_eq!(Integer::new(123).as_i64(), 123);
}

#[test]
fn checked_add_normal() {
    let a = Integer::new(10);
    let b = Integer::new(5);
    assert_eq!(a.checked_add(b), Some(Integer::new(15)));
}

#[test]
fn checked_add_overflow() {
    let max = Integer::new(i64::MAX);
    let one = Integer::new(1);
    assert_eq!(max.checked_add(one), None);
}

#[test]
fn checked_add_underflow() {
    let min = Integer::new(i64::MIN);
    let neg_one = Integer::new(-1);
    assert_eq!(min.checked_add(neg_one), None);
}

#[test]
fn checked_add_zero() {
    let a = Integer::new(5);
    let zero = Integer::new(0);
    assert_eq!(a.checked_add(zero), Some(Integer::new(5)));
}

#[test]
fn checked_sub_normal() {
    let a = Integer::new(10);
    let b = Integer::new(3);
    assert_eq!(a.checked_sub(b), Some(Integer::new(7)));
}

#[test]
fn checked_sub_underflow() {
    let min = Integer::new(i64::MIN);
    let one = Integer::new(1);
    assert_eq!(min.checked_sub(one), None);
}

#[test]
fn checked_sub_negative_result() {
    let a = Integer::new(5);
    let b = Integer::new(10);
    assert_eq!(a.checked_sub(b), Some(Integer::new(-5)));
}

#[test]
fn checked_mul_normal() {
    let a = Integer::new(3);
    let b = Integer::new(4);
    assert_eq!(a.checked_mul(b), Some(Integer::new(12)));
}

#[test]
fn checked_mul_overflow() {
    let max = Integer::new(i64::MAX);
    let two = Integer::new(2);
    assert_eq!(max.checked_mul(two), None);
}

#[test]
fn checked_mul_zero() {
    let a = Integer::new(100);
    let zero = Integer::new(0);
    assert_eq!(a.checked_mul(zero), Some(Integer::new(0)));
}

#[test]
fn checked_div_normal() {
    let ten = Integer::new(10);
    let two = Integer::new(2);
    assert_eq!(ten.checked_div(two), Some(Integer::new(5)));
}

#[test]
fn checked_div_by_zero() {
    let ten = Integer::new(10);
    let zero = Integer::new(0);
    assert_eq!(ten.checked_div(zero), None);
}

#[test]
fn checked_div_truncates() {
    let seven = Integer::new(7);
    let two = Integer::new(2);
    assert_eq!(seven.checked_div(two), Some(Integer::new(3)));
}

#[test]
fn checked_div_min_by_neg_one_overflow() {
    let min = Integer::new(i64::MIN);
    let neg_one = Integer::new(-1);
    assert_eq!(min.checked_div(neg_one), None);
}

#[test]
fn to_le_bytes_roundtrip() {
    let val = 0x0102030405060708_i64;
    let integer = Integer::new(val);
    assert_eq!(integer.to_le_bytes(), val.to_le_bytes());
}

#[test]
fn from_le_bytes_roundtrip() {
    let val = 123456789_i64;
    let bytes = val.to_le_bytes();
    assert_eq!(Integer::from_le_bytes(bytes).as_i64(), val);
}

#[test]
fn display_formats_number() {
    assert_eq!(format!("{}", Integer::new(42)), "42");
    assert_eq!(format!("{}", Integer::new(-1)), "-1");
}

#[test]
fn from_i64() {
    assert_eq!(Integer::from(10).as_i64(), 10);
}

#[test]
fn try_from_str_valid() -> Result<(), DbError> {
    assert_eq!(Integer::try_from("42")?.as_i64(), 42);
    assert_eq!(Integer::try_from("-10")?.as_i64(), -10);
    assert_eq!(Integer::try_from("0")?.as_i64(), 0);
    Ok(())
}

#[test]
fn try_from_str_invalid() {
    assert!(Integer::try_from("").is_err());
    assert!(Integer::try_from("abc").is_err());
}

#[test]
fn try_from_str_overflow() {
    assert!(Integer::try_from("9223372036854775808").is_err());
}
