use core::error::Error;
use monumentum_db::error::{DbError, ErrorKind, MonumentumError};
use proptest::prelude::*;
use std::io;
type ConstraintViolationFields = (ErrorKind, String, Option<String>, Option<String>);

fn expect_table_not_found(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::TableNotFound(name) => Ok(name),
        _ => Err("expected TableNotFound variant"),
    }
}

fn expect_column_not_found(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::ColumnNotFound(name) => Ok(name),
        _ => Err("expected ColumnNotFound variant"),
    }
}

fn expect_type_mismatch(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::TypeMismatch(msg) => Ok(msg),
        _ => Err("expected TypeMismatch variant"),
    }
}

fn expect_invalid_operation(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::InvalidOperation(msg) => Ok(msg),
        _ => Err("expected InvalidOperation variant"),
    }
}

fn expect_invalid_query(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::InvalidQuery(msg) => Ok(msg),
        _ => Err("expected InvalidQuery variant"),
    }
}

fn expect_unsupported(err: DbError) -> Result<String, &'static str> {
    match err {
        DbError::Unsupported(msg) => Ok(msg),
        _ => Err("expected Unsupported variant"),
    }
}

fn expect_constraint_violation(err: DbError) -> Result<ConstraintViolationFields, &'static str> {
    match err {
        DbError::ConstraintViolation {
            kind,
            message,
            constraint,
            table,
        } => Ok((kind, message, constraint, table)),
        _ => Err("expected ConstraintViolation variant"),
    }
}

#[test]
fn table_not_found_constructor_preserves_name() -> Result<(), &'static str> {
    let name = expect_table_not_found(DbError::table_not_found("users"))?;
    assert_eq!(name, "users");
    Ok(())
}

#[test]
fn column_not_found_constructor_preserves_name() -> Result<(), &'static str> {
    let name = expect_column_not_found(DbError::column_not_found("id"))?;
    assert_eq!(name, "id");
    Ok(())
}

#[test]
fn type_mismatch_constructor_preserves_message() -> Result<(), &'static str> {
    let msg = expect_type_mismatch(DbError::type_mismatch("expected INTEGER"))?;
    assert_eq!(msg, "expected INTEGER");
    Ok(())
}

#[test]
fn invalid_operation_constructor_preserves_message() -> Result<(), &'static str> {
    let msg = expect_invalid_operation(DbError::invalid_operation("table name empty"))?;
    assert_eq!(msg, "table name empty");
    Ok(())
}

#[test]
fn invalid_query_constructor_preserves_message() -> Result<(), &'static str> {
    let msg = expect_invalid_query(DbError::invalid_query("syntax error near WHERE"))?;
    assert_eq!(msg, "syntax error near WHERE");
    Ok(())
}

#[test]
fn unsupported_constructor_preserves_message() -> Result<(), &'static str> {
    let msg = expect_unsupported(DbError::unsupported("JOIN not implemented"))?;
    assert_eq!(msg, "JOIN not implemented");
    Ok(())
}

#[test]
fn constraint_violation_constructor_preserves_all_fields() -> Result<(), &'static str> {
    let err = DbError::constraint_violation(
        ErrorKind::CheckViolation,
        "check failed",
        Some("positive_age".to_string()),
        Some("users".to_string()),
    );
    let (kind, message, constraint, table) = expect_constraint_violation(err)?;
    assert_eq!(kind, ErrorKind::CheckViolation);
    assert_eq!(message, "check failed");
    assert_eq!(constraint.as_deref(), Some("positive_age"));
    assert_eq!(table.as_deref(), Some("users"));
    Ok(())
}

#[test]
fn display_constraint_violation() {
    let err = DbError::constraint_violation(
        ErrorKind::NotNullViolation,
        "column cannot be null",
        Some("not_null_users_id".to_string()),
        Some("users".to_string()),
    );
    assert_eq!(
        format!("{err}"),
        "Constraint violation: column cannot be null"
    );
}

#[test]
fn display_io_error_wraps_message() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let db_err = DbError::from(io_err);
    assert_eq!(format!("{db_err}"), "I/O error: access denied");
}

#[test]
fn display_corruption_shows_source_message() {
    let inner = io::Error::new(io::ErrorKind::InvalidData, "checksum mismatch");
    let err = DbError::corruption(inner);
    assert_eq!(format!("{err}"), "Data corruption: checksum mismatch");
}

#[test]
fn display_transaction_shows_source_message() {
    let inner = io::Error::other("rollback required");
    let err = DbError::transaction(inner);
    assert_eq!(format!("{err}"), "Transaction error: rollback required");
}

#[test]
fn source_is_none_for_constraint_violation() {
    let err = DbError::constraint_violation(ErrorKind::CheckViolation, "check failed", None, None);
    assert!(err.source().is_none());
}

#[test]
fn source_is_some_for_corruption_wrapped_io() {
    let inner = io::Error::new(io::ErrorKind::InvalidData, "corrupt");
    let err = DbError::corruption(inner);
    assert!(err.source().is_some());
}

#[test]
fn source_is_some_for_transaction_wrapped_io() {
    let inner = io::Error::other("txn");
    let err = DbError::transaction(inner);
    assert!(err.source().is_some());
}

#[test]
fn clone_corruption_error_preserves_source() {
    let inner = io::Error::new(io::ErrorKind::InvalidData, "corrupt");
    let err = DbError::corruption(inner);
    let cloned = err.clone();
    assert_eq!(format!("{err}"), format!("{cloned}"));
}

#[test]
fn clone_constraint_violation_preserves_fields() {
    let err = DbError::constraint_violation(
        ErrorKind::UniqueViolation,
        "duplicate key",
        Some("unique_idx".to_string()),
        Some("users".to_string()),
    );
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn clone_io_error_preserves_kind_and_message() -> Result<(), &'static str> {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let db_err = DbError::from(io_err);
    let cloned = db_err.clone();
    match (&db_err, &cloned) {
        (DbError::Io(a), DbError::Io(b)) => {
            assert_eq!(a.kind(), b.kind());
            assert_eq!(a.to_string(), b.to_string());
        }
        _ => return Err("expected Io variant"),
    }
    Ok(())
}

#[test]
fn eq_corruption_same_message() {
    let e1 = DbError::corruption(io::Error::new(io::ErrorKind::InvalidData, "bad"));
    let e2 = DbError::corruption(io::Error::other("bad"));
    assert_eq!(e1, e2);
}

#[test]
fn eq_transaction_same_message() {
    let e1 = DbError::transaction(io::Error::other("deadlock"));
    let e2 = DbError::transaction(io::Error::other("deadlock"));
    assert_eq!(e1, e2);
}

#[test]
fn ne_different_variants() {
    assert_ne!(
        DbError::table_not_found("t"),
        DbError::column_not_found("c")
    );
}

#[test]
fn kind_for_all_variants() {
    assert_eq!(DbError::Io(io::Error::other("io")).kind(), ErrorKind::Io);
    assert_eq!(
        DbError::Corruption(std::sync::Arc::new(io::Error::other("corr"))).kind(),
        ErrorKind::Corruption
    );
    assert_eq!(
        DbError::table_not_found("t").kind(),
        ErrorKind::InvalidOperation
    );
    assert_eq!(
        DbError::column_not_found("c").kind(),
        ErrorKind::InvalidOperation
    );
    assert_eq!(DbError::type_mismatch("tm").kind(), ErrorKind::TypeMismatch);
    assert_eq!(
        DbError::invalid_operation("io").kind(),
        ErrorKind::InvalidOperation
    );
    assert_eq!(DbError::invalid_query("iq").kind(), ErrorKind::InvalidQuery);
    assert_eq!(
        DbError::Transaction(std::sync::Arc::new(io::Error::other("txn"))).kind(),
        ErrorKind::Other
    );
    assert_eq!(DbError::unsupported("u").kind(), ErrorKind::Unsupported);
    assert_eq!(
        DbError::constraint_violation(ErrorKind::CheckViolation, "c", None, None).kind(),
        ErrorKind::CheckViolation
    );
}

#[test]
fn message_for_all_variants() {
    assert_eq!(DbError::Io(io::Error::other("io")).message(), "I/O error");
    assert_eq!(
        DbError::Corruption(std::sync::Arc::new(io::Error::other("corr"))).message(),
        "Data corruption"
    );
    assert_eq!(DbError::table_not_found("users").message(), "users");
    assert_eq!(DbError::column_not_found("id").message(), "id");
    assert_eq!(DbError::type_mismatch("tm").message(), "tm");
    assert_eq!(DbError::invalid_operation("io").message(), "io");
    assert_eq!(DbError::invalid_query("iq").message(), "iq");
    assert_eq!(
        DbError::Transaction(std::sync::Arc::new(io::Error::other("txn"))).message(),
        "Transaction error"
    );
    assert_eq!(DbError::unsupported("u").message(), "u");
    assert_eq!(
        DbError::constraint_violation(ErrorKind::CheckViolation, "check", None, None).message(),
        "check"
    );
}

#[test]
fn constraint_and_table_default_none_for_non_constraint_violation() {
    assert_eq!(DbError::table_not_found("t").constraint(), None);
    assert_eq!(DbError::column_not_found("c").table(), None);
    assert_eq!(DbError::type_mismatch("tm").constraint(), None);
}

#[test]
fn constraint_and_table_for_constraint_violation() {
    let err = DbError::constraint_violation(
        ErrorKind::UniqueViolation,
        "dup",
        Some("unique_idx".to_string()),
        Some("users".to_string()),
    );
    assert_eq!(err.constraint(), Some("unique_idx"));
    assert_eq!(err.table(), Some("users"));
}

#[test]
fn is_type_mismatch_method() {
    assert!(DbError::type_mismatch("tm").is_type_mismatch());
    assert!(!DbError::invalid_operation("io").is_type_mismatch());
}

#[test]
fn is_unique_violation_method() {
    let err = DbError::constraint_violation(ErrorKind::UniqueViolation, "dup", None, None);
    assert!(err.is_unique_violation());
    assert!(!DbError::type_mismatch("tm").is_unique_violation());
}

#[test]
fn is_foreign_key_violation_method() {
    let err = DbError::constraint_violation(ErrorKind::ForeignKeyViolation, "fk", None, None);
    assert!(err.is_foreign_key_violation());
}

#[test]
fn is_not_null_violation_method() {
    let err = DbError::constraint_violation(ErrorKind::NotNullViolation, "nn", None, None);
    assert!(err.is_not_null_violation());
}

#[test]
fn is_check_violation_method() {
    let err = DbError::constraint_violation(ErrorKind::CheckViolation, "chk", None, None);
    assert!(err.is_check_violation());
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(128))]

    #[test]
    fn table_not_found_message_roundtrip(name in ".*") {
        let err = DbError::table_not_found(name.clone());
        prop_assert_eq!(err.message(), name.as_str());
        prop_assert_eq!(format!("{err}"), format!("Table not found: {name}"));
    }

    #[test]
    fn column_not_found_message_roundtrip(name in ".*") {
        let err = DbError::column_not_found(name.clone());
        prop_assert_eq!(err.message(), name.as_str());
        prop_assert_eq!(format!("{err}"), format!("Column not found: {name}"));
    }

    #[test]
    fn type_mismatch_message_roundtrip(msg in ".*") {
        let err = DbError::type_mismatch(msg.clone());
        prop_assert_eq!(err.message(), msg.as_str());
        prop_assert_eq!(format!("{err}"), format!("Type mismatch: {msg}"));
    }

    #[test]
    fn invalid_operation_message_roundtrip(msg in ".*") {
        let err = DbError::invalid_operation(msg.clone());
        prop_assert_eq!(err.message(), msg.as_str());
        prop_assert_eq!(format!("{err}"), format!("Invalid operation: {msg}"));
    }

    #[test]
    fn invalid_query_message_roundtrip(msg in ".*") {
        let err = DbError::invalid_query(msg.clone());
        prop_assert_eq!(err.message(), msg.as_str());
        prop_assert_eq!(format!("{err}"), format!("Invalid query: {msg}"));
    }

    #[test]
    fn unsupported_message_roundtrip(msg in ".*") {
        let err = DbError::unsupported(msg.clone());
        prop_assert_eq!(err.message(), msg.as_str());
        prop_assert_eq!(format!("{err}"), format!("Unsupported: {msg}"));
    }
}
