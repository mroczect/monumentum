use monumentum_handler::constants::MAX_NAME_LENGTH;
use monumentum_handler::error::{ErrorKind, MonumentumError};
use monumentum_handler::validation::{validate_column_name, validate_name, validate_table_name};
use proptest as _;

#[test]
fn test_validate_name_valid() {
    assert!(validate_name("valid_name").is_ok());
    assert!(validate_name("Valid_Name-123").is_ok());
    assert!(validate_name(&"a".repeat(MAX_NAME_LENGTH)).is_ok());
}

#[test]
fn test_validate_name_empty() {
    let result = validate_name("");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.kind(), ErrorKind::InvalidOperation);
        assert!(err.to_string().contains("name cannot be empty"));
    }
}

#[test]
fn test_validate_name_too_long() {
    let long = "a".repeat(MAX_NAME_LENGTH + 1);
    let result = validate_name(&long);
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.kind(), ErrorKind::InvalidOperation);
        assert!(err.to_string().contains("name too long"));
    }
}

#[test]
fn test_validate_name_control_chars() {
    assert!(validate_name("bad\nname").is_err());
    assert!(validate_name("bad\tname").is_err());
    assert!(validate_name("bad\0name").is_err());
}

#[test]
fn test_validate_column_name() {
    assert!(validate_column_name("col").is_ok());
    let result = validate_column_name("");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.kind(), ErrorKind::InvalidOperation);
        assert!(err.to_string().contains("invalid column name"));
    }
}

#[test]
fn test_validate_table_name() {
    assert!(validate_table_name("table").is_ok());
    let result = validate_table_name("bad\nname");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.kind(), ErrorKind::InvalidOperation);
        assert!(err.to_string().contains("invalid table name"));
    }
}
