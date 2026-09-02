use monumentum_db::core::schema::column::{CheckConstraint, ColumnDef, ComparisonOp, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn int_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Integer)
}

fn text_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Text)
}

fn float_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Float)
}

fn blob_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Blob)
}

fn nullable_col(mut col: ColumnDef, nullable: bool) -> ColumnDef {
    col.set_nullable(nullable);
    col
}

fn with_check(mut col: ColumnDef, check: CheckConstraint) -> ColumnDef {
    col.set_check(Some(check));
    col
}

fn create_schema(name: &str, columns: Vec<ColumnDef>) -> Result<TableSchema, DbError> {
    TableSchema::try_new(name, columns)
}

#[test]
fn try_new_valid_creates_schema() {
    let result = create_schema("users", vec![int_col("id"), text_col("name")]);
    assert!(result.is_ok());
    let schema = result.unwrap();
    assert_eq!(schema.name(), "users");
    assert_eq!(schema.columns().len(), 2);
}

#[test]
fn try_new_empty_table_name_returns_error() {
    let result = create_schema("", vec![int_col("id")]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
        assert_eq!(
            e.to_string(),
            "Invalid operation: table name cannot be empty"
        );
    }
}

#[test]
fn try_new_empty_columns_returns_error() {
    let result = create_schema("users", vec![]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
        assert_eq!(
            e.to_string(),
            "Invalid operation: table must have at least one column"
        );
    }
}

#[test]
fn try_new_column_with_empty_name_returns_error() {
    let col = int_col("");
    let result = create_schema("users", vec![col]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
        assert_eq!(
            e.to_string(),
            "Invalid operation: column name cannot be empty"
        );
    }
}

#[test]
fn try_new_duplicate_column_name_case_sensitive_returns_error() {
    let result = create_schema("users", vec![int_col("id"), int_col("id")]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
        assert_eq!(
            e.to_string(),
            "Invalid operation: duplicate column name 'id'"
        );
    }
}

#[test]
fn try_new_duplicate_column_name_case_insensitive_returns_error() {
    let result = create_schema("users", vec![int_col("ID"), int_col("id")]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
        assert!(e.to_string().contains("duplicate column name"));
    }
}

#[test]
fn name_returns_correct_name() {
    let schema = create_schema("products", vec![int_col("id")]).unwrap();
    assert_eq!(schema.name(), "products");
}

#[test]
fn columns_returns_slice() {
    let schema = create_schema("products", vec![int_col("id"), text_col("name")]).unwrap();
    let cols = schema.columns();
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name(), "id");
    assert_eq!(cols[1].name(), "name");
}

#[test]
fn column_index_exact_match() {
    let schema = create_schema("products", vec![int_col("id"), text_col("name")]).unwrap();
    assert_eq!(schema.column_index("id"), Some(0));
    assert_eq!(schema.column_index("name"), Some(1));
}

#[test]
fn column_index_case_insensitive_match() {
    let schema = create_schema("products", vec![int_col("id"), text_col("name")]).unwrap();
    assert_eq!(schema.column_index("ID"), Some(0));
    assert_eq!(schema.column_index("Name"), Some(1));
}

#[test]
fn column_index_not_found_returns_none() {
    let schema = create_schema("products", vec![int_col("id")]).unwrap();
    assert_eq!(schema.column_index("nonexistent"), None);
}

#[test]
fn get_column_returns_reference() {
    let schema = create_schema("products", vec![int_col("id"), text_col("name")]).unwrap();
    let col = schema.get_column("name");
    assert!(col.is_some());
    assert_eq!(col.unwrap().name(), "name");
    assert_eq!(col.unwrap().data_type(), &DataType::Text);
}

#[test]
fn get_column_case_insensitive_returns_reference() {
    let schema = create_schema("products", vec![int_col("id"), text_col("name")]).unwrap();
    let col = schema.get_column("NAME");
    assert!(col.is_some());
    assert_eq!(col.unwrap().name(), "name");
}

#[test]
fn get_column_not_found_returns_none() {
    let schema = create_schema("products", vec![int_col("id")]).unwrap();
    assert!(schema.get_column("missing").is_none());
}

#[test]
fn validate_values_correct_length_and_types_ok() {
    let schema = create_schema("users", vec![int_col("id"), text_col("name")]).unwrap();
    let values = vec![Value::from(1_i64), Value::from("Alice")];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_wrong_length_returns_error() {
    let schema = create_schema("users", vec![int_col("id"), text_col("name")]).unwrap();
    let values = vec![Value::from(1_i64)];
    let result = schema.validate_values(&values);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: expected 2 values, got 1");
    }
}

#[test]
fn validate_values_null_in_non_nullable_column_returns_error() {
    let schema = create_schema("users", vec![nullable_col(int_col("id"), false)]).unwrap();
    let values = vec![Value::Null];
    let result = schema.validate_values(&values);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: column 'id' is not nullable"
        );
    }
}

#[test]
fn validate_values_null_in_nullable_column_ok() {
    let schema = create_schema("users", vec![nullable_col(int_col("age"), true)]).unwrap();
    let values = vec![Value::Null];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_type_mismatch_returns_error() {
    let schema = create_schema("users", vec![int_col("id")]).unwrap();
    let values = vec![Value::from("not integer")];
    let result = schema.validate_values(&values);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TypeMismatch(_)));
        assert_eq!(
            e.to_string(),
            "Type mismatch: column 'id' expects INTEGER, got text"
        );
    }
}

#[test]
fn validate_values_integer_check_eq_pass() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from(30_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(30_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_integer_check_eq_fail() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from(30_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(31_i64)];
    let result = schema.validate_values(&values);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("check constraint failed"));
    }
}

#[test]
fn validate_values_integer_check_neq_pass() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::NotEq,
        value: Value::from(0_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(1_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_integer_check_lt_pass() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Lt,
        value: Value::from(18_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(17_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_integer_check_lt_fail() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Lt,
        value: Value::from(18_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(18_i64)];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_integer_check_lte_pass_equal() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Lte,
        value: Value::from(18_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(18_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_integer_check_gt_pass() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(1_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_integer_check_gte_pass_equal() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gte,
        value: Value::from(18_i64),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(18_i64)];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_float_check_eq_pass() {
    let check = CheckConstraint {
        column: "price".to_string(),
        op: ComparisonOp::Eq,
        value: Value::try_from(9.99_f64).unwrap(),
    };
    let col = with_check(float_col("price"), check);
    let schema = create_schema("products", vec![col]).unwrap();
    let values = vec![Value::try_from(9.99_f64).unwrap()];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_float_check_lt_fail() {
    let check = CheckConstraint {
        column: "price".to_string(),
        op: ComparisonOp::Lt,
        value: Value::try_from(10.0_f64).unwrap(),
    };
    let col = with_check(float_col("price"), check);
    let schema = create_schema("products", vec![col]).unwrap();
    let values = vec![Value::try_from(10.5_f64).unwrap()];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_text_check_eq_pass() {
    let check = CheckConstraint {
        column: "status".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from("active"),
    };
    let col = with_check(text_col("status"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from("active")];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_text_check_not_eq_fail() {
    let check = CheckConstraint {
        column: "status".to_string(),
        op: ComparisonOp::NotEq,
        value: Value::from("inactive"),
    };
    let col = with_check(text_col("status"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from("inactive")];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_text_check_lt_pass() {
    let check = CheckConstraint {
        column: "name".to_string(),
        op: ComparisonOp::Lt,
        value: Value::from("m"),
    };
    let col = with_check(text_col("name"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from("alice")];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_blob_check_eq_pass() {
    let check = CheckConstraint {
        column: "data".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from(vec![1_u8, 2, 3]),
    };
    let col = with_check(blob_col("data"), check);
    let schema = create_schema("files", vec![col]).unwrap();
    let values = vec![Value::from(vec![1_u8, 2, 3])];
    assert!(schema.validate_values(&values).is_ok());
}

#[test]
fn validate_values_blob_check_eq_fail() {
    let check = CheckConstraint {
        column: "data".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from(vec![1_u8, 2, 3]),
    };
    let col = with_check(blob_col("data"), check);
    let schema = create_schema("files", vec![col]).unwrap();
    let values = vec![Value::from(vec![4_u8, 5])];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_blob_check_lt_returns_false_and_fails() {
    let check = CheckConstraint {
        column: "data".to_string(),
        op: ComparisonOp::Lt,
        value: Value::from(vec![1_u8]),
    };
    let col = with_check(blob_col("data"), check);
    let schema = create_schema("files", vec![col]).unwrap();
    let values = vec![Value::from(vec![1_u8])];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_check_with_mismatched_types_fails() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Eq,
        value: Value::from("30"),
    };
    let col = with_check(int_col("age"), check);
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::from(30_i64)];
    assert!(schema.validate_values(&values).is_err());
}

#[test]
fn validate_values_null_skips_check_constraint() {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    let mut col = int_col("age");
    col.set_nullable(true);
    col.set_check(Some(check));
    let schema = create_schema("users", vec![col]).unwrap();
    let values = vec![Value::Null];
    assert!(schema.validate_values(&values).is_ok());
}
