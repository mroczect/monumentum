#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

#[cfg(test)]
mod tests {
    use core::error::Error as _;

    use monumentum_handler::constants::*;
    use monumentum_handler::core::row::Row;
    use monumentum_handler::core::schema::column::{
        CheckConstraint, ColumnDef, ColumnIndex, ComparisonOp, DataType, ForeignKey,
    };
    use monumentum_handler::core::schema::table_schema::TableSchema;
    use monumentum_handler::core::value::Value;
    use monumentum_handler::error::{DbError, ErrorKind, MonumentumError};
    use monumentum_handler::traits::{CatalogStore, Index, StorageEngine, TableStore};
    use monumentum_handler::types::{Blob, Float, Integer, Text};
    use monumentum_handler::validation::{
        validate_column_name, validate_name, validate_table_name,
    };

    fn unwrap_result<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(v) => v,
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    fn unwrap_option<T>(option: Option<T>) -> T {
        option.unwrap_or_else(|| panic!("unexpected None"))
    }
    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < f64::EPSILON, "floats not equal: {a} vs {b}");
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(HASH_LENGTH, 64);
        assert_eq!(MAX_NAME_LENGTH, 255);
        assert_eq!(MAX_COLUMNS, 1024);
        assert_eq!(MAX_TEXT_SIZE, 16 * 1024 * 1024);
        assert_eq!(MAX_BLOB_SIZE, 64 * 1024 * 1024);
        assert_eq!(MAX_ROWS_PER_TABLE, 10_000_000);
        assert_eq!(MAX_TABLES, 1024);
        assert_eq!(MAX_RECORD_SIZE, 64 * 1024 * 1024);
        assert_eq!(MAX_SNAPSHOT_SIZE, 256 * 1024 * 1024);
        assert_eq!(MAX_VEC_ELEMENTS, 1_000_000);
    }

    #[test]
    fn text_new_and_accessors() {
        let text = unwrap_result(Text::try_new("hello".to_string()));
        assert_eq!(text.as_str(), "hello");
        assert_eq!(text.len(), 5);
        assert!(!text.is_empty());
        assert_eq!(text.to_uppercase().as_str(), "HELLO");
        assert_eq!(text.to_lowercase().as_str(), "hello");
        assert!(text.contains_ignore_case("ELL"));
        assert_eq!(text.as_bytes(), b"hello");
    }

    #[test]
    fn text_try_new_accepts_boundary_size() {
        let max = MAX_TEXT_SIZE;
        let s = "x".repeat(max);
        assert!(Text::try_new(s).is_ok());
    }

    #[test]
    fn text_try_new_rejects_over_max() {
        let s = "x".repeat(MAX_TEXT_SIZE + 1);
        let result = Text::try_new(s);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => {
                assert!(msg.contains("text size"));
                assert!(msg.contains("exceeds maximum"));
            }
            _ => panic!("wrong error type"),
        }
    }

    #[test]
    fn text_display() {
        let text = unwrap_result(Text::try_new("abc".to_string()));
        assert_eq!(format!("{text}"), "abc");
    }

    #[test]
    fn text_try_from_string_and_str() {
        let from_string = unwrap_result(Text::try_from("hello".to_string()));
        let from_str = unwrap_result(Text::try_from("hello"));
        assert_eq!(from_string, from_str);
        assert_eq!(from_string.as_str(), "hello");
    }

    #[test]
    fn text_try_from_invalid_size() {
        let s1 = "x".repeat(MAX_TEXT_SIZE + 1);
        let s2 = s1.clone();
        assert!(Text::try_from(s1).is_err());
        assert!(Text::try_from(s2.as_str()).is_err());
    }

    #[test]
    fn text_as_ref_str() {
        let text = unwrap_result(Text::try_new("ref".to_string()));
        let s: &str = text.as_ref();
        assert_eq!(s, "ref");
    }

    #[test]
    fn text_empty() {
        let empty = unwrap_result(Text::try_new(String::new()));
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_str(), "");
        assert_eq!(empty.as_bytes(), b"");
        assert_eq!(empty.to_uppercase().as_str(), "");
        assert_eq!(empty.to_lowercase().as_str(), "");
        assert!(!empty.contains_ignore_case("x"));
    }

    #[test]
    fn text_case_insensitive_contains() {
        let text = unwrap_result(Text::try_new("HelloWorld".to_string()));
        assert!(text.contains_ignore_case("helloworld"));
        assert!(text.contains_ignore_case("HELLO"));
        assert!(text.contains_ignore_case("world"));
        assert!(!text.contains_ignore_case("xyz"));
    }

    #[test]
    fn blob_new_and_accessors() {
        let data = vec![1, 2, 3, 4];
        let blob = unwrap_result(Blob::try_new(data.clone()));
        assert_eq!(blob.as_slice(), &data[..]);
        assert_eq!(blob.len(), 4);
        assert!(!blob.is_empty());
        assert_eq!(blob.as_ref(), &data[..]);
    }

    #[test]
    fn blob_try_new_accepts_boundary_size() {
        let data = vec![0u8; MAX_BLOB_SIZE];
        assert!(Blob::try_new(data).is_ok());
    }

    #[test]
    fn blob_try_new_rejects_over_max() {
        let data = vec![0u8; MAX_BLOB_SIZE + 1];
        let result = Blob::try_new(data);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => {
                assert!(msg.contains("blob size"));
                assert!(msg.contains("exceeds maximum"));
            }
            _ => panic!("wrong error type"),
        }
    }

    #[test]
    fn blob_display() {
        let blob = unwrap_result(Blob::try_new(vec![1, 2, 3]));
        assert_eq!(format!("{blob}"), "Blob(3 bytes)");
    }

    #[test]
    fn blob_try_from_vec_and_slice() {
        let v = vec![5, 6, 7];
        let from_vec = unwrap_result(Blob::try_from(v.clone()));
        let from_slice = unwrap_result(Blob::try_from(&v[..]));
        assert_eq!(from_vec, from_slice);
        assert_eq!(from_vec.as_slice(), &v[..]);
    }

    #[test]
    fn blob_try_from_invalid_size() {
        let v = vec![0u8; MAX_BLOB_SIZE + 1];
        assert!(Blob::try_from(v).is_err());
    }

    #[test]
    fn blob_empty() {
        let blob = unwrap_result(Blob::try_new(Vec::new()));
        assert!(blob.is_empty());
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.as_slice(), &[]);
        assert_eq!(format!("{blob}"), "Blob(0 bytes)");
    }

    #[test]
    fn float_try_new_rejects_non_finite() {
        assert!(Float::try_new(f64::NAN).is_err());
        assert!(Float::try_new(f64::INFINITY).is_err());
        assert!(Float::try_new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn float_try_new_accepts_finite() {
        assert!(Float::try_new(0.0).is_ok());
        assert!(Float::try_new(-1.5).is_ok());
        assert!(Float::try_new(f64::MAX).is_ok());
        assert!(Float::try_new(f64::MIN).is_ok());
        assert!(Float::try_new(f64::EPSILON).is_ok());
    }

    #[test]
    fn float_as_f64() {
        let f = unwrap_result(Float::try_new(2.5));
        assert_float_eq(f.as_f64(), 2.5);
    }

    #[test]
    fn float_total_cmp() {
        let a = unwrap_result(Float::try_new(1.0));
        let b = unwrap_result(Float::try_new(2.0));
        let c = unwrap_result(Float::try_new(1.0));
        assert_eq!(a.total_cmp(&b), core::cmp::Ordering::Less);
        assert_eq!(b.total_cmp(&a), core::cmp::Ordering::Greater);
        assert_eq!(a.total_cmp(&c), core::cmp::Ordering::Equal);
    }

    #[test]
    fn float_bytes_roundtrip() {
        let original = unwrap_result(Float::try_new(-123.456));
        let bytes = original.to_le_bytes();
        let reconstructed = unwrap_result(Float::try_from_le_bytes(bytes));
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn float_display() {
        let f = unwrap_result(Float::try_new(2.5));
        assert_eq!(format!("{f}"), "2.5");
    }

    #[test]
    fn float_try_from_f64() {
        assert!(Float::try_from(1.0).is_ok());
        assert!(Float::try_from(f64::NAN).is_err());
    }

    #[test]
    fn float_try_from_str_valid() {
        assert!(Float::try_from("1.5").is_ok());
        assert!(Float::try_from("-3.2").is_ok());
        assert!(Float::try_from("0").is_ok());
    }

    #[test]
    fn float_try_from_str_invalid() {
        assert!(Float::try_from("abc").is_err());
        assert!(Float::try_from("").is_err());
        assert!(Float::try_from("  ").is_err());
        assert!(Float::try_from("inf").is_err());
        assert!(Float::try_from("nan").is_err());
    }

    #[test]
    fn float_try_from_str_rejects_non_finite() {
        assert!(Float::try_from("inf").is_err());
        assert!(Float::try_from("NaN").is_err());
    }

    #[test]
    fn integer_new_and_as_i64() {
        let i = Integer::new(42);
        assert_eq!(i.as_i64(), 42);
    }

    #[test]
    fn integer_checked_operations() {
        let a = Integer::new(10);
        let b = Integer::new(3);
        assert_eq!(a.checked_add(b), Some(Integer::new(13)));
        assert_eq!(a.checked_sub(b), Some(Integer::new(7)));
        assert_eq!(a.checked_mul(b), Some(Integer::new(30)));
        assert_eq!(a.checked_div(b), Some(Integer::new(3)));
    }

    #[test]
    fn integer_checked_operations_overflow() {
        let max = Integer::new(i64::MAX);
        let one = Integer::new(1);
        assert_eq!(max.checked_add(one), None);

        let min = Integer::new(i64::MIN);
        assert_eq!(min.checked_sub(one), None);

        assert_eq!(max.checked_mul(Integer::new(2)), None);
        assert_eq!(min.checked_div(Integer::new(-1)), None);
    }

    #[test]
    fn integer_checked_div_by_zero() {
        assert_eq!(Integer::new(1).checked_div(Integer::new(0)), None);
    }

    #[test]
    fn integer_bytes_roundtrip() {
        let original = Integer::new(-123_456_789);
        let bytes = original.to_le_bytes();
        let reconstructed = Integer::from_le_bytes(bytes);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn integer_display() {
        let i = Integer::new(-7);
        assert_eq!(format!("{i}"), "-7");
    }

    #[test]
    fn integer_from_i64() {
        let i: Integer = 5.into();
        assert_eq!(i.as_i64(), 5);
    }

    #[test]
    fn integer_try_from_str_valid() {
        assert!(Integer::try_from("123").is_ok());
        assert!(Integer::try_from("-456").is_ok());
        assert!(Integer::try_from("0").is_ok());
    }

    #[test]
    fn integer_try_from_str_invalid() {
        assert!(Integer::try_from("12.3").is_err());
        assert!(Integer::try_from("abc").is_err());
        assert!(Integer::try_from("").is_err());
        assert!(Integer::try_from("99999999999999999999999999").is_err());
    }

    #[test]
    fn value_default_is_null() {
        let v = Value::default();
        assert!(v.is_null());
    }

    #[test]
    fn value_null_properties() {
        let v = Value::Null;
        assert!(v.is_null());
        assert!(!v.is_integer());
        assert!(!v.is_float());
        assert!(!v.is_text());
        assert!(!v.is_blob());
        assert!(!v.is_boolean());
        assert_eq!(v.type_name(), "null");
        assert_eq!(v.as_integer(), None);
        assert_eq!(v.as_float(), None);
        assert_eq!(v.as_text(), None);
        assert_eq!(v.as_blob(), None);
        assert_eq!(v.as_boolean(), None);
        assert_eq!(v.as_i64(), None);
        assert_eq!(v.as_f64(), None);
        assert_eq!(v.as_bool(), None);
        assert_eq!(v.as_str(), None);
        assert_eq!(v.to_string(), "NULL");
    }

    #[test]
    fn value_integer_variants() {
        let v = Value::Integer(Integer::new(10));
        assert!(v.is_integer());
        assert_eq!(v.type_name(), "integer");
        assert_eq!(v.as_integer(), Some(&Integer::new(10)));
        assert_eq!(v.as_i64(), Some(10));
        assert_eq!(v.to_string(), "10");
        assert_eq!(v.into_integer(), Some(Integer::new(10)));
    }

    #[test]
    fn value_float_variants() {
        let float = unwrap_result(Float::try_new(1.5));
        let v = Value::Float(float);
        assert!(v.is_float());
        assert_eq!(v.type_name(), "float");
        let expected_float = unwrap_result(Float::try_new(1.5));
        assert_eq!(v.as_float(), Some(&expected_float));
        let f = v.as_f64().unwrap_or_else(|| panic!("should have f64"));
        assert_float_eq(f, 1.5);
        assert_eq!(v.to_string(), "1.5");
    }

    #[test]
    fn value_text_variants() {
        let text = unwrap_result(Text::try_new("hello".to_string()));
        let v = Value::Text(text.clone());
        assert!(v.is_text());
        assert_eq!(v.type_name(), "text");
        assert_eq!(v.as_text(), Some(&text));
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(v.to_string(), "'hello'");
    }

    #[test]
    fn value_blob_variants() {
        let data = vec![1, 2];
        let blob = unwrap_result(Blob::try_new(data));
        let v = Value::Blob(blob.clone());
        assert!(v.is_blob());
        assert_eq!(v.type_name(), "blob");
        assert_eq!(v.as_blob(), Some(&blob));
        assert_eq!(v.to_string(), "Blob(2 bytes)");
    }

    #[test]
    fn value_boolean_variants() {
        let v = Value::Boolean(true);
        assert!(v.is_boolean());
        assert_eq!(v.type_name(), "boolean");
        assert_eq!(v.as_boolean(), Some(true));
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.to_string(), "true");

        let v2 = Value::Boolean(false);
        assert_eq!(v2.as_boolean(), Some(false));
        assert_eq!(v2.to_string(), "false");
    }

    #[test]
    fn value_conversions_from() {
        let v: Value = ().into();
        assert!(v.is_null());

        let v: Value = Integer::new(5).into();
        assert!(v.is_integer());

        let float = unwrap_result(Float::try_new(1.0));
        let v: Value = float.into();
        assert!(v.is_float());

        let text = unwrap_result(Text::try_new("abc".to_string()));
        let v: Value = text.into();
        assert!(v.is_text());

        let blob = unwrap_result(Blob::try_new(vec![1]));
        let v: Value = blob.into();
        assert!(v.is_blob());

        let v: Value = true.into();
        assert!(v.is_boolean());

        let v: Value = 42i64.into();
        assert!(v.is_integer());
    }

    #[test]
    fn value_try_from_float() {
        let v = unwrap_result(Value::try_from(2.5f64));
        assert!(v.is_float());
        let f = v.as_f64().unwrap_or_else(|| panic!("should have f64"));
        assert_float_eq(f, 2.5);

        assert!(Value::try_from(f64::NAN).is_err());
        assert!(Value::try_from(f64::INFINITY).is_err());
    }

    #[test]
    fn value_try_from_string_and_str() {
        let v = unwrap_result(Value::try_from("hello".to_string()));
        assert!(v.is_text());
        assert_eq!(v.as_str(), Some("hello"));

        let v = unwrap_result(Value::try_from("world"));
        assert!(v.is_text());
        assert_eq!(v.as_str(), Some("world"));
    }
    #[test]
    fn value_try_from_bytes() {
        let v = unwrap_result(Value::try_from(vec![1, 2, 3]));
        assert!(v.is_blob());
        let blob = v.as_blob().unwrap_or_else(|| panic!("should be blob"));
        assert_eq!(blob.as_slice(), &[1, 2, 3]);

        let v = unwrap_result(Value::try_from(&[4, 5][..]));
        assert!(v.is_blob());
        let blob = v.as_blob().unwrap_or_else(|| panic!("should be blob"));
        assert_eq!(blob.as_slice(), &[4, 5]);
    }
    #[test]
    fn value_equality() {
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(1))
        );
        assert_ne!(
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(2))
        );
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
    }

    #[test]
    fn row_new_and_len() {
        let text = unwrap_result(Text::try_new("a".to_string()));
        let values = vec![Value::Integer(Integer::new(1)), Value::Text(text)];
        let row = Row::new(values);
        assert_eq!(row.len(), 2);
        assert!(!row.is_empty());
        assert_eq!(row.values().len(), 2);
    }

    #[test]
    fn row_empty() {
        let row = Row::new(Vec::new());
        assert_eq!(row.len(), 0);
        assert!(row.is_empty());
    }

    #[test]
    fn row_get_by_index() {
        let v1 = Value::Integer(Integer::new(10));
        let v2 = Value::Boolean(true);
        let row = Row::new(vec![v1.clone(), v2.clone()]);
        assert_eq!(row.get(&0usize), Some(&v1));
        assert_eq!(row.get(&1usize), Some(&v2));
        assert_eq!(row.get(&2usize), None);
    }

    #[test]
    fn row_get_by_name_using_schema() {
        let schema = unwrap_result(TableSchema::try_new(
            "test",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        let text = unwrap_result(Text::try_new("alice".to_string()));
        let row = Row::new(vec![
            Value::Integer(Integer::new(1)),
            Value::Text(text.clone()),
        ]);
        let idx = unwrap_option(schema.column_index("name"));
        assert_eq!(row.get(&idx), Some(&Value::Text(text)));
    }

    #[test]
    fn data_type_as_str() {
        assert_eq!(DataType::Null.as_str(), "NULL");
        assert_eq!(DataType::Integer.as_str(), "INTEGER");
        assert_eq!(DataType::Float.as_str(), "FLOAT");
        assert_eq!(DataType::Text.as_str(), "TEXT");
        assert_eq!(DataType::Blob.as_str(), "BLOB");
        assert_eq!(DataType::Boolean.as_str(), "BOOLEAN");
    }

    #[test]
    fn data_type_display() {
        assert_eq!(format!("{}", DataType::Null), "NULL");
        assert_eq!(format!("{}", DataType::Integer), "INTEGER");
        assert_eq!(format!("{}", DataType::Float), "FLOAT");
        assert_eq!(format!("{}", DataType::Text), "TEXT");
        assert_eq!(format!("{}", DataType::Blob), "BLOB");
        assert_eq!(format!("{}", DataType::Boolean), "BOOLEAN");
    }

    #[test]
    fn column_def_new_defaults() {
        let col = ColumnDef::new("id", DataType::Integer);
        assert_eq!(col.name(), "id");
        assert_eq!(col.data_type(), &DataType::Integer);
        assert!(col.is_nullable());
        assert!(!col.is_primary_key());
        assert!(!col.is_unique());
        assert_eq!(col.default_value(), None);
        assert_eq!(col.check_constraint(), None);
        assert_eq!(col.foreign_key(), None);
        assert_eq!(col.allowed_values(), None);
    }

    #[test]
    fn column_def_setters() {
        let mut col = ColumnDef::new("id", DataType::Integer);
        col.set_nullable(false);
        assert!(!col.is_nullable());

        col.set_nullable(true);
        assert!(col.is_nullable());

        col.set_primary_key(true);
        assert!(col.is_primary_key());
        assert!(col.is_unique());
        assert!(!col.is_nullable());

        col.set_nullable(true);
        assert!(!col.is_nullable());
        assert!(col.is_primary_key());

        col.set_unique(false);
        assert!(col.is_unique());
    }

    #[test]
    fn column_def_set_default() {
        let mut col = ColumnDef::new("age", DataType::Integer);
        col.set_default(Some(Value::Integer(Integer::new(0))));
        assert_eq!(col.default_value(), Some(&Value::Integer(Integer::new(0))));

        col.set_default(None);
        assert_eq!(col.default_value(), None);
    }

    #[test]
    fn column_def_set_check() {
        let mut col = ColumnDef::new("score", DataType::Integer);
        let check = CheckConstraint {
            column: "score".to_string(),
            op: ComparisonOp::Gte,
            value: Value::Integer(Integer::new(0)),
        };
        col.set_check(Some(check.clone()));
        assert_eq!(col.check_constraint(), Some(&check));

        col.set_check(None);
        assert_eq!(col.check_constraint(), None);
    }

    #[test]
    fn column_def_set_foreign_key() {
        let mut col = ColumnDef::new("user_id", DataType::Integer);
        let fk = ForeignKey {
            table: "users".to_string(),
            column: "id".to_string(),
        };
        col.set_foreign_key(Some(fk.clone()));
        assert_eq!(col.foreign_key(), Some(&fk));

        col.set_foreign_key(None);
        assert_eq!(col.foreign_key(), None);
    }

    #[test]
    fn column_def_set_allowed_values() {
        let mut col = ColumnDef::new("status", DataType::Text);
        let active = unwrap_result(Text::try_new("active".to_string()));
        let inactive = unwrap_result(Text::try_new("inactive".to_string()));
        let allowed = vec![Value::Text(active), Value::Text(inactive)];
        col.set_allowed_values(Some(allowed.clone()));
        assert_eq!(col.allowed_values(), Some(&allowed));

        col.set_allowed_values(None);
        assert_eq!(col.allowed_values(), None);
    }

    #[test]
    fn column_def_validate_value_null() {
        let nullable_col = ColumnDef::new("x", DataType::Integer);
        assert!(nullable_col.validate_value(&Value::Null).is_ok());

        let mut non_nullable = ColumnDef::new("y", DataType::Integer);
        non_nullable.set_nullable(false);
        let result = non_nullable.validate_value(&Value::Null);
        assert!(result.is_err());
        match result {
            Err(DbError::ConstraintViolation {
                kind,
                message,
                constraint,
                table,
            }) => {
                assert_eq!(kind, ErrorKind::NotNullViolation);
                assert!(message.contains("not nullable"));
                assert_eq!(constraint, Some("y".to_string()));
                assert_eq!(table, None);
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn column_def_validate_value_type_mismatch() {
        let col = ColumnDef::new("age", DataType::Integer);
        let text = unwrap_result(Text::try_new("abc".to_string()));
        let result = col.validate_value(&Value::Text(text));
        assert!(result.is_err());
        match result {
            Err(DbError::TypeMismatch(msg)) => {
                assert!(msg.contains("expects INTEGER"));
                assert!(msg.contains("got text"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn column_def_validate_value_check_constraint_integer() {
        let mut col = ColumnDef::new("score", DataType::Integer);
        col.set_check(Some(CheckConstraint {
            column: "score".to_string(),
            op: ComparisonOp::Gte,
            value: Value::Integer(Integer::new(0)),
        }));
        assert!(col.validate_value(&Value::Integer(Integer::new(5))).is_ok());
        assert!(col.validate_value(&Value::Integer(Integer::new(0))).is_ok());
        let result = col.validate_value(&Value::Integer(Integer::new(-1)));
        assert!(result.is_err());
        match result {
            Err(DbError::ConstraintViolation { kind, message, .. }) => {
                assert_eq!(kind, ErrorKind::CheckViolation);
                assert!(message.contains("check constraint failed"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn column_def_validate_value_check_constraint_float() {
        let mut col = ColumnDef::new("value", DataType::Float);
        let two = unwrap_result(Float::try_new(2.0));
        col.set_check(Some(CheckConstraint {
            column: "value".to_string(),
            op: ComparisonOp::Lt,
            value: Value::Float(two),
        }));
        let one = unwrap_result(Float::try_new(1.0));
        assert!(col.validate_value(&Value::Float(one)).is_ok());
        let two = unwrap_result(Float::try_new(2.0));
        assert!(col.validate_value(&Value::Float(two)).is_err());
    }

    #[test]
    fn column_def_validate_value_allowed_values() {
        let mut col = ColumnDef::new("color", DataType::Text);
        let red = unwrap_result(Text::try_new("red".to_string()));
        let green = unwrap_result(Text::try_new("green".to_string()));
        col.set_allowed_values(Some(vec![Value::Text(red.clone()), Value::Text(green)]));
        assert!(col.validate_value(&Value::Text(red)).is_ok());
        let blue = unwrap_result(Text::try_new("blue".to_string()));
        let result = col.validate_value(&Value::Text(blue));
        assert!(result.is_err());
        match result {
            Err(DbError::ConstraintViolation { kind, message, .. }) => {
                assert_eq!(kind, ErrorKind::CheckViolation);
                assert!(message.contains("allowed list"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_try_new_valid() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        assert_eq!(schema.name(), "users");
        assert_eq!(schema.columns().len(), 2);
    }

    #[test]
    fn table_schema_try_new_empty_name() {
        let result = TableSchema::try_new("", vec![ColumnDef::new("id", DataType::Integer)]);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => {
                assert!(msg.contains("table name cannot be empty"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_try_new_too_long_name() {
        let name = "a".repeat(MAX_NAME_LENGTH + 1);
        let result = TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)]);
        assert!(result.is_err());
    }

    #[test]
    fn table_schema_try_new_control_char_name() {
        let result =
            TableSchema::try_new("bad\nname", vec![ColumnDef::new("id", DataType::Integer)]);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("control characters")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_try_new_no_columns() {
        let result = TableSchema::try_new("empty", Vec::new());
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("at least one column")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_try_new_too_many_columns() {
        let columns = (0..=MAX_COLUMNS)
            .map(|i| ColumnDef::new(format!("col{i}"), DataType::Integer))
            .collect();
        let result = TableSchema::try_new("many", columns);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("too many columns")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_try_new_duplicate_column_name() {
        let result = TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("ID", DataType::Integer),
            ],
        );
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("duplicate column name")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_column_index() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        assert_eq!(schema.column_index("id"), Some(0));
        assert_eq!(schema.column_index("name"), Some(1));
        assert_eq!(schema.column_index("NAME"), Some(1));
        assert_eq!(schema.column_index("missing"), None);
    }

    #[test]
    fn table_schema_get_column() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        assert!(schema.get_column("id").is_some());
        assert!(schema.get_column("name").is_some());
        assert!(schema.get_column("missing").is_none());
    }

    #[test]
    fn table_schema_validate_values_valid() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        let text = unwrap_result(Text::try_new("alice".to_string()));
        let values = vec![Value::Integer(Integer::new(1)), Value::Text(text)];
        assert!(schema.validate_values(&values).is_ok());
    }

    #[test]
    fn table_schema_validate_values_wrong_count() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        let too_few = vec![Value::Integer(Integer::new(1))];
        let result = schema.validate_values(&too_few);
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => {
                assert!(msg.contains("expected 2 values, got 1"));
            }
            _ => panic!("wrong error"),
        }

        let text = unwrap_result(Text::try_new("a".to_string()));
        let too_many = vec![
            Value::Integer(Integer::new(1)),
            Value::Text(text),
            Value::Null,
        ];
        assert!(schema.validate_values(&too_many).is_err());
    }

    #[test]
    fn table_schema_validate_values_type_error() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![ColumnDef::new("id", DataType::Integer)],
        ));
        let text = unwrap_result(Text::try_new("not int".to_string()));
        let values = vec![Value::Text(text)];
        let result = schema.validate_values(&values);
        assert!(result.is_err());
        match result {
            Err(DbError::TypeMismatch(_)) => (),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn table_schema_get_column_by_index() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        assert_eq!(
            unwrap_option(schema.get_column_by_index(&0usize)).name(),
            "id"
        );
        assert_eq!(
            unwrap_option(schema.get_column_by_index(&1usize)).name(),
            "name"
        );
        assert_eq!(
            unwrap_option(schema.get_column_by_index(&"name")).name(),
            "name"
        );
        assert!(schema.get_column_by_index(&2usize).is_none());
        assert!(schema.get_column_by_index(&"missing").is_none());
    }

    #[test]
    fn db_error_constructors() {
        assert!(matches!(
            DbError::table_not_found("users"),
            DbError::TableNotFound(name) if name == "users"
        ));
        assert!(matches!(
            DbError::column_not_found("id"),
            DbError::ColumnNotFound(name) if name == "id"
        ));
        assert!(matches!(
            DbError::type_mismatch("bad type"),
            DbError::TypeMismatch(msg) if msg == "bad type"
        ));
        assert!(matches!(
            DbError::invalid_operation("invalid"),
            DbError::InvalidOperation(msg) if msg == "invalid"
        ));
        assert!(matches!(
            DbError::invalid_query("bad query"),
            DbError::InvalidQuery(msg) if msg == "bad query"
        ));
        assert!(matches!(
            DbError::unsupported("not supported"),
            DbError::Unsupported(msg) if msg == "not supported"
        ));
        assert!(matches!(
            DbError::from_io(std::io::Error::other("io error")),
            DbError::Io(_)
        ));
    }

    #[test]
    fn db_error_constraint_violation() {
        let err = DbError::constraint_violation(
            ErrorKind::UniqueViolation,
            "duplicate key",
            Some("idx".to_string()),
            Some("users".to_string()),
        );
        if let DbError::ConstraintViolation {
            kind,
            message,
            constraint,
            table,
        } = err
        {
            assert_eq!(kind, ErrorKind::UniqueViolation);
            assert_eq!(message, "duplicate key");
            assert_eq!(constraint, Some("idx".to_string()));
            assert_eq!(table, Some("users".to_string()));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn db_error_display() {
        let e = DbError::TableNotFound("users".to_string());
        assert_eq!(format!("{e}"), "Table not found: users");

        let e = DbError::ColumnNotFound("id".to_string());
        assert_eq!(format!("{e}"), "Column not found: id");

        let e = DbError::TypeMismatch("bad type".to_string());
        assert_eq!(format!("{e}"), "Type mismatch: bad type");

        let e = DbError::InvalidOperation("invalid".to_string());
        assert_eq!(format!("{e}"), "Invalid operation: invalid");

        let e = DbError::ConstraintViolation {
            kind: ErrorKind::CheckViolation,
            message: "check failed".to_string(),
            constraint: Some("c".to_string()),
            table: None,
        };
        assert_eq!(format!("{e}"), "Constraint violation: check failed");
    }

    #[test]
    fn db_error_source() {
        let io_err = std::io::Error::other("file not found");
        let db_err = DbError::from_io(io_err);
        assert!(db_err.source().is_some());

        let err = DbError::TableNotFound("t".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn db_error_partial_eq() {
        assert_eq!(
            DbError::TableNotFound("a".to_string()),
            DbError::TableNotFound("a".to_string())
        );
        assert_ne!(
            DbError::TableNotFound("a".to_string()),
            DbError::TableNotFound("b".to_string())
        );

        assert_eq!(
            DbError::constraint_violation(ErrorKind::UniqueViolation, "dup", None, None),
            DbError::constraint_violation(ErrorKind::UniqueViolation, "dup", None, None)
        );
        assert_ne!(
            DbError::constraint_violation(ErrorKind::UniqueViolation, "dup", None, None),
            DbError::constraint_violation(ErrorKind::UniqueViolation, "dup2", None, None)
        );
    }

    #[test]
    fn db_error_monumentum_trait_methods() {
        let err = DbError::constraint_violation(
            ErrorKind::ForeignKeyViolation,
            "fk failed",
            Some("fk_constraint".to_string()),
            Some("users".to_string()),
        );
        assert_eq!(err.kind(), ErrorKind::ForeignKeyViolation);
        assert_eq!(err.message(), "fk failed");
        assert_eq!(err.constraint(), Some("fk_constraint"));
        assert_eq!(err.table(), Some("users"));
        assert!(!err.is_unique_violation());
        assert!(err.is_foreign_key_violation());
        assert!(!err.is_not_null_violation());
        assert!(!err.is_check_violation());
        assert!(!err.is_type_mismatch());
    }

    #[test]
    fn db_error_is_methods() {
        let unique = DbError::constraint_violation(ErrorKind::UniqueViolation, "dup", None, None);
        assert!(unique.is_unique_violation());
        assert!(!unique.is_foreign_key_violation());

        let not_null =
            DbError::constraint_violation(ErrorKind::NotNullViolation, "not null", None, None);
        assert!(not_null.is_not_null_violation());
        assert!(!not_null.is_unique_violation());

        let check = DbError::constraint_violation(ErrorKind::CheckViolation, "check", None, None);
        assert!(check.is_check_violation());

        let type_err = DbError::TypeMismatch("bad".to_string());
        assert!(type_err.is_type_mismatch());
    }

    #[test]
    fn validate_name_valid() {
        assert!(validate_name("valid_name").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name(&"x".repeat(MAX_NAME_LENGTH)).is_ok());
    }

    #[test]
    fn validate_name_empty() {
        let result = validate_name("");
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("name cannot be empty")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn validate_name_too_long() {
        let s = "x".repeat(MAX_NAME_LENGTH + 1);
        assert!(validate_name(&s).is_err());
    }

    #[test]
    fn validate_name_control_chars() {
        assert!(validate_name("bad\nname").is_err());
        assert!(validate_name("tab\tname").is_err());
        assert!(validate_name("nul\0name").is_err());
    }

    #[test]
    fn validate_column_name_wrapper() {
        assert!(validate_column_name("col").is_ok());
        let result = validate_column_name("");
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("invalid column name")),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn validate_table_name_wrapper() {
        assert!(validate_table_name("table").is_ok());
        let result = validate_table_name("");
        assert!(result.is_err());
        match result {
            Err(DbError::InvalidOperation(msg)) => assert!(msg.contains("invalid table name")),
            _ => panic!("wrong error"),
        }
    }

    struct MockIndex {
        entries: Vec<(Value, Vec<usize>)>,
    }
    impl MockIndex {
        const fn new() -> Self {
            Self {
                entries: Vec::new(),
            }
        }
    }
    impl Index for MockIndex {
        fn insert(&mut self, key: &Value, row_idx: usize) {
            if let Some(entry) = self.entries.iter_mut().find(|(k, _)| k == key) {
                entry.1.push(row_idx);
            } else {
                self.entries.push((key.clone(), vec![row_idx]));
            }
        }
        fn remove(&mut self, key: &Value, row_idx: usize) {
            if let Some(entry) = self.entries.iter_mut().find(|(k, _)| k == key)
                && let Some(pos) = entry.1.iter().position(|&i| i == row_idx)
            {
                let _ = entry.1.remove(pos);
                if entry.1.is_empty() {
                    self.entries.retain(|(k, _)| k != key);
                }
            }
        }
        fn lookup(&self, key: &Value) -> Option<&[usize]> {
            self.entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_slice())
        }
    }

    #[test]
    fn index_trait_basic() {
        let mut index = MockIndex::new();
        let key1 = Value::Integer(Integer::new(1));
        let key2 = Value::Integer(Integer::new(2));
        index.insert(&key1, 10);
        index.insert(&key1, 20);
        index.insert(&key2, 30);
        assert_eq!(index.lookup(&key1), Some(&[10, 20][..]));
        assert_eq!(index.lookup(&key2), Some(&[30][..]));
        assert_eq!(index.lookup(&Value::Null), None);

        index.remove(&key1, 10);
        assert_eq!(index.lookup(&key1), Some(&[20][..]));
        index.remove(&key1, 20);
        assert_eq!(index.lookup(&key1), None);
    }

    struct MockCatalogStore {
        tables: Vec<(String, TableSchema)>,
    }

    impl MockCatalogStore {
        const fn new() -> Self {
            Self { tables: Vec::new() }
        }
    }

    impl CatalogStore for MockCatalogStore {
        fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
            let name = schema.name().to_string();
            if self.tables.iter().any(|(n, _)| n == &name) {
                return Err(DbError::invalid_operation(format!(
                    "table {name} already exists"
                )));
            }
            self.tables.push((name, schema));
            Ok(())
        }

        fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
            if let Some(pos) = self.tables.iter().position(|(n, _)| n == name) {
                let _ = self.tables.remove(pos);
                Ok(())
            } else {
                Err(DbError::table_not_found(name))
            }
        }

        fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
            if let Some(pos) = self.tables.iter().position(|(n, _)| n == old_name) {
                let (_, schema) = self.tables.remove(pos);
                self.tables.push((new_name.to_string(), schema));
                Ok(())
            } else {
                Err(DbError::table_not_found(old_name))
            }
        }
    }

    #[test]
    fn catalog_store_trait() {
        let mut catalog = MockCatalogStore::new();
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![ColumnDef::new("id", DataType::Integer)],
        ));
        assert!(catalog.create_table(schema.clone()).is_ok());
        assert!(catalog.create_table(schema).is_err());

        assert!(catalog.drop_table("users").is_ok());
        assert!(catalog.drop_table("users").is_err());

        let schema2 = unwrap_result(TableSchema::try_new(
            "customers",
            vec![ColumnDef::new("id", DataType::Integer)],
        ));
        assert!(catalog.create_table(schema2).is_ok());
        assert!(catalog.rename_table("customers", "clients").is_ok());
        assert!(catalog.rename_table("customers", "x").is_err());
    }

    struct MockStorageEngine {
        catalog: Vec<TableSchema>,
    }
    impl MockStorageEngine {
        const fn new() -> Self {
            Self {
                catalog: Vec::new(),
            }
        }
    }
    impl StorageEngine for MockStorageEngine {
        fn load_catalog(&mut self) -> Result<(), DbError> {
            Ok(())
        }
        fn save_catalog(&mut self) -> Result<(), DbError> {
            Ok(())
        }
        fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
            self.catalog.push(schema);
            Ok(())
        }
    }

    #[test]
    fn storage_engine_trait() {
        let mut engine = MockStorageEngine::new();
        assert!(engine.load_catalog().is_ok());
        assert!(engine.save_catalog().is_ok());
        let schema = unwrap_result(TableSchema::try_new(
            "t",
            vec![ColumnDef::new("c", DataType::Integer)],
        ));
        assert!(engine.create_table(schema).is_ok());
        assert_eq!(engine.catalog.len(), 1);
    }

    struct MockTableStore {
        rows: Vec<Row>,
        schema: TableSchema,
    }
    impl MockTableStore {
        const fn new(schema: TableSchema) -> Self {
            Self {
                rows: Vec::new(),
                schema,
            }
        }
    }
    impl TableStore for MockTableStore {
        fn insert(&mut self, row: &Row) -> Result<(), DbError> {
            self.schema.validate_values(row.values())?;
            self.rows.push(row.clone());
            Ok(())
        }
        fn set_cell(
            &mut self,
            row_idx: usize,
            col_idx: usize,
            value: Value,
        ) -> Result<(), DbError> {
            let row = self
                .rows
                .get(row_idx)
                .ok_or_else(|| DbError::invalid_operation("row index out of bounds"))?;
            if col_idx >= row.len() {
                return Err(DbError::invalid_operation("column index out of bounds"));
            }
            let col = self
                .schema
                .columns()
                .get(col_idx)
                .ok_or_else(|| DbError::invalid_operation("column index out of bounds"))?;
            col.validate_value(&value)?;
            Ok(())
        }
        fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError> {
            for row in &rows {
                self.schema.validate_values(row.values())?;
            }
            self.rows = rows;
            Ok(())
        }
    }

    #[test]
    fn table_store_trait() {
        let schema = unwrap_result(TableSchema::try_new(
            "users",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("name", DataType::Text),
            ],
        ));
        let mut store = MockTableStore::new(schema);

        let text = unwrap_result(Text::try_new("alice".to_string()));
        let valid_row = Row::new(vec![Value::Integer(Integer::new(1)), Value::Text(text)]);
        assert!(store.insert(&valid_row).is_ok());
        assert_eq!(store.rows.len(), 1);

        let bad_text = unwrap_result(Text::try_new("bad".to_string()));
        let bob_text = unwrap_result(Text::try_new("bob".to_string()));
        let invalid_row = Row::new(vec![Value::Text(bad_text), Value::Text(bob_text)]);
        assert!(store.insert(&invalid_row).is_err());

        let new_value = Value::Integer(Integer::new(2));
        assert!(store.set_cell(0, 0, new_value).is_ok());
        let x_text = unwrap_result(Text::try_new("x".to_string()));
        assert!(store.set_cell(0, 0, Value::Text(x_text)).is_err());
        assert!(store.set_cell(10, 0, Value::Null).is_err());
        assert!(store.set_cell(0, 10, Value::Null).is_err());

        let rows = vec![valid_row.clone()];
        assert!(store.replace_rows(rows).is_ok());
        assert_eq!(store.rows.len(), 1);
    }

    #[test]
    fn column_index_usize_for_row() {
        let row = Row::new(vec![Value::Null, Value::Null]);
        assert_eq!(unwrap_result(0usize.index(&row)), 0);
        assert_eq!(unwrap_result(1usize.index(&row)), 1);
        assert!(2usize.index(&row).is_err());
    }

    #[test]
    fn column_index_usize_for_schema() {
        let schema = unwrap_result(TableSchema::try_new(
            "t",
            vec![
                ColumnDef::new("a", DataType::Integer),
                ColumnDef::new("b", DataType::Text),
            ],
        ));
        assert_eq!(unwrap_result(0usize.index(&schema)), 0);
        assert_eq!(unwrap_result(1usize.index(&schema)), 1);
        assert!(2usize.index(&schema).is_err());
    }

    #[test]
    fn column_index_str_for_schema() {
        let schema = unwrap_result(TableSchema::try_new(
            "t",
            vec![
                ColumnDef::new("a", DataType::Integer),
                ColumnDef::new("b", DataType::Text),
            ],
        ));
        assert_eq!(unwrap_result("a".index(&schema)), 0);
        assert_eq!(unwrap_result("b".index(&schema)), 1);
        assert!("c".index(&schema).is_err());
    }

    #[test]
    fn value_try_from_invalid_float() {
        assert!(Value::try_from(f64::NAN).is_err());
        assert!(Value::try_from(f64::INFINITY).is_err());
    }

    #[test]
    fn value_try_from_invalid_string_size() {
        let s1 = "x".repeat(MAX_TEXT_SIZE + 1);
        let s2 = s1.clone();
        assert!(Value::try_from(s1).is_err());
        assert!(Value::try_from(s2.as_str()).is_err());
    }

    #[test]
    fn value_try_from_invalid_blob_size() {
        let v1 = vec![0u8; MAX_BLOB_SIZE + 1];
        let v2 = v1.clone();
        assert!(Value::try_from(v1).is_err());
        assert!(Value::try_from(&v2[..]).is_err());
    }

    #[test]
    fn row_partial_eq() {
        let r1 = Row::new(vec![Value::Integer(Integer::new(1))]);
        let r2 = Row::new(vec![Value::Integer(Integer::new(1))]);
        let r3 = Row::new(vec![Value::Integer(Integer::new(2))]);
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn value_partial_ord() {
        let a = Value::Integer(Integer::new(1));
        let b = Value::Integer(Integer::new(2));
        assert!(a < b);
        let c = Value::Text(unwrap_result(Text::try_new("a".to_string())));
        let d = Value::Text(unwrap_result(Text::try_new("b".to_string())));
        assert!(c < d);
    }
}
