use monumentum_handler::constants::{MAX_BLOB_SIZE, MAX_TEXT_SIZE};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::types::{Blob, Text};
use proptest as _;

fn create_test_schema() -> TableSchema {
    let result = TableSchema::try_new(
        "test_table",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    result.unwrap_or_else(|_| unreachable!())
}

fn make_text(s: &str) -> Text {
    Text::try_new(s.to_string()).unwrap_or_else(|_| unreachable!())
}

fn make_blob(data: Vec<u8>) -> Blob {
    Blob::try_new(data).unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_row_new_and_values() {
    let row = Row::new(vec![Value::from(1i64), Value::from(make_text("hello"))]);
    assert_eq!(row.len(), 2);
    assert!(!row.is_empty());
    assert_eq!(row.values().len(), 2);
}

#[test]
fn test_row_get_valid_index() {
    let row = Row::new(vec![
        Value::from(42i64),
        Value::try_from(3.5).unwrap_or(Value::Null),
    ]);
    if let Some(v) = row.get(0) {
        assert_eq!(*v, Value::from(42i64));
    } else {
        unreachable!("expected Some");
    }
    if let Some(v) = row.get(1) {
        assert!(v.is_float());
        if let Some(f) = v.as_f64() {
            assert!((f - 3.5).abs() < f64::EPSILON);
        } else {
            unreachable!("expected float");
        }
    } else {
        unreachable!("expected Some");
    }
}

#[test]
fn test_row_get_out_of_bounds() {
    let row = Row::new(vec![Value::from(1i64)]);
    assert!(row.get(1).is_none());
}

#[test]
fn test_row_get_by_name_valid() {
    let schema = create_test_schema();
    let row = Row::new(vec![Value::from(1i64), Value::from(make_text("Alice"))]);
    let id = row.get_by_name(&schema, "id");
    assert!(id.is_some());
    if let Some(v) = id {
        assert_eq!(*v, Value::from(1i64));
    }
    let name = row.get_by_name(&schema, "NAME");
    assert!(name.is_some());
}

#[test]
fn test_row_get_by_name_invalid() {
    let schema = create_test_schema();
    let row = Row::new(vec![Value::from(1i64), Value::Null]);
    assert!(row.get_by_name(&schema, "missing").is_none());
}

#[test]
fn test_row_len_and_is_empty() {
    let empty = Row::new(Vec::new());
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

#[test]
fn test_value_null() {
    let v = Value::Null;
    assert!(v.is_null());
    assert_eq!(v.type_name(), "null");
    assert!(v.as_integer().is_none());
    assert!(v.as_float().is_none());
    assert!(v.as_text().is_none());
    assert!(v.as_blob().is_none());
    assert!(v.as_boolean().is_none());
    assert!(v.as_i64().is_none());
    assert!(v.as_f64().is_none());
    assert!(v.as_bool().is_none());
    assert!(v.as_str().is_none());
    assert_eq!(v.to_string(), "NULL");
}

#[test]
fn test_value_integer() {
    let v = Value::from(42i64);
    assert!(v.is_integer());
    assert_eq!(v.type_name(), "integer");
    assert_eq!(v.as_i64(), Some(42));
    if let Some(i) = v.as_integer() {
        assert_eq!(i.as_i64(), 42);
    } else {
        unreachable!("expected integer");
    }
    assert_eq!(v.to_string(), "42");
}

#[test]
fn test_value_float() {
    let result = Value::try_from(3.5);
    assert!(result.is_ok());
    if let Ok(v) = result {
        assert!(v.is_float());
        assert_eq!(v.type_name(), "float");
        if let Some(f) = v.as_f64() {
            assert!((f - 3.5).abs() < f64::EPSILON);
        } else {
            unreachable!("expected float");
        }
        if let Some(f) = v.as_float() {
            assert!((f.as_f64() - 3.5).abs() < f64::EPSILON);
        } else {
            unreachable!("expected float");
        }
        assert!(v.to_string().contains("3.5"));
    }
}

#[test]
fn test_value_text() {
    let text = make_text("O'Reilly");
    let v = Value::from(text);
    assert!(v.is_text());
    assert_eq!(v.type_name(), "text");
    assert_eq!(v.as_str(), Some("O'Reilly"));
    if let Some(t) = v.as_text() {
        assert_eq!(t.as_str(), "O'Reilly");
    } else {
        unreachable!("expected text");
    }
    assert_eq!(v.to_string(), "'O''Reilly'");
}

#[test]
fn test_value_blob() {
    let blob = make_blob(vec![1, 2, 3]);
    let v = Value::from(blob);
    assert!(v.is_blob());
    assert_eq!(v.type_name(), "blob");
    if let Some(b) = v.as_blob() {
        assert_eq!(b.as_slice(), &[1, 2, 3][..]);
    } else {
        unreachable!("expected blob");
    }
    assert_eq!(v.to_string(), "Blob(3 bytes)");
}

#[test]
fn test_value_boolean() {
    let v = Value::from(true);
    assert!(v.is_boolean());
    assert_eq!(v.type_name(), "boolean");
    assert_eq!(v.as_boolean(), Some(true));
    assert_eq!(v.as_bool(), Some(true));
    assert_eq!(v.to_string(), "true");
}

#[test]
fn test_value_type_mismatch_accessors() {
    let v = Value::from(42i64);
    assert!(v.as_float().is_none());
    assert!(v.as_text().is_none());
    assert!(v.as_blob().is_none());
    assert!(v.as_boolean().is_none());
    assert!(v.as_str().is_none());
}

#[test]
fn test_value_into_conversions() {
    let v = Value::from(42i64);
    if let Some(i) = v.into_integer() {
        assert_eq!(i.as_i64(), 42);
    } else {
        unreachable!("expected integer");
    }

    let result = Value::try_from(2.5);
    assert!(result.is_ok());
    if let Ok(v) = result {
        if let Some(f) = v.into_float() {
            assert!((f.as_f64() - 2.5).abs() < f64::EPSILON);
        } else {
            unreachable!("expected float");
        }
    }

    let v = Value::from(make_text("hi"));
    if let Some(t) = v.into_text() {
        assert_eq!(t.as_str(), "hi");
    } else {
        unreachable!("expected text");
    }

    let v = Value::from(make_blob(vec![7]));
    if let Some(b) = v.into_blob() {
        assert_eq!(b.as_slice(), &[7]);
    } else {
        unreachable!("expected blob");
    }

    let v = Value::from(false);
    assert_eq!(v.into_boolean(), Some(false));
}

#[test]
fn test_value_from_and_tryfrom() {
    let v = Value::from(());
    assert_eq!(v, Value::Null);

    let int: monumentum_handler::types::Integer = 10.into();
    let v = Value::from(int);
    assert!(matches!(v, Value::Integer(_)));

    let float_result = monumentum_handler::types::Float::try_new(1.5);
    assert!(float_result.is_ok());
    if let Ok(f) = float_result {
        let v = Value::from(f);
        assert!(matches!(v, Value::Float(_)));
    }

    let text = make_text("hello");
    let v = Value::from(text);
    assert!(matches!(v, Value::Text(_)));

    let blob = make_blob(vec![1]);
    let v = Value::from(blob);
    assert!(matches!(v, Value::Blob(_)));

    let v = Value::from(true);
    assert!(matches!(v, Value::Boolean(true)));

    let v = Value::from(5i64);
    assert!(matches!(v, Value::Integer(_)));
}

#[test]
fn test_value_tryfrom_f64_invalid() {
    let result = Value::try_from(f64::NAN);
    assert!(result.is_err());
    let result = Value::try_from(f64::INFINITY);
    assert!(result.is_err());
}

#[test]
fn test_value_tryfrom_string_and_str() {
    let s = "a".repeat(MAX_TEXT_SIZE + 1);
    assert!(Value::try_from(s.clone()).is_err());
    assert!(Value::try_from(s.as_str()).is_err());

    let ok_s = "valid";
    assert!(Value::try_from(ok_s.to_string()).is_ok());
    assert!(Value::try_from(ok_s).is_ok());
}

#[test]
fn test_value_tryfrom_bytes() {
    let big = vec![0u8; MAX_BLOB_SIZE + 1];
    assert!(Value::try_from(big.clone()).is_err());
    assert!(Value::try_from(big.as_slice()).is_err());

    let small = vec![1, 2, 3];
    assert!(Value::try_from(small.clone()).is_ok());
    assert!(Value::try_from(small.as_slice()).is_ok());
}

#[test]
fn test_value_partial_eq_and_ord() {
    let v1 = Value::from(1i64);
    let v2 = Value::from(1i64);
    let v3 = Value::from(2i64);
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
    assert!(v1 < v3);
}
