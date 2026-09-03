use monumentum_db::types::Text;
use proptest::prelude::*;

#[test]
fn new_empty_string() {
    let text = Text::new(String::new());
    assert_eq!(text.len(), 0);
    assert!(text.is_empty());
    assert_eq!(text.as_str(), "");
}

#[test]
fn new_non_empty_string() {
    let text = Text::new("hello".to_string());
    assert_eq!(text.len(), 5);
    assert!(!text.is_empty());
    assert_eq!(text.as_str(), "hello");
}

#[test]
fn as_str_returns_slice() {
    let text = Text::new("rust".to_string());
    assert_eq!(text.as_str(), "rust");
}

#[test]
fn len_returns_byte_length() {
    assert_eq!(Text::new("hello".to_string()).len(), 5);
    assert_eq!(Text::new("é".to_string()).len(), 2);
}

#[test]
fn is_empty_returns_true_only_for_empty() {
    assert!(Text::new(String::new()).is_empty());
    assert!(!Text::new("a".to_string()).is_empty());
}

#[test]
fn to_lowercase_ascii() {
    let text = Text::new("HeLLo".to_string());
    assert_eq!(text.to_lowercase().as_str(), "hello");
}

#[test]
fn to_lowercase_unicode() {
    let text = Text::new("ÄBC".to_string());
    assert_eq!(text.to_lowercase().as_str(), "äbc");
}

#[test]
fn to_uppercase_ascii() {
    let text = Text::new("hello".to_string());
    assert_eq!(text.to_uppercase().as_str(), "HELLO");
}

#[test]
fn contains_ignore_case_true() {
    let text = Text::new("Hello World".to_string());
    assert!(text.contains_ignore_case("WORLD"));
    assert!(text.contains_ignore_case("hello"));
}

#[test]
fn contains_ignore_case_false() {
    let text = Text::new("Hello".to_string());
    assert!(!text.contains_ignore_case("xyz"));
}

#[test]
fn contains_ignore_case_empty_needle() {
    let text = Text::new("abc".to_string());
    assert!(text.contains_ignore_case(""));
}

#[test]
fn contains_ignore_case_mixed_case() {
    let text = Text::new("Rust Programming".to_string());
    assert!(text.contains_ignore_case("rUsT"));
}

#[test]
fn as_bytes_returns_slice() {
    let text = Text::new("abc".to_string());
    assert_eq!(text.as_bytes(), b"abc");
}

#[test]
fn display_formats_plain() {
    assert_eq!(format!("{}", Text::new("hello".to_string())), "hello");
}

#[test]
fn from_string() {
    let text = Text::from("hello".to_string());
    assert_eq!(text.as_str(), "hello");
}

#[test]
fn from_str() {
    let text = Text::from("world");
    assert_eq!(text.as_str(), "world");
}

#[test]
fn as_ref_str() {
    let text = Text::new("data".to_string());
    let r: &str = text.as_ref();
    assert_eq!(r, "data");
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    #[test]
    fn text_new_and_accessors_roundtrip(s in ".*") {
        let text = Text::new(s.clone());
        prop_assert_eq!(text.as_str(), s.as_str());
        prop_assert_eq!(text.len(), s.len());
        prop_assert_eq!(text.is_empty(), s.is_empty());
        prop_assert_eq!(text.as_bytes(), s.as_bytes());
    }

    #[test]
    fn text_lowercase_uppercase_roundtrip(s in ".*") {
        let text = Text::new(s.clone());
        let lower = text.to_lowercase();
        let upper = text.to_uppercase();
        prop_assert_eq!(lower.as_str().to_lowercase(), lower.as_str());
        prop_assert_eq!(upper.as_str().to_uppercase(), upper.as_str());
    }

    #[test]
    fn text_contains_ignore_case_reflexive(s in ".*") {
        let text = Text::new(s.clone());
        prop_assert!(text.contains_ignore_case(&s));
    }

    #[test]
    fn text_from_str_and_string_equality(s in ".*") {
        let from_str = Text::from(s.as_str());
        let from_string = Text::from(s.clone());
        prop_assert_eq!(from_str, from_string);
    }

    #[test]
    fn text_clone_equality(s in ".*") {
        let text = Text::new(s);
        let cloned = text.clone();
        prop_assert_eq!(text, cloned);
    }
}
