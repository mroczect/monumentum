use monumentum_handler::validation::{validate_column_name, validate_name, validate_table_name};

#[test]
fn validate_name_valid() {
    assert!(validate_name("hello").is_ok());
    assert!(validate_name("a").is_ok());
    assert!(validate_name(&"x".repeat(255)).is_ok());
}

#[test]
fn validate_name_empty() {
    let result = validate_name("");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            monumentum_handler::error::DbError::InvalidOperation(_)
        ));
    }
}

#[test]
fn validate_name_too_long() {
    let long_name = "x".repeat(256);
    let result = validate_name(&long_name);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            monumentum_handler::error::DbError::InvalidOperation(_)
        ));
    }
}

#[test]
fn validate_name_control_chars() {
    let result = validate_name("bad\nname");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            monumentum_handler::error::DbError::InvalidOperation(_)
        ));
    }
}

#[test]
fn validate_column_name_uses_validate_name() {
    assert!(validate_column_name("valid").is_ok());
    let result = validate_column_name("");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            monumentum_handler::error::DbError::InvalidOperation(_)
        ));
    }
}

#[test]
fn validate_table_name_uses_validate_name() {
    assert!(validate_table_name("valid").is_ok());
    let result = validate_table_name("bad\tname");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            monumentum_handler::error::DbError::InvalidOperation(_)
        ));
    }
}
