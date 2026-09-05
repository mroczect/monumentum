use monumentum_handler::constants::MAX_COLUMNS;
use monumentum_handler::constants::MAX_NAME_LENGTH;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{
    CheckConstraint, ColumnDef, ColumnIndex, ComparisonOp, DataType, ForeignKey,
};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::{ErrorKind, MonumentumError};
use monumentum_handler::types::{Blob, Text};
use proptest as _;

fn int_value(v: i64) -> Value {
    Value::from(v)
}

fn text_value(s: &str) -> Value {
    Text::try_new(s.to_string()).map_or_else(|_| unreachable!(), Value::from)
}

fn float_value(v: f64) -> Value {
    Value::try_from(v).unwrap_or_else(|_| unreachable!())
}

fn blob_value(data: Vec<u8>) -> Value {
    Blob::try_new(data)
        .unwrap_or_else(|_| unreachable!())
        .into()
}

#[test]
fn test_column_def_new_defaults() {
    let col = ColumnDef::new("age", DataType::Integer);
    assert_eq!(col.name(), "age");
    assert_eq!(*col.data_type(), DataType::Integer);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(!col.is_unique());
    assert!(col.default_value().is_none());
    assert!(col.check_constraint().is_none());
    assert!(col.foreign_key().is_none());
    assert!(col.allowed_values().is_none());
}

#[test]
fn test_column_def_setters() {
    let mut col = ColumnDef::new("id", DataType::Integer);

    col.set_primary_key(true);
    assert!(!col.is_nullable());
    col.set_nullable(true);
    assert!(!col.is_nullable());

    col.set_primary_key(false);
    col.set_nullable(true);
    assert!(col.is_nullable());

    col.set_primary_key(true);
    assert!(col.is_primary_key());
    assert!(col.is_unique());
    assert!(!col.is_nullable());

    col.set_primary_key(false);
    col.set_unique(true);
    assert!(col.is_unique());

    let default_val = int_value(10);
    col.set_default(Some(default_val.clone()));
    assert_eq!(col.default_value(), Some(&default_val));

    let check = CheckConstraint {
        column: "id".to_string(),
        op: ComparisonOp::Gte,
        value: int_value(0),
    };
    col.set_check(Some(check.clone()));
    assert_eq!(col.check_constraint(), Some(&check));

    let fk = ForeignKey {
        table: "other".to_string(),
        column: "id".to_string(),
    };
    col.set_foreign_key(Some(fk.clone()));
    assert_eq!(col.foreign_key(), Some(&fk));

    let allowed = vec![int_value(1), int_value(2)];
    col.set_allowed_values(Some(allowed.clone()));
    assert_eq!(col.allowed_values(), Some(&allowed));
}

#[test]
fn test_column_def_validate_nullability() {
    let mut col = ColumnDef::new("name", DataType::Text);
    assert!(col.validate_value(&Value::Null).is_ok());

    col.set_nullable(false);
    let result = col.validate_value(&Value::Null);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::NotNullViolation);
    }
}

#[test]
fn test_column_def_validate_type() {
    let col_int = ColumnDef::new("num", DataType::Integer);
    assert!(col_int.validate_value(&int_value(5)).is_ok());
    let result = col_int.validate_value(&text_value("hello"));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::TypeMismatch);
    }

    let col_float = ColumnDef::new("price", DataType::Float);
    assert!(col_float.validate_value(&float_value(2.5)).is_ok());
    assert!(col_float.validate_value(&int_value(1)).is_err());

    let col_text = ColumnDef::new("name", DataType::Text);
    assert!(col_text.validate_value(&text_value("abc")).is_ok());
    assert!(col_text.validate_value(&int_value(1)).is_err());

    let col_blob = ColumnDef::new("data", DataType::Blob);
    assert!(col_blob.validate_value(&blob_value(vec![1])).is_ok());
    assert!(col_blob.validate_value(&text_value("x")).is_err());

    let col_bool = ColumnDef::new("flag", DataType::Boolean);
    assert!(col_bool.validate_value(&Value::from(true)).is_ok());
    assert!(col_bool.validate_value(&int_value(1)).is_err());
}

#[test]
fn test_column_def_validate_check_constraint() {
    let mut col = ColumnDef::new("val", DataType::Integer);
    col.set_check(Some(CheckConstraint {
        column: "val".to_string(),
        op: ComparisonOp::Gte,
        value: int_value(10),
    }));
    assert!(col.validate_value(&int_value(10)).is_ok());
    assert!(col.validate_value(&int_value(9)).is_err());
    assert!(col.validate_value(&int_value(11)).is_ok());

    let mut col_f = ColumnDef::new("ratio", DataType::Float);
    col_f.set_check(Some(CheckConstraint {
        column: "ratio".to_string(),
        op: ComparisonOp::Gt,
        value: float_value(0.0),
    }));
    assert!(col_f.validate_value(&float_value(0.0001)).is_ok());
    assert!(col_f.validate_value(&float_value(-1.0)).is_err());

    let mut col_t = ColumnDef::new("code", DataType::Text);
    col_t.set_check(Some(CheckConstraint {
        column: "code".to_string(),
        op: ComparisonOp::Eq,
        value: text_value("abc"),
    }));
    assert!(col_t.validate_value(&text_value("abc")).is_ok());
    assert!(col_t.validate_value(&text_value("def")).is_err());

    let mut col_b = ColumnDef::new("bin", DataType::Blob);
    col_b.set_check(Some(CheckConstraint {
        column: "bin".to_string(),
        op: ComparisonOp::Eq,
        value: blob_value(vec![1, 2]),
    }));
    assert!(col_b.validate_value(&blob_value(vec![1, 2])).is_ok());
    assert!(col_b.validate_value(&blob_value(vec![1, 3])).is_err());
    col_b.set_check(Some(CheckConstraint {
        column: "bin".to_string(),
        op: ComparisonOp::Lt,
        value: blob_value(vec![9]),
    }));
    assert!(col_b.validate_value(&blob_value(vec![1])).is_err());

    let mut col_bool = ColumnDef::new("flag", DataType::Boolean);
    col_bool.set_check(Some(CheckConstraint {
        column: "flag".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from(true),
    }));
    assert!(col_bool.validate_value(&Value::from(true)).is_ok());
    assert!(col_bool.validate_value(&Value::from(false)).is_err());
    col_bool.set_check(Some(CheckConstraint {
        column: "flag".to_string(),
        op: ComparisonOp::Lt,
        value: Value::from(true),
    }));
    assert!(col_bool.validate_value(&Value::from(true)).is_err());
}

#[test]
fn test_column_def_validate_allowed_values() {
    let mut col = ColumnDef::new("status", DataType::Integer);
    col.set_allowed_values(Some(vec![int_value(1), int_value(2), int_value(3)]));
    assert!(col.validate_value(&int_value(2)).is_ok());
    assert!(col.validate_value(&int_value(4)).is_err());
}

#[test]
fn test_column_index_for_row_usize() {
    let row = Row::new(vec![int_value(1), int_value(2)]);
    let result = 0usize.index(&row);
    assert!(result.is_ok());
    if let Ok(idx) = result {
        assert_eq!(idx, 0);
    }
    let result = 1usize.index(&row);
    assert!(result.is_ok());
    if let Ok(idx) = result {
        assert_eq!(idx, 1);
    }
    assert!(2usize.index(&row).is_err());
}

#[test]
fn test_column_index_for_schema_usize() {
    let result = TableSchema::try_new("t", vec![ColumnDef::new("a", DataType::Integer)]);
    assert!(result.is_ok());
    if let Ok(schema) = result {
        let idx_result = 0usize.index(&schema);
        assert!(idx_result.is_ok());
        if let Ok(idx) = idx_result {
            assert_eq!(idx, 0);
        }
        assert!(1usize.index(&schema).is_err());
    }
}

#[test]
fn test_column_index_for_schema_str() {
    let result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    if let Ok(schema) = result {
        let idx = "id".index(&schema);
        assert!(idx.is_ok());
        if let Ok(i) = idx {
            assert_eq!(i, 0);
        }
        let idx = "NAME".index(&schema);
        assert!(idx.is_ok());
        if let Ok(i) = idx {
            assert_eq!(i, 1);
        }
        assert!("missing".index(&schema).is_err());
    }
}

#[test]
fn test_table_schema_valid_creation() {
    let result = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    if let Ok(schema) = result {
        assert_eq!(schema.name(), "users");
        assert_eq!(schema.columns().len(), 2);
    }
}

#[test]
fn test_table_schema_invalid_name_empty() {
    let result = TableSchema::try_new("", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidOperation);
    }
}

#[test]
fn test_table_schema_name_too_long() {
    let long = "a".repeat(MAX_NAME_LENGTH + 1);
    let result = TableSchema::try_new(long, vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_err());
}

#[test]
fn test_table_schema_name_control_chars() {
    let result = TableSchema::try_new("bad\nname", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_err());
}

#[test]
fn test_table_schema_no_columns() {
    let result = TableSchema::try_new("empty", Vec::new());
    assert!(result.is_err());
}

#[test]
fn test_table_schema_too_many_columns() {
    let mut cols = Vec::new();
    for i in 0..=MAX_COLUMNS {
        cols.push(ColumnDef::new(format!("col{}", i), DataType::Integer));
    }
    let result = TableSchema::try_new("many", cols);
    assert!(result.is_err());
}

#[test]
fn test_table_schema_duplicate_column_names() {
    let result = TableSchema::try_new(
        "dup",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("ID", DataType::Text),
        ],
    );
    assert!(result.is_err());
}

#[test]
fn test_table_schema_column_index() {
    let result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    if let Ok(schema) = result {
        assert_eq!(schema.column_index("id"), Some(0));
        assert_eq!(schema.column_index("NAME"), Some(1));
        assert_eq!(schema.column_index("missing"), None);
    }
}

#[test]
fn test_table_schema_validate_values() {
    let result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    if let Ok(schema) = result {
        assert!(
            schema
                .validate_values(&[int_value(1), text_value("Alice")])
                .is_ok()
        );

        let err_result = schema.validate_values(&[int_value(1)]);
        assert!(err_result.is_err());
        if let Err(e) = err_result {
            assert_eq!(e.kind(), ErrorKind::InvalidOperation);
        }

        let err_result = schema.validate_values(&[text_value("x"), text_value("y")]);
        assert!(err_result.is_err());
        if let Err(e) = err_result {
            assert_eq!(e.kind(), ErrorKind::TypeMismatch);
        }
    }
}

#[test]
fn test_table_schema_get_column_by_index() {
    let result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    if let Ok(schema) = result {
        assert!(schema.get_column_by_index(0).is_some());
        assert!(schema.get_column_by_index(1).is_some());
        assert!(schema.get_column_by_index(2).is_none());
    }
}
