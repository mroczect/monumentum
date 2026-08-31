use monumentum_db::types::Integer;
use std::collections::HashSet;

#[test]
fn new_basic() {
    let i = Integer::new(42);
    assert_eq!(i.as_i64(), 42);
    let i = Integer::new(-7);
    assert_eq!(i.as_i64(), -7);
    let i = Integer::new(0);
    assert_eq!(i.as_i64(), 0);
    let min = Integer::new(i64::MIN);
    let max = Integer::new(i64::MAX);
    assert_eq!(min.as_i64(), i64::MIN);
    assert_eq!(max.as_i64(), i64::MAX);
}

#[test]
fn checked_add_normal() {
    let a = Integer::new(2);
    let b = Integer::new(3);
    assert_eq!(a.checked_add(b), Some(Integer::new(5)));
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
fn checked_sub_normal() {
    let a = Integer::new(5);
    let b = Integer::new(3);
    assert_eq!(a.checked_sub(b), Some(Integer::new(2)));
}

#[test]
fn checked_sub_underflow() {
    let min = Integer::new(i64::MIN);
    let one = Integer::new(1);
    assert_eq!(min.checked_sub(one), None);
}

#[test]
fn checked_sub_overflow() {
    let max = Integer::new(i64::MAX);
    let neg_one = Integer::new(-1);
    assert_eq!(max.checked_sub(neg_one), None);
}

#[test]
fn checked_mul_normal() {
    let a = Integer::new(2);
    let b = Integer::new(3);
    assert_eq!(a.checked_mul(b), Some(Integer::new(6)));
}

#[test]
fn checked_mul_overflow() {
    let max = Integer::new(i64::MAX);
    let two = Integer::new(2);
    assert_eq!(max.checked_mul(two), None);
    let min = Integer::new(i64::MIN);
    let neg_one = Integer::new(-1);
    assert_eq!(min.checked_mul(neg_one), None);
}

#[test]
fn checked_div_normal() {
    let ten = Integer::new(10);
    let two = Integer::new(2);
    assert_eq!(ten.checked_div(two), Some(Integer::new(5)));
}

#[test]
fn checked_div_by_zero() {
    let one = Integer::new(1);
    let zero = Integer::new(0);
    assert_eq!(one.checked_div(zero), None);
}

#[test]
fn checked_div_overflow() {
    let min = Integer::new(i64::MIN);
    let neg_one = Integer::new(-1);
    assert_eq!(min.checked_div(neg_one), None);
}

#[test]
fn to_le_bytes_roundtrip() {
    let vals = [0, 1, -1, i64::MIN, i64::MAX, 1_234_567_890];
    for v in vals {
        let i = Integer::new(v);
        let bytes = i.to_le_bytes();
        assert_eq!(bytes, v.to_le_bytes());
        let i2 = Integer::from_le_bytes(bytes);
        assert_eq!(i2, i);
    }
}

#[test]
fn from_i64() {
    let i = Integer::from(42);
    assert_eq!(i.as_i64(), 42);
}

#[test]
fn try_from_str_valid() {
    let cases = [
        "42",
        "-123",
        "0",
        "9223372036854775807",
        "-9223372036854775808",
    ];
    for s in cases {
        let i = Integer::try_from(s).unwrap();
        assert_eq!(i.as_i64(), s.parse::<i64>().unwrap());
    }
}

#[test]
fn try_from_str_invalid() {
    let cases = ["abc", "", "12.5", "9999999999999999999999", "1_000"];
    for s in cases {
        assert!(Integer::try_from(s).is_err());
    }
}

#[test]
fn display() {
    assert_eq!(format!("{}", Integer::new(42)), "42");
    assert_eq!(format!("{}", Integer::new(-7)), "-7");
    assert_eq!(format!("{}", Integer::new(0)), "0");
}

#[test]
fn equality_and_order() {
    let a = Integer::new(1);
    let b = Integer::new(1);
    let c = Integer::new(2);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(a < c);
    assert!(c > a);
    assert!(a <= b);
}

#[test]
fn sort_integer() {
    let mut v = [Integer::new(3), Integer::new(1), Integer::new(2)];
    v.sort();
    assert_eq!(v[0].as_i64(), 1);
    assert_eq!(v[1].as_i64(), 2);
    assert_eq!(v[2].as_i64(), 3);
}

#[test]
fn hash_consistent() {
    let mut set = HashSet::new();
    set.insert(Integer::new(5));
    set.insert(Integer::new(5));
    assert_eq!(set.len(), 1);
    set.insert(Integer::new(6));
    assert_eq!(set.len(), 2);
}

#[test]
fn copy_trait() {
    let a = Integer::new(10);
    let b = a;
    assert_eq!(a.as_i64(), 10);
    assert_eq!(b.as_i64(), 10);
}

#[test]
fn debug_format() {
    let i = Integer::new(5);
    assert!(format!("{i:?}").contains('5'));
}

#[test]
fn no_panic_on_extremes() {
    let _ = Integer::new(i64::MIN).checked_add(Integer::new(1));
    let _ = Integer::new(i64::MAX).checked_sub(Integer::new(1));
}
