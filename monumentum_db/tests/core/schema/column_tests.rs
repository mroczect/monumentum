use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{
    CheckConstraint, Column, ColumnDef, ColumnIndex, ComparisonOp, DataType, ForeignKey,
};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use proptest::prelude::*;

fn make_table(name: &str, columns: Vec<ColumnDef>) -> Table {
    let schema = TableSchema::try_new(name, columns).expect("valid schema");
    Table::new(schema)
}

#[test]
fn data_type_as_str_boolean() {
    assert_eq!(DataType::Boolean.as_str(), "BOOLEAN");
}

#[test]
fn data_type_display_boolean() {
    assert_eq!(format!("{}", DataType::Boolean), "BOOLEAN");
}

#[test]
fn data_type_equality_all() {
    assert_eq!(DataType::Null, DataType::Null);
    assert_eq!(DataType::Integer, DataType::Integer);
    assert_eq!(DataType::Float, DataType::Float);
    assert_eq!(DataType::Text, DataType::Text);
    assert_eq!(DataType::Blob, DataType::Blob);
    assert_eq!(DataType::Boolean, DataType::Boolean);
    assert_ne!(DataType::Integer, DataType::Boolean);
}

#[test]
fn comparison_op_equality() {
    assert_eq!(ComparisonOp::Eq, ComparisonOp::Eq);
    assert_eq!(ComparisonOp::NotEq, ComparisonOp::NotEq);
    assert_eq!(ComparisonOp::Lt, ComparisonOp::Lt);
    assert_eq!(ComparisonOp::Lte, ComparisonOp::Lte);
    assert_eq!(ComparisonOp::Gt, ComparisonOp::Gt);
    assert_eq!(ComparisonOp::Gte, ComparisonOp::Gte);
    assert_ne!(ComparisonOp::Eq, ComparisonOp::NotEq);
}

#[test]
fn check_constraint_equality() {
    let c1 = CheckConstraint {
        column: "age".into(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    let c2 = c1.clone();
    assert_eq!(c1, c2);
}

#[test]
fn foreign_key_equality() {
    let f1 = ForeignKey {
        table: "users".into(),
        column: "id".into(),
    };
    let f2 = f1.clone();
    assert_eq!(f1, f2);
}

#[test]
fn column_def_new_with_str_and_string_equality() {
    let a = ColumnDef::new("id", DataType::Integer);
    let b = ColumnDef::new(String::from("id"), DataType::Integer);
    assert_eq!(a, b);
}

#[test]
fn column_def_defaults() {
    let col = ColumnDef::new("col", DataType::Text);
    assert_eq!(col.name(), "col");
    assert_eq!(col.data_type(), &DataType::Text);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(!col.is_unique());
    assert!(col.default_value().is_none());
    assert!(col.check_constraint().is_none());
    assert!(col.foreign_key().is_none());
    assert!(col.allowed_values().is_none());
}

#[test]
fn set_nullable_disables_primary_key_only_when_true() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    col.set_nullable(true);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(col.is_unique());
    col.set_nullable(false);
    assert!(!col.is_nullable());
}

#[test]
fn set_primary_key_toggles_consistency() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    assert!(col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());
    col.set_primary_key(false);
    assert!(!col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());
}

#[test]
fn set_unique_flag() {
    let mut col = ColumnDef::new("email", DataType::Text);
    col.set_unique(true);
    assert!(col.is_unique());
    col.set_unique(false);
    assert!(!col.is_unique());
}

#[test]
fn set_default_value_roundtrip() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    let v = Value::from(30_i64);
    col.set_default(Some(v.clone()));
    assert_eq!(col.default_value(), Some(&v));
    col.set_default(None);
    assert!(col.default_value().is_none());
}

#[test]
fn set_check_constraint_roundtrip() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    let check = CheckConstraint {
        column: "age".into(),
        op: ComparisonOp::Gte,
        value: Value::from(0_i64),
    };
    col.set_check(Some(check.clone()));
    assert_eq!(col.check_constraint(), Some(&check));
    col.set_check(None);
    assert!(col.check_constraint().is_none());
}

#[test]
fn set_foreign_key_roundtrip() {
    let mut col = ColumnDef::new("user_id", DataType::Integer);
    let fk = ForeignKey {
        table: "users".into(),
        column: "id".into(),
    };
    col.set_foreign_key(Some(fk.clone()));
    assert_eq!(col.foreign_key(), Some(&fk));
    col.set_foreign_key(None);
    assert!(col.foreign_key().is_none());
}

#[test]
fn set_allowed_values_roundtrip() {
    let mut col = ColumnDef::new("status", DataType::Text);
    let values = vec![Value::from("active"), Value::from("inactive")];
    col.set_allowed_values(Some(values.clone()));
    assert_eq!(col.allowed_values(), Some(&values));
    col.set_allowed_values(None);
    assert!(col.allowed_values().is_none());
}

#[test]
fn validate_formula_always_ok() {
    let col = ColumnDef::new("formula_col", DataType::Integer);
    let formula = Value::Formula("SUM(A1:A10)".into());
    assert!(col.validate_value(&formula).is_ok());
}

#[test]
fn validate_null_respect_nullable() {
    let nullable_col = ColumnDef::new("maybe", DataType::Integer);
    assert!(nullable_col.validate_value(&Value::Null).is_ok());

    let mut not_null_col = ColumnDef::new("required", DataType::Integer);
    not_null_col.set_nullable(false);
    assert!(not_null_col.validate_value(&Value::Null).is_err());
}

#[test]
fn validate_integer_column_type() {
    let col = ColumnDef::new("i", DataType::Integer);
    assert!(col.validate_value(&Value::from(42_i64)).is_ok());
    assert!(col.validate_value(&Value::from("not int")).is_err());
    assert!(col.validate_value(&Value::try_from(2.5).unwrap()).is_err());
    assert!(col.validate_value(&Value::Boolean(true)).is_err());
}

#[test]
fn validate_float_column_type() {
    let col = ColumnDef::new("f", DataType::Float);
    let float_val = Value::try_from(2.5).unwrap();
    assert!(col.validate_value(&float_val).is_ok());
    assert!(col.validate_value(&Value::from(42_i64)).is_err());
    assert!(col.validate_value(&Value::from("2.5")).is_err());
    assert!(col.validate_value(&Value::Boolean(false)).is_err());
}

#[test]
fn validate_text_column_type() {
    let col = ColumnDef::new("t", DataType::Text);
    assert!(col.validate_value(&Value::from("hello")).is_ok());
    assert!(col.validate_value(&Value::from(42_i64)).is_err());
    assert!(col.validate_value(&Value::try_from(2.71).unwrap()).is_err());
    assert!(col.validate_value(&Value::Boolean(true)).is_err());
}

#[test]
fn validate_blob_column_type() {
    let col = ColumnDef::new("b", DataType::Blob);
    assert!(col.validate_value(&Value::from(vec![1_u8, 2, 3])).is_ok());
    assert!(col.validate_value(&Value::from("bytes")).is_err());
    assert!(col.validate_value(&Value::from(7_i64)).is_err());
    assert!(col.validate_value(&Value::Boolean(false)).is_err());
}

#[test]
fn validate_boolean_column_type() {
    let col = ColumnDef::new("flag", DataType::Boolean);
    assert!(col.validate_value(&Value::Boolean(true)).is_ok());
    assert!(col.validate_value(&Value::Boolean(false)).is_ok());
    assert!(col.validate_value(&Value::from(1_i64)).is_err());
    assert!(col.validate_value(&Value::from("true")).is_err());
}

#[test]
fn validate_check_constraint_integer_ops() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    let check = CheckConstraint {
        column: "age".into(),
        op: ComparisonOp::Gte,
        value: Value::from(18_i64),
    };
    col.set_check(Some(check));

    assert!(col.validate_value(&Value::from(18_i64)).is_ok());
    assert!(col.validate_value(&Value::from(30_i64)).is_ok());
    assert!(col.validate_value(&Value::from(17_i64)).is_err());
}

#[test]
fn validate_check_constraint_float_ops() {
    let mut col = ColumnDef::new("price", DataType::Float);
    let check = CheckConstraint {
        column: "price".into(),
        op: ComparisonOp::Lt,
        value: Value::try_from(100.0).unwrap(),
    };
    col.set_check(Some(check));

    assert!(col.validate_value(&Value::try_from(99.99).unwrap()).is_ok());
    assert!(
        col.validate_value(&Value::try_from(100.0).unwrap())
            .is_err()
    );
}

#[test]
fn validate_check_constraint_text_ops() {
    let mut col = ColumnDef::new("name", DataType::Text);
    let check = CheckConstraint {
        column: "name".into(),
        op: ComparisonOp::NotEq,
        value: Value::from("admin"),
    };
    col.set_check(Some(check));

    assert!(col.validate_value(&Value::from("user")).is_ok());
    assert!(col.validate_value(&Value::from("admin")).is_err());
}

#[test]
fn validate_check_constraint_blob_eq() {
    let mut col = ColumnDef::new("data", DataType::Blob);
    let check = CheckConstraint {
        column: "data".into(),
        op: ComparisonOp::Eq,
        value: Value::from(vec![1_u8, 2, 3]),
    };
    col.set_check(Some(check));

    assert!(col.validate_value(&Value::from(vec![1_u8, 2, 3])).is_ok());
    assert!(col.validate_value(&Value::from(vec![1_u8, 2, 4])).is_err());
}

#[test]
fn validate_allowed_values() {
    let mut col = ColumnDef::new("status", DataType::Text);
    col.set_allowed_values(Some(vec![Value::from("active"), Value::from("inactive")]));

    assert!(col.validate_value(&Value::from("active")).is_ok());
    assert!(col.validate_value(&Value::from("inactive")).is_ok());
    assert!(col.validate_value(&Value::from("pending")).is_err());
}

#[test]
fn validate_combined_check_and_allowed() {
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

    assert!(col.validate_value(&Value::from(18_i64)).is_ok());
    assert!(col.validate_value(&Value::from(21_i64)).is_ok());
    assert!(col.validate_value(&Value::from(25_i64)).is_err());
    assert!(col.validate_value(&Value::from(17_i64)).is_err());
}

#[test]
fn column_trait_methods() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    let col_ref: &dyn Column = &col;
    assert_eq!(col_ref.name(), "id");
    assert_eq!(col_ref.data_type(), &DataType::Integer);
    assert!(!col_ref.is_nullable());
    assert!(col_ref.is_primary_key());
    assert!(col_ref.is_unique());
}

#[test]
fn column_index_on_row_with_usize() {
    let row = Row::new(vec![Value::from(1_i64), Value::from("a")]);
    assert_eq!(usize::index(&0, &row), Ok(0));
    assert_eq!(usize::index(&1, &row), Ok(1));
    assert!(usize::index(&2, &row).is_err());
}

#[test]
fn column_index_on_schema_with_usize_and_str() {
    let schema = TableSchema::try_new(
        "test",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )
    .unwrap();

    assert_eq!(usize::index(&0, &schema), Ok(0));
    assert_eq!(usize::index(&1, &schema), Ok(1));
    assert!(usize::index(&2, &schema).is_err());

    assert_eq!(<&str as ColumnIndex<_>>::index(&"id", &schema), Ok(0));
    assert_eq!(<&str as ColumnIndex<_>>::index(&"NAME", &schema), Ok(1));
    assert!(<&str as ColumnIndex<_>>::index(&"missing", &schema).is_err());
}

#[test]
fn column_index_on_table_with_str() {
    let table = make_table(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("value", DataType::Text),
        ],
    );

    assert_eq!(<&str as ColumnIndex<_>>::index(&"id", &table), Ok(0));
    assert_eq!(<&str as ColumnIndex<_>>::index(&"VALUE", &table), Ok(1));
    assert!(<&str as ColumnIndex<_>>::index(&"nope", &table).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn validate_value_accepts_matching_type(
        int_val in any::<i64>(),
    ) {
        let col = ColumnDef::new("int_col", DataType::Integer);
        prop_assert!(col.validate_value(&Value::from(int_val)).is_ok());
    }

    #[test]
    fn validate_value_rejects_wrong_type_for_integer(
        text in ".*",
    ) {
        let col = ColumnDef::new("int_col", DataType::Integer);
        prop_assert!(col.validate_value(&Value::from(text)).is_err());
    }

    #[test]
    fn validate_float_accepts_finite(
        finite_float in any::<f64>().prop_filter("must be finite", |f| f.is_finite()),
    ) {
        let col = ColumnDef::new("float_col", DataType::Float);
        let val = Value::try_from(finite_float).unwrap();
        prop_assert!(col.validate_value(&val).is_ok());
    }

    #[test]
    fn validate_boolean_accepts_any_bool(
        b in any::<bool>(),
    ) {
        let col = ColumnDef::new("bool_col", DataType::Boolean);
        prop_assert!(col.validate_value(&Value::Boolean(b)).is_ok());
    }

    #[test]
    fn validate_text_rejects_non_text(
        int_val in any::<i64>(),
    ) {
        let col = ColumnDef::new("text_col", DataType::Text);
        prop_assert!(col.validate_value(&Value::from(int_val)).is_err());
    }

    #[test]
    fn validate_allowed_values_accepts_any_listed_value(
        allowed in prop::collection::vec(any::<i64>(), 1..10),
        index in 0_usize..10,
    ) {
        let allowed_values: Vec<Value> = allowed.iter().map(|&v| Value::from(v)).collect();
        let mut col = ColumnDef::new("num", DataType::Integer);
        col.set_allowed_values(Some(allowed_values.clone()));

        if index < allowed_values.len() {
            let chosen = allowed_values[index].clone();
            prop_assert!(col.validate_value(&chosen).is_ok());
        }
    }

    #[test]
    fn column_index_usize_bounds(
        len in 0_usize..100,
        idx in 0_usize..150,
    ) {
        let row = Row::new(vec![Value::Null; len]);
        let result = usize::index(&idx, &row);
        if idx < len {
            prop_assert_eq!(result, Ok(idx));
        } else {
            prop_assert!(result.is_err());
        }
    }
}

#[test]
fn data_type_as_str_stable() {
    assert_eq!(DataType::Null.as_str(), "NULL");
    assert_eq!(DataType::Integer.as_str(), "INTEGER");
    assert_eq!(DataType::Float.as_str(), "FLOAT");
    assert_eq!(DataType::Text.as_str(), "TEXT");
    assert_eq!(DataType::Blob.as_str(), "BLOB");
    assert_eq!(DataType::Boolean.as_str(), "BOOLEAN");
}
