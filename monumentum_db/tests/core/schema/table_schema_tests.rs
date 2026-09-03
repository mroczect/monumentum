use monumentum_db::core::schema::column::{
    CheckConstraint, ColumnDef, ColumnIndex, ComparisonOp, DataType,
};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use proptest::prelude::*;

fn make_schema(name: &str, columns: Vec<ColumnDef>) -> TableSchema {
    TableSchema::try_new(name, columns).expect("valid schema")
}

#[test]
fn get_column_by_index_with_usize() {
    let schema = make_schema(
        "products",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert_eq!(schema.get_column_by_index(0).unwrap().name(), "id");
    assert_eq!(schema.get_column_by_index(1).unwrap().name(), "name");
    assert!(schema.get_column_by_index(2).is_none());
}

#[test]
fn get_column_by_index_with_str() {
    let schema = make_schema(
        "products",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert_eq!(schema.get_column_by_index("id").unwrap().name(), "id");
    assert_eq!(schema.get_column_by_index("NAME").unwrap().name(), "name");
    assert!(schema.get_column_by_index("missing").is_none());
}

#[test]
fn column_index_trait_usize_bounds() {
    let schema = make_schema(
        "test",
        vec![
            ColumnDef::new("a", DataType::Integer),
            ColumnDef::new("b", DataType::Text),
        ],
    );
    assert_eq!(usize::index(&0, &schema), Ok(0));
    assert_eq!(usize::index(&1, &schema), Ok(1));
    assert!(usize::index(&2, &schema).is_err());
}

#[test]
fn column_index_trait_str_case_insensitive() {
    let schema = make_schema(
        "test",
        vec![
            ColumnDef::new("Alpha", DataType::Integer),
            ColumnDef::new("Beta", DataType::Text),
        ],
    );
    assert_eq!(<&str as ColumnIndex<_>>::index(&"alpha", &schema), Ok(0));
    assert_eq!(<&str as ColumnIndex<_>>::index(&"BETA", &schema), Ok(1));
    assert!(<&str as ColumnIndex<_>>::index(&"gamma", &schema).is_err());
}

#[test]
fn validate_values_allowed_values_pass() {
    let mut col = ColumnDef::new("status", DataType::Text);
    col.set_allowed_values(Some(vec![Value::from("active"), Value::from("inactive")]));
    let schema = make_schema("users", vec![col]);
    assert!(schema.validate_values(&[Value::from("active")]).is_ok());
    assert!(schema.validate_values(&[Value::from("inactive")]).is_ok());
    assert!(schema.validate_values(&[Value::from("pending")]).is_err());
}

#[test]
fn validate_values_allowed_values_and_check_constraint_combined() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    col.set_check(Some(CheckConstraint {
        column: "age".into(),
        op: ComparisonOp::Gte,
        value: Value::from(18_i64),
    }));
    col.set_allowed_values(Some(vec![
        Value::from(18_i64),
        Value::from(21_i64),
        Value::from(30_i64),
    ]));
    let schema = make_schema("users", vec![col]);

    assert!(schema.validate_values(&[Value::from(18_i64)]).is_ok());
    assert!(schema.validate_values(&[Value::from(21_i64)]).is_ok());
    assert!(schema.validate_values(&[Value::from(25_i64)]).is_err());
    assert!(schema.validate_values(&[Value::from(17_i64)]).is_err());
}

#[test]
fn validate_values_formula_skips_type_check() {
    let schema = make_schema("calc", vec![ColumnDef::new("result", DataType::Integer)]);
    let formula = Value::Formula("SUM(A1:A10)".to_string());
    assert!(schema.validate_values(&[formula]).is_ok());
}

#[test]
fn validate_values_multiple_columns_type_mismatch_reports_first_error() {
    let schema = make_schema(
        "mixed",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let values = vec![Value::from("bukan integer"), Value::from("Alice")];
    let err = schema.validate_values(&values).unwrap_err();
    assert!(matches!(err, DbError::TypeMismatch(_)));
    assert_eq!(
        err.to_string(),
        "Type mismatch: column 'id' expects INTEGER, got text"
    );
}

#[test]
fn validate_values_nullable_mix() {
    let mut nullable_int = ColumnDef::new("maybe_age", DataType::Integer);
    nullable_int.set_nullable(true);
    let mut not_null_text = ColumnDef::new("required_name", DataType::Text);
    not_null_text.set_nullable(false);
    let schema = make_schema("users", vec![nullable_int, not_null_text]);

    assert!(
        schema
            .validate_values(&[Value::Null, Value::from("Bob")])
            .is_ok()
    );
    assert!(
        schema
            .validate_values(&[Value::from(30_i64), Value::Null])
            .is_err()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn try_new_accepts_unique_column_names(name in "[a-zA-Z][a-zA-Z0-9_]*") {
        let col = ColumnDef::new(name.clone(), DataType::Integer);
        let schema = TableSchema::try_new("t", vec![col]);
        prop_assert!(schema.is_ok());
    }

    #[test]
    fn column_index_finds_known_name_case_insensitive(name in "alpha|beta|ALPHA|BETA") {
        let schema = TableSchema::try_new(
            "t",
            vec![
                ColumnDef::new("Alpha", DataType::Integer),
                ColumnDef::new("Beta", DataType::Text),
            ],
        )
        .unwrap();

        let idx = schema.column_index(&name);
        prop_assert!(idx.is_some());
        let expected = if name.eq_ignore_ascii_case("alpha") { 0 } else { 1 };
        prop_assert_eq!(idx, Some(expected));
    }

    #[test]
    fn try_new_rejects_duplicate_column_names_case_insensitive(
        name in "[a-zA-Z][a-zA-Z0-9_]*",
    ) {
        let col1 = ColumnDef::new(name.clone(), DataType::Integer);
        let col2 = ColumnDef::new(name.to_lowercase(), DataType::Integer);
        prop_assert!(TableSchema::try_new("t", vec![col1, col2]).is_err());
    }

    #[test]
    fn column_index_returns_same_position_for_case_insensitive_name(
        name in "[a-zA-Z]+",
    ) {
        let schema = make_schema(
            "t",
            vec![
                ColumnDef::new("Alpha", DataType::Integer),
                ColumnDef::new("Beta", DataType::Text),
            ],
        );

        let lower = name.to_lowercase();
        let upper = name.to_uppercase();
        prop_assert_eq!(schema.column_index(&lower), schema.column_index(&upper));
    }
}
