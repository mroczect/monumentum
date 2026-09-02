use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError};
use pretty_assertions::assert_eq;

fn workbook_with_sheets() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();

    let schema_alpha = TableSchema::try_new("alpha", vec![ColumnDef::new("x", DataType::Integer)])?;
    catalog.create_table(schema_alpha)?;
    {
        let table = catalog
            .get_table_mut("alpha")
            .ok_or("table 'alpha' not found")?;
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
    }

    let schema_beta = TableSchema::try_new("beta", vec![ColumnDef::new("y", DataType::Text)])?;
    catalog.create_table(schema_beta)?;
    {
        let table = catalog
            .get_table_mut("beta")
            .ok_or("table 'beta' not found")?;
        table.insert(Row::new(vec![Value::from("hello")]))?;
    }

    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

fn empty_workbook() -> Workbook<InMemoryStorage> {
    Workbook::<InMemoryStorage>::new_in_memory()
}

#[test]
fn sheet_names_returns_all_sheet_names() -> Result<(), Box<dyn Error>> {
    let wb = workbook_with_sheets()?;
    let mut names = wb.sheet_names();
    names.sort();
    assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    Ok(())
}

#[test]
fn sheet_names_empty_workbook_returns_empty_vec() {
    let wb = empty_workbook();
    assert!(wb.sheet_names().is_empty());
}

#[test]
fn create_sheet_success() -> Result<(), Box<dyn Error>> {
    let mut wb = empty_workbook();
    wb.create_sheet("new_sheet", vec![ColumnDef::new("id", DataType::Integer)])?;
    assert!(wb.sheet("new_sheet").is_ok());
    assert_eq!(wb.sheet_names(), vec!["new_sheet".to_string()]);
    Ok(())
}

#[test]
fn create_sheet_duplicate_name_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.create_sheet("alpha", vec![ColumnDef::new("z", DataType::Integer)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn create_sheet_invalid_schema_returns_error() {
    let mut wb = empty_workbook();
    let result = wb.create_sheet("", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_err());
    let result2 = wb.create_sheet("no_cols", vec![]);
    assert!(result2.is_err());
}

#[test]
fn drop_sheet_existing_removes_sheet() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    wb.drop_sheet("alpha")?;
    assert!(wb.sheet("alpha").is_err());
    assert!(wb.sheet("beta").is_ok());
    Ok(())
}

#[test]
fn drop_sheet_missing_returns_error() {
    let mut wb = empty_workbook();
    let result = wb.drop_sheet("ghost");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn rename_sheet_success_preserves_data() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    wb.rename_sheet("alpha", "renamed")?;

    assert!(wb.sheet("alpha").is_err());
    assert!(wb.sheet("renamed").is_ok());

    assert_eq!(wb.row_count("renamed")?, 2);
    assert_eq!(wb.get_cell("renamed", 0, 0), Some(&Value::from(1_i64)));
    Ok(())
}

#[test]
fn rename_sheet_missing_old_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.rename_sheet("ghost", "new_name");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn rename_sheet_target_exists_returns_file_exists() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.rename_sheet("alpha", "beta");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::FileExists);
    }
    Ok(())
}

#[test]
fn rename_sheet_invalid_new_name_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.rename_sheet("alpha", "");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn insert_row_success() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let before = wb.row_count("alpha")?;
    wb.insert_row("alpha", vec![Value::from(99_i64)])?;
    assert_eq!(wb.row_count("alpha")?, before + 1);
    assert_eq!(wb.get_cell("alpha", before, 0), Some(&Value::from(99_i64)));
    Ok(())
}

#[test]
fn insert_row_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.insert_row("ghost", vec![Value::from(1_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn insert_row_wrong_number_of_values_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.insert_row("alpha", vec![Value::from(1_i64), Value::from(2_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn insert_row_type_mismatch_returns_error() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let result = wb.insert_row("alpha", vec![Value::from("not an int")]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn delete_row_success() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let before = wb.row_count("alpha")?;
    wb.delete_row("alpha", 0)?;
    assert_eq!(wb.row_count("alpha")?, before - 1);
    assert_eq!(wb.get_cell("alpha", 0, 0), Some(&Value::from(2_i64)));
    Ok(())
}

#[test]
fn delete_row_index_out_of_bounds_returns_invalid_reference() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    let len = wb.row_count("alpha")?;
    let result = wb.delete_row("alpha", len);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidReference);
    }
    Ok(())
}

#[test]
fn delete_row_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.delete_row("ghost", 0);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}

#[test]
fn clear_sheet_removes_all_rows() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_sheets()?;
    wb.clear_sheet("alpha")?;
    assert_eq!(wb.row_count("alpha")?, 0);
    assert_eq!(wb.row_count("beta")?, 1);
    Ok(())
}

#[test]
fn clear_sheet_missing_sheet_returns_db_error() {
    let mut wb = empty_workbook();
    let result = wb.clear_sheet("ghost");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
}
