use monumentum_db::error::DbError;
use std::error::Error;
use std::io;

fn io_error(msg: &str) -> io::Error {
    io::Error::other(msg)
}

fn assert_send_sync<T: Send + Sync>() {}

#[derive(Debug)]
struct CustomError(&'static str);

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CustomError {}

#[test]
fn io_variant_display_and_source() {
    let io_err = io_error("disk full");
    let db_err = DbError::Io(io_err);
    let display = format!("{db_err}");
    assert!(display.contains("I/O error: disk full"));
    let source = db_err.source().expect("source should exist");
    assert_eq!(source.to_string(), "disk full");
}

#[test]
fn corruption_variant_with_source() {
    let custom = CustomError("bad checksum");
    let db_err = DbError::corruption(custom);
    let display = format!("{db_err}");
    assert!(display.contains("Data corruption: bad checksum"));
    let source = db_err.source().expect("source should exist");
    assert_eq!(source.to_string(), "bad checksum");
}

#[test]
fn corruption_variant_without_source() {
    let db_err = DbError::corruption(CustomError("x"));
    assert!(db_err.source().is_some());
}

#[test]
fn table_not_found_display_and_source() {
    let db_err = DbError::table_not_found("users");
    let display = format!("{db_err}");
    assert_eq!(display, "Table not found: users");
    assert!(db_err.source().is_none());
}

#[test]
fn column_not_found_display_and_source() {
    let db_err = DbError::column_not_found("email");
    let display = format!("{db_err}");
    assert_eq!(display, "Column not found: email");
    assert!(db_err.source().is_none());
}

#[test]
fn type_mismatch_display_and_source() {
    let db_err = DbError::type_mismatch("expected Integer, got Text");
    let display = format!("{db_err}");
    assert_eq!(display, "Type mismatch: expected Integer, got Text");
    assert!(db_err.source().is_none());
}

#[test]
fn invalid_operation_display_and_source() {
    let db_err = DbError::invalid_operation("cannot drop non-empty table");
    let display = format!("{db_err}");
    assert_eq!(display, "Invalid operation: cannot drop non-empty table");
    assert!(db_err.source().is_none());
}

#[test]
fn invalid_query_display_and_source() {
    let db_err = DbError::invalid_query("unexpected token");
    let display = format!("{db_err}");
    assert_eq!(display, "Invalid query: unexpected token");
    assert!(db_err.source().is_none());
}

#[test]
fn transaction_variant_with_source() {
    let custom = CustomError("deadlock detected");
    let db_err = DbError::transaction(custom);
    let display = format!("{db_err}");
    assert!(display.contains("Transaction error: deadlock detected"));
    let source = db_err.source().expect("source should exist");
    assert_eq!(source.to_string(), "deadlock detected");
}

#[test]
fn unsupported_display_and_source() {
    let db_err = DbError::unsupported("feature not implemented");
    let display = format!("{db_err}");
    assert_eq!(display, "Unsupported: feature not implemented");
    assert!(db_err.source().is_none());
}

#[test]
fn display_with_special_characters() {
    let long_string = "very long string ".repeat(1000);
    let cases = vec![
        "",
        "   ",
        "line\nbreak",
        "tab\there",
        "quote\"double\"",
        "back\\slash",
        "emoji 😀",
        "nul\0char",
        long_string.as_str(),
    ];

    for payload in cases {
        let err = DbError::invalid_operation(payload.to_string());
        let formatted = format!("{err}");
        assert!(formatted.starts_with("Invalid operation: "));
        assert!(formatted.ends_with(payload));
    }
}

#[test]
fn display_io_error_preserves_os_details() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let db_err = DbError::from(io_err);
    let display = format!("{db_err}");
    assert!(display.contains("access denied"));
}

#[test]
fn source_for_variants_with_source() {
    let io_err = io_error("io fail");
    let db_err = DbError::Io(io_err);
    assert!(db_err.source().is_some());

    let db_err = DbError::corruption(CustomError("corrupt"));
    assert!(db_err.source().is_some());

    let db_err = DbError::transaction(CustomError("txn fail"));
    assert!(db_err.source().is_some());
}

#[test]
fn source_none_for_string_variants() {
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
fn source_returns_same_error() {
    let custom = CustomError("same error");
    let db_err = DbError::corruption(custom);
    let source = db_err.source().unwrap();
    assert_eq!(source.to_string(), "same error");
    let downcasted = source.downcast_ref::<CustomError>();
    assert!(downcasted.is_some());
}

#[test]
fn db_error_implements_std_error() {
    fn assert_std_error<T: Error>() {}
    assert_std_error::<DbError>();

    let boxed: Box<dyn Error> = Box::new(DbError::invalid_query("bad"));
    assert!(boxed.to_string().contains("bad"));
}

#[test]
fn from_io_error_produces_io_variant() {
    let io_err = io_error("source io");
    let db_err: DbError = io_err.into();
    match db_err {
        DbError::Io(e) => assert_eq!(e.to_string(), "source io"),
        _ => panic!("expected Io variant"),
    }
}

#[test]
fn from_io_error_preserves_source() {
    let io_err = io_error("original");
    let db_err = DbError::from(io_err);
    let source = db_err.source().unwrap();
    assert_eq!(source.to_string(), "original");
}

#[test]
fn constructors_accept_impl_into_string() {
    let err = DbError::table_not_found("users");
    assert_eq!(format!("{err}"), "Table not found: users");

    let err = DbError::table_not_found(String::from("orders"));
    assert_eq!(format!("{err}"), "Table not found: orders");

    let name = "products".to_string();
    let err = DbError::table_not_found(&name);
    assert_eq!(format!("{err}"), "Table not found: products");
}

#[test]
fn constructor_does_not_modify_payload() {
    let payload = "  trim me  ";
    let err = DbError::invalid_operation(payload.to_string());
    assert_eq!(format!("{err}"), "Invalid operation:   trim me  ");
}

#[test]
fn constructor_with_source_preserves_error() {
    let custom = CustomError("source preserved");
    let err = DbError::transaction(custom);
    let source = err.source().unwrap();
    assert_eq!(source.to_string(), "source preserved");
    let downcasted = source.downcast_ref::<CustomError>();
    assert!(downcasted.is_some());
}

#[test]
fn db_error_is_send_and_sync() {
    assert_send_sync::<DbError>();
}

#[test]
fn db_error_can_be_moved_between_threads() {
    let err = DbError::invalid_operation("thread error");
    let handle = std::thread::spawn(move || {
        let s = format!("{err}");
        assert_eq!(s, "Invalid operation: thread error");
    });
    handle.join().unwrap();
}

fn function_returning_result() -> Result<(), DbError> {
    let _ = std::fs::read_to_string("/nonexistent/path").map_err(DbError::from)?;
    Ok(())
}

#[test]
fn question_mark_propagates_io_error() {
    let result = function_returning_result();
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        DbError::Io(_) => (),
        _ => panic!("expected Io variant"),
    }
}

#[test]
fn propagated_error_has_source_and_display() {
    let result = function_returning_result();
    let err = result.unwrap_err();
    assert!(err.source().is_some());
    assert!(!err.to_string().is_empty());
}

#[test]
fn empty_string_payload() {
    let err = DbError::table_not_found("");
    assert_eq!(format!("{err}"), "Table not found: ");
    let err = DbError::invalid_query("");
    assert_eq!(format!("{err}"), "Invalid query: ");
}

#[test]
fn very_long_payload() {
    let payload = "a".repeat(1_000_000);
    let err = DbError::type_mismatch(payload.clone());
    let formatted = format!("{err}");
    assert!(formatted.starts_with("Type mismatch: "));
    assert!(formatted.ends_with(&payload));
}

#[test]
fn payload_with_null_char() {
    let payload = "bad\0data";
    let err = DbError::unsupported(payload.to_string());
    let formatted = format!("{err}");
    assert!(formatted.contains('\0'));
}

#[test]
fn nested_error_chain() {
    let inner = DbError::invalid_operation("inner");
    let outer = DbError::corruption(inner);
    let source = outer.source().unwrap();
    let downcasted = source.downcast_ref::<DbError>();
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().to_string(), "Invalid operation: inner");
}

#[test]
fn payload_with_format_string() {
    let payload = "{}";
    let err = DbError::invalid_operation(payload.to_string());
    let formatted = format!("{err}");
    assert_eq!(formatted, "Invalid operation: {}");
}

#[test]
fn all_public_methods_exist() {
    let _ = DbError::table_not_found("x");
    let _ = DbError::column_not_found("x");
    let _ = DbError::type_mismatch("x");
    let _ = DbError::invalid_operation("x");
    let _ = DbError::invalid_query("x");
    let _ = DbError::unsupported("x");
    let _ = DbError::corruption(CustomError("x"));
    let _ = DbError::transaction(CustomError("x"));
}
