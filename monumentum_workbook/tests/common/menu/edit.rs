use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError};

fn workbook_with_data() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "test",
        vec![
            ColumnDef::new("a", DataType::Integer),
            ColumnDef::new("b", DataType::Integer),
        ],
    )?;
    catalog.create_table(schema)?;

    {
        let table = catalog
            .get_table_mut("test")
            .ok_or("table 'test' not found")?;
        table.insert(Row::new(vec![Value::from(1_i64), Value::from(2_i64)]))?;
        table.insert(Row::new(vec![Value::from(3_i64), Value::from(4_i64)]))?;
    }

    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn empty_workbook() -> Workbook<InMemoryStorage> {
    Workbook::<InMemoryStorage>::new_in_memory()
}

#[test]
fn sheet_existing_returns_table() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_data()?;
    let table = wb.sheet("test")?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn sheet_missing_returns_db_error() {
    let wb = empty_workbook();
    let result = wb.sheet("nonexistent");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn sheet_mut_existing_returns_mutable_table() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_data()?;
    {
        let table = wb.sheet_mut("test")?;
        table.insert(Row::new(vec![Value::from(5_i64), Value::from(6_i64)]))?;
    }
    assert_eq!(wb.row_count("test")?, 3);
    Ok(())
}

#[test]
fn sheet_mut_missing_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.sheet_mut("ghost");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn row_count_returns_number_of_rows() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_data()?;
    assert_eq!(wb.row_count("test")?, 2);
    assert!(wb.row_count("missing").is_err());
    Ok(())
}

#[test]
fn column_count_returns_number_of_columns() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_data()?;
    assert_eq!(wb.column_count("test")?, 2);
    assert!(wb.column_count("missing").is_err());
    Ok(())
}

#[test]
fn get_cell_returns_correct_value() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_data()?;
    assert_eq!(wb.get_cell("test", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("test", 1, 1), Some(&Value::from(4_i64)));
    Ok(())
}

#[test]
fn get_cell_missing_sheet_returns_none() {
    let wb = empty_workbook();
    assert_eq!(wb.get_cell("nope", 0, 0), None);
}

#[test]
fn get_cell_invalid_indices_returns_none() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_data()?;
    assert_eq!(wb.get_cell("test", 5, 0), None);
    assert_eq!(wb.get_cell("test", 0, 5), None);
    Ok(())
}

#[test]
fn set_cell_updates_value() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_data()?;
    wb.set_cell("test", 0, 0, Value::from(99_i64))?;

    assert_eq!(wb.get_cell("test", 0, 0), Some(&Value::from(99_i64)));
    assert_eq!(wb.get_cell("test", 0, 1), Some(&Value::from(2_i64)));
    Ok(())
}

#[test]
fn set_cell_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.set_cell("ghost", 0, 0, Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn set_cell_invalid_row_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_data()?;
    let result = wb.set_cell("test", 10, 0, Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn set_cell_invalid_column_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_data()?;
    let result = wb.set_cell("test", 0, 10, Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn find_in_sheet_returns_all_matches() -> Result<(), Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "data",
        vec![
            ColumnDef::new("x", DataType::Integer),
            ColumnDef::new("y", DataType::Integer),
        ],
    )?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("data")
            .ok_or("table 'data' not found")?;
        table.insert(Row::new(vec![Value::from(7_i64), Value::from(8_i64)]))?;
        table.insert(Row::new(vec![Value::from(7_i64), Value::from(9_i64)]))?;
        table.insert(Row::new(vec![Value::from(10_i64), Value::from(7_i64)]))?;
    }
    let wb = Workbook::<InMemoryStorage>::load_in_memory(catalog);

    let matches = wb.find_in_sheet("data", &Value::from(7_i64))?;
    assert_eq!(matches, vec![(0, 0), (1, 0), (2, 1)]);

    let empty = wb.find_in_sheet("data", &Value::from(42_i64))?;
    assert!(empty.is_empty());

    Ok(())
}

#[test]
fn find_in_sheet_missing_sheet_returns_db_error() {
    let wb = empty_workbook();
    let result = wb.find_in_sheet("ghost", &Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn replace_in_sheet_replaces_values_and_returns_count() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_data()?;
    let count = wb.replace_in_sheet("test", &Value::from(1_i64), &Value::from(100_i64))?;
    assert_eq!(count, 1);
    assert_eq!(wb.get_cell("test", 0, 0), Some(&Value::from(100_i64)));
    assert_eq!(wb.get_cell("test", 0, 1), Some(&Value::from(2_i64)));

    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("multi", vec![ColumnDef::new("v", DataType::Integer)])?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("multi")
            .ok_or("table 'multi' not found")?;
        table.insert(Row::new(vec![Value::from(5_i64)]))?;
        table.insert(Row::new(vec![Value::from(5_i64)]))?;
        table.insert(Row::new(vec![Value::from(6_i64)]))?;
    }
    let mut wb2 = Workbook::<InMemoryStorage>::load_in_memory(catalog);
    let count = wb2.replace_in_sheet("multi", &Value::from(5_i64), &Value::from(9_i64))?;
    assert_eq!(count, 2);
    assert_eq!(wb2.get_cell("multi", 0, 0), Some(&Value::from(9_i64)));
    assert_eq!(wb2.get_cell("multi", 1, 0), Some(&Value::from(9_i64)));
    assert_eq!(wb2.get_cell("multi", 2, 0), Some(&Value::from(6_i64)));

    Ok(())
}

#[test]
fn replace_in_sheet_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.replace_in_sheet("ghost", &Value::from(1_i64), &Value::from(2_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}
