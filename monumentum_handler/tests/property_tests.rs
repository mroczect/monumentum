use monumentum_handler::constants::{MAX_BLOB_SIZE, MAX_NAME_LENGTH, MAX_TEXT_SIZE};
use monumentum_handler::core::value::Value;
use monumentum_handler::types::{Blob, Integer, Text};
use monumentum_handler::validation::validate_name;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

fn valid_string(min_len: usize, max_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::range('a', 'z'), min_len..max_len)
        .prop_map(|v| v.into_iter().collect::<String>())
}

fn valid_name() -> impl Strategy<Value = String> {
    let first = proptest::char::range('a', 'z');
    let rest = proptest::collection::vec(
        proptest::char::range('a', 'z'),
        0..MAX_NAME_LENGTH.saturating_sub(1),
    );
    (first, rest).prop_map(|(first_char, rest_chars)| {
        let mut s = String::new();
        s.push(first_char);
        s.extend(rest_chars);
        s
    })
}

fn too_long_name() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        proptest::char::range('a', 'z'),
        (MAX_NAME_LENGTH + 1)..(MAX_NAME_LENGTH + 100),
    )
    .prop_map(|v| v.into_iter().collect::<String>())
}

fn name_with_control() -> impl Strategy<Value = String> {
    let control = proptest::sample::select(vec!['\n', '\t', '\0']);
    let prefix = valid_string(0, 10);
    let suffix = valid_string(0, 10);
    (prefix, control, suffix).prop_map(|(p, c, s)| format!("{p}{c}{s}"))
}

proptest! {
    #[test]
    fn prop_integer_roundtrip(i in any::<i64>()) {
        let int = Integer::new(i);
        prop_assert_eq!(int.as_i64(), i);

        let val = Value::from(i);
        prop_assert_eq!(val.as_i64(), Some(i));
    }

    #[test]
    fn prop_text_roundtrip(s in valid_string(0, 1000)) {
        let text = Text::try_new(s.clone())
            .map_err(|e| TestCaseError::fail(format!("Text::try_new failed: {e}")))?;
        prop_assert_eq!(text.as_str(), s.as_str());

        let val = Value::from(text);
        prop_assert_eq!(val.as_str(), Some(s.as_str()));
    }

    #[test]
    fn prop_blob_roundtrip(v in proptest::collection::vec(any::<u8>(), 0..1000)) {
        let blob = Blob::try_new(v.clone())
            .map_err(|e| TestCaseError::fail(format!("Blob::try_new failed: {e}")))?;
        prop_assert_eq!(blob.as_slice(), v.as_slice());

        let val = Value::from(blob);
        let extracted = val.as_blob()
            .ok_or_else(|| TestCaseError::fail("expected blob"))?;
        prop_assert_eq!(extracted.as_slice(), v.as_slice());
    }

    #[test]
    fn prop_validate_name_valid(name in valid_name()) {
        prop_assert!(validate_name(&name).is_ok());
    }

    #[test]
    fn prop_validate_name_too_long(name in too_long_name()) {
        prop_assert!(validate_name(&name).is_err());
    }

    #[test]
    fn prop_validate_name_control_char(name in name_with_control()) {
        prop_assert!(validate_name(&name).is_err());
    }

    #[test]
    fn prop_checked_ops(a in any::<i64>(), b in any::<i64>()) {
        let ia = Integer::new(a);
        let ib = Integer::new(b);

        let expected_add = a.checked_add(b).map(Integer::new);
        prop_assert_eq!(ia.checked_add(ib), expected_add);

        let expected_sub = a.checked_sub(b).map(Integer::new);
        prop_assert_eq!(ia.checked_sub(ib), expected_sub);

        let expected_mul = a.checked_mul(b).map(Integer::new);
        prop_assert_eq!(ia.checked_mul(ib), expected_mul);

        if b != 0 {
            let expected_div = a.checked_div(b).map(Integer::new);
            prop_assert_eq!(ia.checked_div(ib), expected_div);
        }
    }
}

#[test]
fn test_size_limits_text() {
    let ok_text = "a".repeat(MAX_TEXT_SIZE);
    assert!(Text::try_new(ok_text).is_ok());

    let too_big_text = "a".repeat(MAX_TEXT_SIZE + 1);
    assert!(Text::try_new(too_big_text).is_err());
}

#[test]
fn test_size_limits_blob() {
    let ok_blob = vec![0u8; MAX_BLOB_SIZE];
    assert!(Blob::try_new(ok_blob).is_ok());

    let too_big_blob = vec![0u8; MAX_BLOB_SIZE + 1];
    assert!(Blob::try_new(too_big_blob).is_err());
}
