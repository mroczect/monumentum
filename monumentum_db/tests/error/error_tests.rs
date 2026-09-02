use core::error::Error;
use monumentum_db::error::DbError;
use std::io;

fn assert_std_error<T: Error>() {}

#[test]
fn db_error_is_std_error() {
    assert_std_error::<DbError>();
}

#[test]
fn table_not_found_constructor_preserves_name() {
    let err = DbError::table_not_found("users");
    match err {
        DbError::TableNotFound(name) => assert_eq!(name, "users"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn column_not_found_constructor_preserves_name() {
    let err = DbError::column_not_found("id");
    match err {
        DbError::ColumnNotFound(name) => assert_eq!(name, "id"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn type_mismatch_constructor_preserves_message() {
    let err = DbError::type_mismatch("expected INTEGER");
    match err {
        DbError::TypeMismatch(msg) => assert_eq!(msg, "expected INTEGER"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn invalid_operation_constructor_preserves_message() {
    let err = DbError::invalid_operation("table name empty");
    match err {
        DbError::InvalidOperation(msg) => assert_eq!(msg, "table name empty"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn invalid_query_constructor_preserves_message() {
    let err = DbError::invalid_query("syntax error near WHERE");
    match err {
        DbError::InvalidQuery(msg) => assert_eq!(msg, "syntax error near WHERE"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn unsupported_constructor_preserves_message() {
    let err = DbError::unsupported("JOIN not implemented");
    match err {
        DbError::Unsupported(msg) => assert_eq!(msg, "JOIN not implemented"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn corruption_constructor_wraps_source() {
    let inner = io::Error::new(io::ErrorKind::InvalidData, "bad checksum");
    let err = DbError::corruption(inner);
    assert!(matches!(err, DbError::Corruption(_)));
    assert!(err.source().is_some());
}

#[test]
fn transaction_constructor_wraps_source() {
    let inner = io::Error::other("deadlock detected");
    let err = DbError::transaction(inner);
    assert!(matches!(err, DbError::Transaction(_)));
    assert!(err.source().is_some());
}

#[test]
fn display_table_not_found() {
    let err = DbError::table_not_found("users");
    assert_eq!(format!("{err}"), "Table not found: users");
}

#[test]
fn display_column_not_found() {
    let err = DbError::column_not_found("id");
    assert_eq!(format!("{err}"), "Column not found: id");
}

#[test]
fn display_type_mismatch() {
    let err = DbError::type_mismatch("expected Text, got Integer");
    assert_eq!(
        format!("{err}"),
        "Type mismatch: expected Text, got Integer"
    );
}

#[test]
fn display_invalid_operation() {
    let err = DbError::invalid_operation("cannot drop non-empty table");
    assert_eq!(
        format!("{err}"),
        "Invalid operation: cannot drop non-empty table"
    );
}

#[test]
fn display_invalid_query() {
    let err = DbError::invalid_query("unexpected token");
    assert_eq!(format!("{err}"), "Invalid query: unexpected token");
}

#[test]
fn display_unsupported() {
    let err = DbError::unsupported("feature not available");
    assert_eq!(format!("{err}"), "Unsupported: feature not available");
}

#[test]
fn display_io_wraps_io_message() {
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
fn source_is_some_for_io() {
    let io_err = io::Error::other("io error");
    let db_err = DbError::from(io_err);
    assert!(db_err.source().is_some());
}

#[test]
fn source_is_some_for_corruption() {
    let inner = io::Error::new(io::ErrorKind::InvalidData, "corrupt");
    let err = DbError::corruption(inner);
    assert!(err.source().is_some());
}

#[test]
fn source_is_some_for_transaction() {
    let inner = io::Error::other("txn");
    let err = DbError::transaction(inner);
    assert!(err.source().is_some());
}

#[test]
fn source_is_none_for_string_variants() {
    let variants = [
        DbError::table_not_found("t"),
        DbError::column_not_found("c"),
        DbError::type_mismatch("tm"),
        DbError::invalid_operation("io"),
        DbError::invalid_query("iq"),
        DbError::unsupported("u"),
    ];
    for err in variants {
        assert!(err.source().is_none(), "source should be None for {err:?}");
    }
}

#[test]
fn from_io_error_preserves_kind_and_message() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let db_err = DbError::from(io_err);
    match db_err {
        DbError::Io(inner) => {
            assert_eq!(inner.kind(), io::ErrorKind::NotFound);
            assert_eq!(inner.to_string(), "file missing");
        }
        _ => panic!("expected Io variant"),
    }
}

#[test]
fn db_error_can_be_used_as_boxed_error() {
    let err = DbError::invalid_query("bad");
    let boxed: Box<dyn Error> = Box::new(err);
    assert!(boxed.to_string().contains("bad"));
}

#[test]
fn db_error_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DbError>();
}

#[test]
fn db_error_can_move_to_other_thread() {
    let err = DbError::invalid_operation("thread error");
    let handle = std::thread::spawn(move || {
        assert_eq!(format!("{err}"), "Invalid operation: thread error");
    });
    handle.join().unwrap();
}

#[test]
fn propagated_io_error_preserves_source() {
    fn fallible_operation() -> Result<(), DbError> {
        let io_err = io::Error::other("disk full");
        Err(DbError::from(io_err))
    }

    let result = fallible_operation();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, DbError::Io(_)));
    assert!(err.source().is_some());
}

#[test]
fn empty_string_payload_display() {
    let err = DbError::table_not_found("");
    assert_eq!(format!("{err}"), "Table not found: ");
}

#[test]
fn unicode_payload_display() {
    let err = DbError::invalid_query("error: 😀");
    assert_eq!(format!("{err}"), "Invalid query: error: 😀");
}

#[test]
fn newline_payload_display() {
    let err = DbError::invalid_operation("line1\nline2");
    assert_eq!(format!("{err}"), "Invalid operation: line1\nline2");
}

#[test]
fn format_string_payload_does_not_inject() {
    let err = DbError::invalid_operation("{}");
    assert_eq!(format!("{err}"), "Invalid operation: {}");
}

#[test]
fn very_long_payload_display() {
    let long_msg = "a".repeat(100_000);
    let err = DbError::type_mismatch(long_msg.clone());
    assert!(format!("{err}").ends_with(&long_msg));
}
