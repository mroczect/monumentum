use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError};
use pretty_assertions::assert_eq;

fn workbook_with_table() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "table",
        vec![
            ColumnDef::new("a", DataType::Integer),
            ColumnDef::new("b", DataType::Integer),
        ],
    )?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("table")
            .ok_or("table 'table' not found")?;
        table.insert(Row::new(vec![Value::from(1_i64), Value::from(10_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64), Value::from(20_i64)]))?;
        table.insert(Row::new(vec![Value::from(3_i64), Value::from(30_i64)]))?;
    }
    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn empty_workbook() -> Workbook<InMemoryStorage> {
    Workbook::<InMemoryStorage>::new_in_memory()
}

#[test]
fn insert_row_at_beginning() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.insert_row_at("table", 0, vec![Value::from(99_i64), Value::from(990_i64)])?;

    assert_eq!(wb.row_count("table")?, 4);
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(99_i64)));
    assert_eq!(wb.get_cell("table", 0, 1), Some(&Value::from(990_i64)));
    assert_eq!(wb.get_cell("table", 1, 0), Some(&Value::from(1_i64)));
    Ok(())
}

#[test]
fn insert_row_at_middle() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.insert_row_at("table", 1, vec![Value::from(50_i64), Value::from(500_i64)])?;

    assert_eq!(wb.row_count("table")?, 4);
    assert_eq!(wb.get_cell("table", 1, 0), Some(&Value::from(50_i64)));
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("table", 2, 0), Some(&Value::from(2_i64)));
    Ok(())
}

#[test]
fn insert_row_at_end() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.insert_row_at(
        "table",
        3,
        vec![Value::from(100_i64), Value::from(1000_i64)],
    )?;

    assert_eq!(wb.row_count("table")?, 4);
    assert_eq!(wb.get_cell("table", 3, 0), Some(&Value::from(100_i64)));
    Ok(())
}

#[test]
fn insert_row_at_index_greater_than_len_appends_at_end() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.insert_row_at("table", 10, vec![Value::from(77_i64), Value::from(770_i64)])?;

    assert_eq!(wb.row_count("table")?, 4);
    assert_eq!(wb.get_cell("table", 3, 0), Some(&Value::from(77_i64)));
    Ok(())
}

#[test]
fn insert_row_at_wrong_number_of_values_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let result = wb.insert_row_at("table", 0, vec![Value::from(1_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn insert_row_at_type_mismatch_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let result = wb.insert_row_at("table", 0, vec![Value::from("wrong"), Value::from("types")]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn insert_row_at_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.insert_row_at("ghost", 0, vec![Value::from(1_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn insert_column_at_beginning_with_default() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut new_col = ColumnDef::new("new_col", DataType::Integer);
    new_col.set_default(Some(Value::from(0_i64)));
    wb.insert_column("table", 0, &new_col)?;

    assert_eq!(wb.column_count("table")?, 3);
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(0_i64)));
    assert_eq!(wb.get_cell("table", 1, 0), Some(&Value::from(0_i64)));
    assert_eq!(wb.get_cell("table", 0, 1), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("table", 0, 2), Some(&Value::from(10_i64)));
    Ok(())
}

#[test]
fn insert_column_at_end_no_default_fills_null() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let new_col = ColumnDef::new("new_col", DataType::Integer);
    wb.insert_column("table", 2, &new_col)?;

    assert_eq!(wb.column_count("table")?, 3);
    assert_eq!(wb.get_cell("table", 0, 2), Some(&Value::Null));
    assert_eq!(wb.get_cell("table", 1, 2), Some(&Value::Null));
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("table", 0, 1), Some(&Value::from(10_i64)));
    Ok(())
}

#[test]
fn insert_column_invalid_index_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let new_col = ColumnDef::new("x", DataType::Integer);
    let result = wb.insert_column("table", 3, &new_col);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn insert_column_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let new_col = ColumnDef::new("x", DataType::Integer);
    let result = wb.insert_column("ghost", 0, &new_col);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn delete_column_first() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.delete_column("table", 0)?;

    assert_eq!(wb.column_count("table")?, 1);
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(10_i64)));
    assert_eq!(wb.get_cell("table", 1, 0), Some(&Value::from(20_i64)));
    assert_eq!(wb.get_cell("table", 2, 0), Some(&Value::from(30_i64)));
    Ok(())
}

#[test]
fn delete_column_last() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    wb.delete_column("table", 1)?;

    assert_eq!(wb.column_count("table")?, 1);
    assert_eq!(wb.get_cell("table", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("table", 1, 0), Some(&Value::from(2_i64)));
    assert_eq!(wb.get_cell("table", 2, 0), Some(&Value::from(3_i64)));
    Ok(())
}

#[test]
fn delete_column_middle() -> Result<(), Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "t3",
        vec![
            ColumnDef::new("a", DataType::Integer),
            ColumnDef::new("b", DataType::Integer),
            ColumnDef::new("c", DataType::Integer),
        ],
    )?;
    catalog.create_table(schema)?;
    {
        let table = catalog.get_table_mut("t3").ok_or("table 't3' not found")?;
        table.insert(Row::new(vec![
            Value::from(1_i64),
            Value::from(2_i64),
            Value::from(3_i64),
        ]))?;
    }
    let mut wb = Workbook::<InMemoryStorage>::load_in_memory(catalog);

    wb.delete_column("t3", 1)?;

    assert_eq!(wb.column_count("t3")?, 2);
    assert_eq!(wb.get_cell("t3", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("t3", 0, 1), Some(&Value::from(3_i64)));
    Ok(())
}

#[test]
fn delete_column_invalid_index_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let result = wb.delete_column("table", 2);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn delete_column_last_remaining_returns_error() -> Result<(), Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("single", vec![ColumnDef::new("x", DataType::Integer)])?;
    catalog.create_table(schema)?;
    let mut wb = Workbook::<InMemoryStorage>::load_in_memory(catalog);

    let result = wb.delete_column("single", 0);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn delete_column_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.delete_column("ghost", 0);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}
