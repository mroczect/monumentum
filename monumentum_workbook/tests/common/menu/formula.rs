#![allow(clippy::std_instead_of_core)]
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError};

fn create_workbook() -> Result<Workbook<InMemoryStorage>, Box<dyn std::error::Error>> {
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
    let columns = vec![
        ColumnDef::new("Nama", DataType::Text),
        ColumnDef::new("Nilai", DataType::Integer),
    ];
    wb.create_sheet("Data", columns)?;
    wb.insert_row("Data", vec![Value::from("A"), Value::from(10_i64)])?;
    wb.insert_row("Data", vec![Value::from("B"), Value::from(20_i64)])?;
    wb.insert_row("Data", vec![Value::from("C"), Value::from(30_i64)])?;
    Ok(wb)
}

#[test]
fn set_formula_and_evaluate_average() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    let row_idx = wb.row_count("Data")?;
    wb.insert_row("Data", vec![Value::from("Avg"), Value::Null])?;
    wb.set_formula("Data", row_idx, 1, "AVERAGE(B1:B3)")?;

    let result = wb.get_cell_value("Data", row_idx, 1)?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 20.0).abs() < f64::EPSILON);
    } else {
        return Err(format!("expected Float(20.0), got {result:?}").into());
    }
    Ok(())
}

#[test]
fn set_formula_and_evaluate_sum() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    let row_idx = wb.row_count("Data")?;
    wb.insert_row("Data", vec![Value::from("Sum"), Value::Null])?;
    wb.set_formula("Data", row_idx, 1, "SUM(B1:B3)")?;

    let result = wb.get_cell_value("Data", row_idx, 1)?;
    assert_eq!(result, Value::Integer(60_i64.into()));
    Ok(())
}

#[test]
fn set_formula_and_evaluate_max() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    let row_idx = wb.row_count("Data")?;
    wb.insert_row("Data", vec![Value::from("Max"), Value::Null])?;
    wb.set_formula("Data", row_idx, 1, "MAX(B1:B3)")?;

    let result = wb.get_cell_value("Data", row_idx, 1)?;
    assert_eq!(result, Value::Integer(30_i64.into()));
    Ok(())
}

#[test]
fn set_formula_and_evaluate_min() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    let row_idx = wb.row_count("Data")?;
    wb.insert_row("Data", vec![Value::from("Min"), Value::Null])?;
    wb.set_formula("Data", row_idx, 1, "MIN(B1:B3)")?;

    let result = wb.get_cell_value("Data", row_idx, 1)?;
    assert_eq!(result, Value::Integer(10_i64.into()));
    Ok(())
}

#[test]
fn formula_circular_reference_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    wb.set_formula("Data", 0, 0, "A1")?;
    let result = wb.get_cell_value("Data", 0, 0);
    assert!(matches!(result, Err(WorkbookError::Formula(_))));
    Ok(())
}

#[test]
fn formula_invalid_reference_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = create_workbook()?;
    let row_idx = wb.row_count("Data")?;
    wb.insert_row("Data", vec![Value::from("Invalid"), Value::Null])?;
    wb.set_formula("Data", row_idx, 1, "XFE1")?;
    let result = wb.get_cell_value("Data", row_idx, 1);
    assert!(matches!(result, Err(WorkbookError::Formula(_))));
    Ok(())
}
