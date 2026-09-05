use core::error::Error as _;
use monumentum_handler::error::{DbError, ErrorKind, MonumentumError};
use proptest as _;
use std::io;

#[test]
fn test_constructors() {
    assert_eq!(
        DbError::table_not_found("users"),
        DbError::TableNotFound("users".into())
    );
    assert_eq!(
        DbError::column_not_found("age"),
        DbError::ColumnNotFound("age".into())
    );
    assert_eq!(
        DbError::type_mismatch("bad type"),
        DbError::TypeMismatch("bad type".into())
    );
    assert_eq!(
        DbError::invalid_operation("bad op"),
        DbError::InvalidOperation("bad op".into())
    );
    assert_eq!(
        DbError::invalid_query("bad query"),
        DbError::InvalidQuery("bad query".into())
    );
    assert_eq!(
        DbError::unsupported("unsupported"),
        DbError::Unsupported("unsupported".into())
    );

    let io_err = io::Error::other("io error");
    let db_io = DbError::from_io(io_err);
    assert!(matches!(db_io, DbError::Io(_)));

    let custom = io::Error::other("custom");
    let corr = DbError::corruption(custom);
    assert!(matches!(corr, DbError::Corruption(_)));

    let trans = DbError::transaction(io::Error::other("tx"));
    assert!(matches!(trans, DbError::Transaction(_)));

    let cons = DbError::constraint_violation(
        ErrorKind::UniqueViolation,
        "duplicate",
        Some("uq".to_string()),
        Some("table".to_string()),
    );
    if let DbError::ConstraintViolation {
        kind,
        message,
        constraint,
        table,
    } = cons
    {
        assert_eq!(kind, ErrorKind::UniqueViolation);
        assert_eq!(message, "duplicate");
        assert_eq!(constraint.as_deref(), Some("uq"));
        assert_eq!(table.as_deref(), Some("table"));
    } else {
        unreachable!("expected ConstraintViolation");
    }
}

#[test]
fn test_display() {
    let e = DbError::table_not_found("users");
    assert_eq!(e.to_string(), "Table not found: users");
    let e = DbError::type_mismatch("wrong type");
    assert_eq!(e.to_string(), "Type mismatch: wrong type");
    let e = DbError::invalid_operation("bad op");
    assert_eq!(e.to_string(), "Invalid operation: bad op");
    let e = DbError::invalid_query("bad query");
    assert_eq!(e.to_string(), "Invalid query: bad query");
    let e = DbError::unsupported("unsupported");
    assert_eq!(e.to_string(), "Unsupported: unsupported");
    let e = DbError::constraint_violation(ErrorKind::CheckViolation, "check failed", None, None);
    assert_eq!(e.to_string(), "Constraint violation: check failed");
}

#[test]
fn test_source() {
    let io_err = io::Error::other("io");
    let e = DbError::from_io(io_err);
    assert!(e.source().is_some());

    let custom = io::Error::other("custom");
    let e = DbError::corruption(custom);
    assert!(e.source().is_some());

    let e = DbError::table_not_found("users");
    assert!(e.source().is_none());
}

#[test]
fn test_partial_eq() {
    assert_eq!(DbError::table_not_found("a"), DbError::table_not_found("a"));
    assert_ne!(DbError::table_not_found("a"), DbError::table_not_found("b"));
    assert_eq!(
        DbError::constraint_violation(
            ErrorKind::UniqueViolation,
            "m",
            Some("c".to_string()),
            Some("t".to_string())
        ),
        DbError::constraint_violation(
            ErrorKind::UniqueViolation,
            "m",
            Some("c".to_string()),
            Some("t".to_string())
        )
    );
    assert_ne!(
        DbError::constraint_violation(
            ErrorKind::UniqueViolation,
            "m",
            Some("c".to_string()),
            Some("t".to_string())
        ),
        DbError::constraint_violation(
            ErrorKind::CheckViolation,
            "m",
            Some("c".to_string()),
            Some("t".to_string())
        )
    );
    let io1 = DbError::from_io(io::Error::other("e1"));
    let io2 = DbError::from_io(io::Error::other("e1"));
    assert_eq!(io1, io2);
}

#[test]
fn test_from_io_error() {
    let io_err = io::Error::other("io");
    let db_err: DbError = io_err.into();
    assert!(matches!(db_err, DbError::Io(_)));
}

#[test]
fn test_monumentum_error_trait() {
    let e = DbError::table_not_found("users");
    assert_eq!(e.kind(), ErrorKind::InvalidOperation);
    assert_eq!(e.message(), "users");
    assert_eq!(e.table(), Some("users"));

    let e = DbError::constraint_violation(
        ErrorKind::UniqueViolation,
        "dup",
        Some("uq".to_string()),
        Some("users".to_string()),
    );
    assert_eq!(e.kind(), ErrorKind::UniqueViolation);
    assert_eq!(e.message(), "dup");
    assert_eq!(e.constraint(), Some("uq"));
    assert_eq!(e.table(), Some("users"));
    assert!(e.is_unique_violation());
    assert!(!e.is_foreign_key_violation());

    let e = DbError::constraint_violation(ErrorKind::ForeignKeyViolation, "fk", None, None);
    assert!(e.is_foreign_key_violation());

    let e = DbError::constraint_violation(ErrorKind::NotNullViolation, "nn", None, None);
    assert!(e.is_not_null_violation());

    let e = DbError::constraint_violation(ErrorKind::CheckViolation, "chk", None, None);
    assert!(e.is_check_violation());

    let e = DbError::type_mismatch("bad");
    assert!(e.is_type_mismatch());

    let e = DbError::invalid_operation("op");
    assert_eq!(e.kind(), ErrorKind::InvalidOperation);
}
