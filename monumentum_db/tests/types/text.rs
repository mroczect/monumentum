use monumentum_db::types::Text;
use std::collections::HashSet;

#[test]
fn new_empty() {
    let t = Text::new(String::new());
    assert_eq!(t.len(), 0);
    assert!(t.is_empty());
}

#[test]
fn new_non_empty() {
    let t = Text::new("hello".to_string());
    assert_eq!(t.len(), 5);
    assert!(!t.is_empty());
    assert_eq!(t.as_str(), "hello");
}

#[test]
fn as_str() {
    let t = Text::new("data".to_string());
    assert_eq!(t.as_str(), "data");
}

#[test]
fn len_multibyte() {
    let t = Text::new("a😀".to_string());
    assert_eq!(t.len(), 5);
}

#[test]
fn is_empty() {
    let t = Text::new(String::new());
    assert!(t.is_empty());
    let t = Text::new("x".to_string());
    assert!(!t.is_empty());
}

#[test]
fn to_lowercase_ascii() {
    let t = Text::new("HELLO".to_string());
    assert_eq!(t.to_lowercase().as_str(), "hello");
}

#[test]
fn to_lowercase_mixed() {
    let t = Text::new("HeLLo".to_string());
    assert_eq!(t.to_lowercase().as_str(), "hello");
}

#[test]
fn to_lowercase_unicode() {
    let t = Text::new("ÄBC".to_string());
    assert_eq!(t.to_lowercase().as_str(), "äbc");
}

#[test]
fn to_lowercase_empty() {
    let t = Text::new(String::new());
    assert_eq!(t.to_lowercase().as_str(), "");
}

#[test]
fn to_uppercase_ascii() {
    let t = Text::new("hello".to_string());
    assert_eq!(t.to_uppercase().as_str(), "HELLO");
}

#[test]
fn to_uppercase_mixed() {
    let t = Text::new("HeLLo".to_string());
    assert_eq!(t.to_uppercase().as_str(), "HELLO");
}

#[test]
fn to_uppercase_unicode() {
    let t = Text::new("äbc".to_string());
    assert_eq!(t.to_uppercase().as_str(), "ÄBC");
}

#[test]
fn to_uppercase_empty() {
    let t = Text::new(String::new());
    assert_eq!(t.to_uppercase().as_str(), "");
}

#[test]
fn contains_ignore_case_ascii() {
    let t = Text::new("Hello World".to_string());
    assert!(t.contains_ignore_case("hello"));
    assert!(t.contains_ignore_case("WORLD"));
    assert!(!t.contains_ignore_case("xyz"));
}

#[test]
fn contains_ignore_case_unicode_simple() {
    let t = Text::new("Äpfel".to_string());
    assert!(t.contains_ignore_case("äpfel"));
    assert!(t.contains_ignore_case("ÄPFEL"));
    assert!(!t.contains_ignore_case("xyz"));
}

#[test]
fn contains_ignore_case_empty_needle() {
    let t = Text::new("abc".to_string());
    assert!(t.contains_ignore_case(""));
}

#[test]
fn as_bytes() {
    let t = Text::new("hello".to_string());
    assert_eq!(t.as_bytes(), b"hello");
    let t = Text::new("😀".to_string());
    assert_eq!(t.as_bytes(), &[0xF0, 0x9F, 0x98, 0x80]);
    let t = Text::new(String::new());
    assert!(t.as_bytes().is_empty());
}

#[test]
fn display() {
    let t = Text::new("hello".to_string());
    assert_eq!(format!("{t}"), "hello");
    let t = Text::new("line\nbreak".to_string());
    assert_eq!(format!("{t}"), "line\nbreak");
    let t = Text::new("😀".to_string());
    assert_eq!(format!("{t}"), "😀");
}

#[test]
fn from_string() {
    let t = Text::from(String::from("data"));
    assert_eq!(t.as_str(), "data");
}

#[test]
fn from_str() {
    let t = Text::from("data");
    assert_eq!(t.as_str(), "data");
    let t = Text::from("");
    assert!(t.is_empty());
}

#[test]
fn as_ref_str() {
    let t = Text::new("hello".to_string());
    let s: &str = t.as_ref();
    assert_eq!(s, "hello");
}

#[test]
fn equality_and_order() {
    let a = Text::new("apple".to_string());
    let b = Text::new("apple".to_string());
    let c = Text::new("banana".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(a < c);
    assert!(c > a);
    let empty = Text::new(String::new());
    assert!(empty < a);
}

#[test]
fn hash_consistent() {
    let mut set = HashSet::new();
    set.insert(Text::new("same".to_string()));
    set.insert(Text::new("same".to_string()));
    assert_eq!(set.len(), 1);
    set.insert(Text::new("different".to_string()));
    assert_eq!(set.len(), 2);
}

#[test]
fn clone_independent() {
    let t1 = Text::new("original".to_string());
    let t2 = t1.clone();
    assert_eq!(t1, t2);
    let t3 = t1.to_uppercase();
    assert_ne!(t1, t3);
    assert_eq!(t1.as_str(), "original");
    let empty = Text::new(String::new());
    let empty_clone = empty.clone();
    assert_eq!(empty, empty_clone);
}

#[test]
fn long_string() {
    let s = "a".repeat(1_000_000);
    let t = Text::new(s.clone());
    assert_eq!(t.len(), s.len());
    assert_eq!(t.as_str(), s);
}

#[test]
fn all_character_types() {
    let s = "Hello, 世界! 😀\n\t\0";
    let t = Text::new(s.to_string());
    assert_eq!(t.as_str(), s);
    assert_eq!(format!("{t}"), s);
}
