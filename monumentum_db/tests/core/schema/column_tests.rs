use monumentum_db::core::schema::column::{
    CheckConstraint, ColumnDef, ComparisonOp, DataType, ForeignKey,
};
use monumentum_db::core::value::Value;

#[test]
fn data_type_as_str_returns_expected() {
    assert_eq!(DataType::Null.as_str(), "NULL");
    assert_eq!(DataType::Integer.as_str(), "INTEGER");
    assert_eq!(DataType::Float.as_str(), "FLOAT");
    assert_eq!(DataType::Text.as_str(), "TEXT");
    assert_eq!(DataType::Blob.as_str(), "BLOB");
}

#[test]
fn data_type_display_formats_correctly() {
    assert_eq!(format!("{}", DataType::Null), "NULL");
    assert_eq!(format!("{}", DataType::Integer), "INTEGER");
    assert_eq!(format!("{}", DataType::Float), "FLOAT");
    assert_eq!(format!("{}", DataType::Text), "TEXT");
    assert_eq!(format!("{}", DataType::Blob), "BLOB");
}

#[test]
fn data_type_equality() {
    assert_eq!(DataType::Integer, DataType::Integer);
    assert_ne!(DataType::Integer, DataType::Text);
}

#[test]
fn column_def_new_with_str_and_string() {
    let col1 = ColumnDef::new("id", DataType::Integer);
    let col2 = ColumnDef::new(String::from("id"), DataType::Integer);
    assert_eq!(col1, col2);
}

#[test]
fn column_def_new_defaults() {
    let col = ColumnDef::new("id", DataType::Integer);
    assert_eq!(col.name(), "id");
    assert_eq!(col.data_type(), &DataType::Integer);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(!col.is_unique());
    assert!(col.default_value().is_none());
    assert!(col.check_constraint().is_none());
    assert!(col.foreign_key().is_none());
}

#[test]
fn set_nullable_true_sets_nullable_and_disables_primary_key() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    assert!(!col.is_nullable());
    assert!(col.is_primary_key());
    assert!(col.is_unique());

    col.set_nullable(true);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(col.is_unique());
}

#[test]
fn set_nullable_false_only_sets_nullable() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_nullable(false);
    assert!(!col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(!col.is_unique());
}

#[test]
fn set_primary_key_true_sets_flags_and_forces_consistency() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    assert!(col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());
}

#[test]
fn set_primary_key_false_does_not_reset_unique_or_nullable() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    col.set_primary_key(false);
    assert!(!col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());
}

#[test]
fn set_unique_sets_unique_flag() {
    let mut col = ColumnDef::new("email", DataType::Text);
    assert!(!col.is_unique());
    col.set_unique(true);
    assert!(col.is_unique());
    col.set_unique(false);
    assert!(!col.is_unique());
}

#[test]
fn set_default_sets_default_value() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    assert!(col.default_value().is_none());
    let default_val = Value::from(30_i64);
    col.set_default(Some(default_val.clone()));
    assert_eq!(col.default_value(), Some(&default_val));
    col.set_default(None);
    assert!(col.default_value().is_none());
}

#[test]
fn set_check_sets_check_constraint() {
    let mut col = ColumnDef::new("age", DataType::Integer);
    assert!(col.check_constraint().is_none());
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gte,
        value: Value::from(0_i64),
    };
    col.set_check(Some(check.clone()));
    assert_eq!(col.check_constraint(), Some(&check));
    col.set_check(None);
    assert!(col.check_constraint().is_none());
}

#[test]
fn set_foreign_key_sets_foreign_key() {
    let mut col = ColumnDef::new("user_id", DataType::Integer);
    assert!(col.foreign_key().is_none());
    let fk = ForeignKey {
        table: "users".to_string(),
        column: "id".to_string(),
    };
    col.set_foreign_key(Some(fk.clone()));
    assert_eq!(col.foreign_key(), Some(&fk));
    col.set_foreign_key(None);
    assert!(col.foreign_key().is_none());
}

#[test]
fn check_constraint_equality() {
    let check1 = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    let check2 = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    assert_eq!(check1, check2);
}

#[test]
fn foreign_key_equality() {
    let fk1 = ForeignKey {
        table: "users".to_string(),
        column: "id".to_string(),
    };
    let fk2 = ForeignKey {
        table: "users".to_string(),
        column: "id".to_string(),
    };
    assert_eq!(fk1, fk2);
}

#[test]
fn column_def_equality_considers_constraints() {
    let mut col1 = ColumnDef::new("id", DataType::Integer);
    let mut col2 = ColumnDef::new("id", DataType::Integer);
    assert_eq!(col1, col2);
    col1.set_primary_key(true);
    assert_ne!(col1, col2);
    col2.set_primary_key(true);
    assert_eq!(col1, col2);
}
