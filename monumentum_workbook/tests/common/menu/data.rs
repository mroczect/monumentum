use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError};
use pretty_assertions::assert_eq;

fn workbook_with_numbers() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("numbers", vec![ColumnDef::new("val", DataType::Integer)])?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("numbers")
            .ok_or("table 'numbers' not found")?;
        table.insert(Row::new(vec![Value::from(5_i64)]))?;
        table.insert(Row::new(vec![Value::from(3_i64)]))?;
        table.insert(Row::new(vec![Value::from(8_i64)]))?;
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
    }
    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn workbook_with_texts() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("texts", vec![ColumnDef::new("word", DataType::Text)])?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("texts")
            .ok_or("table 'texts' not found")?;
        table.insert(Row::new(vec![Value::from("banana")]))?;
        table.insert(Row::new(vec![Value::from("apple")]))?;
        table.insert(Row::new(vec![Value::from("cherry")]))?;
    }
    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn workbook_with_duplicates() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("dup", vec![ColumnDef::new("num", DataType::Integer)])?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("dup")
            .ok_or("table 'dup' not found")?;
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
        table.insert(Row::new(vec![Value::from(3_i64)]))?;
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.insert(Row::new(vec![Value::from(3_i64)]))?;
    }
    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn empty_workbook() -> Workbook<InMemoryStorage> {
    Workbook::<InMemoryStorage>::new_in_memory()
}

#[test]
fn sort_sheet_ascending_integer() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_numbers()?;
    wb.sort_sheet("numbers", 0, true)?;

    assert_eq!(wb.get_cell("numbers", 0, 0), Some(&Value::from(1_i64)));
    assert_eq!(wb.get_cell("numbers", 1, 0), Some(&Value::from(3_i64)));
    assert_eq!(wb.get_cell("numbers", 2, 0), Some(&Value::from(5_i64)));
    assert_eq!(wb.get_cell("numbers", 3, 0), Some(&Value::from(8_i64)));
    Ok(())
}

#[test]
fn sort_sheet_descending_integer() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_numbers()?;
    wb.sort_sheet("numbers", 0, false)?;

    assert_eq!(wb.get_cell("numbers", 0, 0), Some(&Value::from(8_i64)));
    assert_eq!(wb.get_cell("numbers", 1, 0), Some(&Value::from(5_i64)));
    assert_eq!(wb.get_cell("numbers", 2, 0), Some(&Value::from(3_i64)));
    assert_eq!(wb.get_cell("numbers", 3, 0), Some(&Value::from(1_i64)));
    Ok(())
}

#[test]
fn sort_sheet_ascending_text() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_texts()?;
    wb.sort_sheet("texts", 0, true)?;

    assert_eq!(wb.get_cell("texts", 0, 0), Some(&Value::from("apple")));
    assert_eq!(wb.get_cell("texts", 1, 0), Some(&Value::from("banana")));
    assert_eq!(wb.get_cell("texts", 2, 0), Some(&Value::from("cherry")));
    Ok(())
}

#[test]
fn sort_sheet_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.sort_sheet("ghost", 0, true);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn sort_sheet_invalid_column_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_numbers()?;
    let result = wb.sort_sheet("numbers", 5, true);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn filter_sheet_returns_matching_rows() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_numbers()?;
    let result = wb.filter_sheet("numbers", 0, &Value::from(3_i64))?;

    assert_eq!(result.len(), 1);
    assert_eq!(
        result.first().and_then(|row| row.get(0)),
        Some(&Value::from(3_i64))
    );
    Ok(())
}

#[test]
fn filter_sheet_no_match_returns_empty() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_numbers()?;
    let result = wb.filter_sheet("numbers", 0, &Value::from(42_i64))?;
    assert!(result.is_empty());
    Ok(())
}

#[test]
fn filter_sheet_missing_sheet_returns_db_error() {
    let wb = empty_workbook();
    let result = wb.filter_sheet("ghost", 0, &Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn filter_sheet_invalid_column_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_numbers()?;
    let result = wb.filter_sheet("numbers", 10, &Value::from(1_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn distinct_values_integer_sorted_unique() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_duplicates()?;
    let distinct = wb.distinct_values("dup", 0)?;

    let expected = vec![Value::from(1_i64), Value::from(2_i64), Value::from(3_i64)];
    assert_eq!(distinct, expected);
    Ok(())
}

#[test]
fn distinct_values_text_sorted_unique() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_texts()?;
    let distinct = wb.distinct_values("texts", 0)?;

    let expected = vec![
        Value::from("apple"),
        Value::from("banana"),
        Value::from("cherry"),
    ];
    assert_eq!(distinct, expected);
    Ok(())
}

#[test]
fn distinct_values_missing_sheet_returns_db_error() {
    let wb = empty_workbook();
    let result = wb.distinct_values("ghost", 0);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn distinct_values_invalid_column_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_numbers()?;
    let result = wb.distinct_values("numbers", 3);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}
