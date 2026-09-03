use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{CheckConstraint, ColumnDef, ComparisonOp, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use proptest::prelude::*;

fn create_column(
    name: &str,
    data_type: DataType,
    unique: bool,
    nullable: bool,
    primary_key: bool,
) -> ColumnDef {
    let mut col = ColumnDef::new(name, data_type);
    col.set_unique(unique);
    col.set_nullable(nullable);
    if primary_key {
        col.set_primary_key(true);
    }
    col
}

fn create_table(columns: Vec<ColumnDef>) -> Table {
    let schema = TableSchema::try_new("test_table", columns).expect("valid schema");
    Table::new(schema)
}

fn int_value(v: i64) -> Value {
    Value::from(v)
}

#[test]
fn rename_schema_updates_name_and_preserves_data() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;
    table.insert(Row::new(vec![int_value(2)]))?;

    table.rename_schema("renamed_table")?;

    assert_eq!(table.schema().name(), "renamed_table");
    assert_eq!(table.len(), 2);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&int_value(1)));
    } else {
        unreachable!();
    }
    if let Some(row) = table.get(1) {
        assert_eq!(row.get(0), Some(&int_value(2)));
    } else {
        unreachable!();
    }
    Ok(())
}

#[test]
fn rename_schema_with_empty_name_returns_error() {
    let col = create_column("id", DataType::Integer, false, false, false);
    let mut table = create_table(vec![col]);
    let result = table.rename_schema("");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("table name cannot be empty"));
    }
}

#[test]
fn set_cell_updates_value_and_index() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;
    table.insert(Row::new(vec![int_value(2)]))?;

    table.set_cell(0, 0, int_value(3))?;

    assert!(table.lookup_by_unique(0, &int_value(1)).is_none());
    if let Some(row) = table.lookup_by_unique(0, &int_value(3)) {
        assert_eq!(row.get(0), Some(&int_value(3)));
    } else {
        unreachable!();
    }
    Ok(())
}

#[test]
fn set_cell_duplicate_unique_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;
    table.insert(Row::new(vec![int_value(2)]))?;

    let result = table.set_cell(0, 0, int_value(2));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: duplicate value for column 'id'"
        );
    }
    Ok(())
}

#[test]
fn set_cell_out_of_bounds_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;

    let result = table.set_cell(1, 0, int_value(5));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: index out of bounds");
    }

    let result = table.set_cell(0, 1, int_value(5));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: index out of bounds");
    }
    Ok(())
}

#[test]
fn set_cell_read_only_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;
    table.set_read_only(true);

    let result = table.set_cell(0, 0, int_value(2));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: table is read-only");
    }
    Ok(())
}

#[test]
fn set_cell_type_mismatch_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(1)]))?;

    let result = table.set_cell(0, 0, Value::from("not int"));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: column 'id' expects INTEGER, got text"
        );
    }
    Ok(())
}

#[test]
fn set_cell_violates_check_constraint_returns_error() -> Result<(), DbError> {
    let mut col = create_column("age", DataType::Integer, false, false, false);
    col.set_check(Some(CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: int_value(0),
    }));
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![int_value(10)]))?;

    let result = table.set_cell(0, 0, int_value(-5));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("check constraint failed"));
    }
    Ok(())
}

#[test]
fn set_column_allowed_values_updates_and_validates() -> Result<(), DbError> {
    let col = create_column("status", DataType::Text, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![Value::from("active")]))?;

    table.set_column_allowed_values(
        0,
        Some(vec![Value::from("active"), Value::from("inactive")]),
    )?;

    table.set_cell(0, 0, Value::from("inactive"))?;

    let result = table.set_cell(0, 0, Value::from("pending"));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("not in the allowed list"));
    }
    Ok(())
}

#[test]
fn set_column_allowed_values_out_of_bounds_returns_error() -> Result<(), DbError> {
    let col = create_column("status", DataType::Text, false, false, false);
    let mut table = create_table(vec![col]);

    let result = table.set_column_allowed_values(1, Some(vec![Value::from("a")]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: column index out of bounds"
        );
    }
    Ok(())
}

#[test]
fn set_column_allowed_values_rejects_if_existing_data_invalid() -> Result<(), DbError> {
    let col = create_column("status", DataType::Text, false, false, false);
    let mut table = create_table(vec![col]);
    table.insert(Row::new(vec![Value::from("active")]))?;

    let result = table.set_column_allowed_values(0, Some(vec![Value::from("inactive")]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("not in the allowed list"));
    }
    Ok(())
}

#[test]
fn get_column_by_name_returns_reference() {
    let col = create_column("id", DataType::Integer, false, false, false);
    let table = create_table(vec![col]);

    let column = table.get_column_by_name("id");
    assert!(column.is_some());
    if let Some(c) = column {
        assert_eq!(c.name(), "id");
        assert_eq!(c.data_type(), &DataType::Integer);
    }
}

#[test]
fn get_column_by_name_missing_returns_none() {
    let col = create_column("id", DataType::Integer, false, false, false);
    let table = create_table(vec![col]);
    assert!(table.get_column_by_name("missing").is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn insert_unique_values_always_succeeds(
        values in prop::collection::vec(any::<i64>(), 0..50),
    ) {
        let col = create_column("id", DataType::Integer, true, false, false);
        let mut table = create_table(vec![col]);
        let mut unique_set = std::collections::HashSet::new();
        for v in values {
            let value = int_value(v);
            if unique_set.insert(v) {
                prop_assert!(table.insert(Row::new(vec![value])).is_ok());
            } else {
                prop_assert!(table.insert(Row::new(vec![value])).is_err());
            }
        }
        prop_assert_eq!(table.len(), unique_set.len());
    }

    #[test]
    fn insert_null_multiple_times_on_unique_column(
        count in 0..20,
    ) {
        let mut col = create_column("id", DataType::Integer, true, true, false);
        col.set_nullable(true);
        let mut table = create_table(vec![col]);
        for _ in 0..count {
            prop_assert!(table.insert(Row::new(vec![Value::Null])).is_ok());
        }
        prop_assert_eq!(table.len(), count as usize);
    }

    #[test]
    fn replace_rows_rebuilds_index_consistently(
        initial in prop::collection::vec(any::<i64>(), 0..20),
        new_values in prop::collection::vec(any::<i64>(), 0..20),
    ) {
        let col = create_column("id", DataType::Integer, true, false, false);
        let mut table = create_table(vec![col]);

        let mut initial_unique = std::collections::HashSet::new();
        for v in initial {
            if initial_unique.insert(v) {
                prop_assert!(table.insert(Row::new(vec![int_value(v)])).is_ok());
            }
        }

        let mut new_rows = Vec::new();
        let mut new_unique = std::collections::HashSet::new();
        for v in new_values {
            if new_unique.insert(v) {
                new_rows.push(Row::new(vec![int_value(v)]));
            }
        }

        prop_assert!(table.replace_rows(new_rows).is_ok());

        for v in &new_unique {
            prop_assert!(table.lookup_by_unique(0, &int_value(*v)).is_some());
        }
        for v in initial_unique.difference(&new_unique) {
            prop_assert!(table.lookup_by_unique(0, &int_value(*v)).is_none());
        }
    }

    #[test]
    fn set_cell_and_lookup_consistent(
        values in prop::collection::vec(any::<i64>(), 1..20),
    ) {
        let col = create_column("id", DataType::Integer, true, false, false);
        let mut table = create_table(vec![col]);

        let mut unique_vals = Vec::new();
        for v in values {
            let value = int_value(v);
            if !unique_vals.contains(&v) {
                prop_assert!(table.insert(Row::new(vec![value.clone()])).is_ok());
                unique_vals.push(v);
            }
        }

        for (i, &old_val) in unique_vals.iter().enumerate().take(unique_vals.len().min(10)) {
            let new_val = old_val + 1000;
            prop_assert!(table.set_cell(i, 0, int_value(new_val)).is_ok());
            prop_assert!(table.lookup_by_unique(0, &int_value(new_val)).is_some());
            prop_assert!(table.lookup_by_unique(0, &int_value(old_val)).is_none());
        }
    }
}
