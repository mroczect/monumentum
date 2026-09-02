use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::Workbook;
use monumentum_workbook::transaction::Transaction;
use pretty_assertions::assert_eq;

fn workbook_with_table() -> Result<Workbook<InMemoryStorage>, Box<dyn Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("main", vec![ColumnDef::new("id", DataType::Integer)])?;
    catalog.create_table(schema)?;
    {
        let table = catalog
            .get_table_mut("main")
            .ok_or("table 'main' not found")?;
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
    }
    Ok(Workbook::<InMemoryStorage>::load_in_memory(catalog))
}

#[test]
fn begin_and_rollback_restores_initial_state() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let initial_count = wb.row_count("main")?;

    let mut tx = Transaction::begin(&mut wb);
    tx.workbook_mut()
        .insert_row("main", vec![Value::from(3_i64)])?;
    assert_eq!(tx.workbook_mut().row_count("main")?, initial_count + 1);

    tx.rollback();

    assert_eq!(wb.row_count("main")?, initial_count);
    Ok(())
}

#[test]
fn multiple_transactions_are_independent_after_rollback() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let initial_count = wb.row_count("main")?;

    {
        let mut tx = Transaction::begin(&mut wb);
        tx.workbook_mut()
            .insert_row("main", vec![Value::from(3_i64)])?;
        tx.rollback();
    }

    assert_eq!(wb.row_count("main")?, initial_count);

    {
        let mut tx = Transaction::begin(&mut wb);
        tx.workbook_mut()
            .insert_row("main", vec![Value::from(4_i64)])?;
        tx.commit()?;
    }

    assert_eq!(wb.row_count("main")?, initial_count + 1);
    Ok(())
}

#[test]
fn workbook_mut_allows_modification() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .insert_row("main", vec![Value::from(99_i64)])?;

    assert_eq!(tx.workbook_mut().row_count("main")?, 3);
    assert_eq!(
        tx.workbook_mut().get_cell("main", 2, 0),
        Some(&Value::from(99_i64))
    );

    tx.commit()?;

    assert_eq!(wb.row_count("main")?, 3);
    assert_eq!(wb.get_cell("main", 2, 0), Some(&Value::from(99_i64)));
    Ok(())
}

#[test]
fn commit_keeps_modifications() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .insert_row("main", vec![Value::from(42_i64)])?;
    tx.commit()?;

    assert_eq!(wb.row_count("main")?, 3);
    assert_eq!(wb.get_cell("main", 2, 0), Some(&Value::from(42_i64)));
    Ok(())
}

#[test]
fn commit_without_changes_succeeds() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let tx = Transaction::begin(&mut wb);
    tx.commit()?;
    assert_eq!(wb.row_count("main")?, 2);
    Ok(())
}

#[test]
fn commit_after_add_sheet_persists_new_sheet() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .create_sheet("extra", vec![ColumnDef::new("val", DataType::Integer)])?;
    tx.commit()?;

    assert!(wb.sheet("extra").is_ok());
    Ok(())
}

#[test]
fn rollback_restores_previous_state_after_insert() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .insert_row("main", vec![Value::from(100_i64)])?;
    assert_eq!(tx.workbook_mut().row_count("main")?, 3);

    tx.rollback();

    assert_eq!(wb.row_count("main")?, 2);
    assert_eq!(wb.get_cell("main", 2, 0), None);
    Ok(())
}

#[test]
fn rollback_restores_cell_value_after_set_cell() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .set_cell("main", 0, 0, Value::from(999_i64))?;
    assert_eq!(
        tx.workbook_mut().get_cell("main", 0, 0),
        Some(&Value::from(999_i64))
    );

    tx.rollback();

    assert_eq!(wb.get_cell("main", 0, 0), Some(&Value::from(1_i64)));
    Ok(())
}

#[test]
fn rollback_after_drop_sheet_restores_sheet() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut().drop_sheet("main")?;
    assert!(tx.workbook_mut().sheet("main").is_err());

    tx.rollback();

    assert!(wb.sheet("main").is_ok());
    assert_eq!(wb.row_count("main")?, 2);
    Ok(())
}

#[test]
fn rollback_after_rename_sheet_restores_old_name() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut().rename_sheet("main", "renamed")?;
    assert!(tx.workbook_mut().sheet("renamed").is_ok());
    assert!(tx.workbook_mut().sheet("main").is_err());

    tx.rollback();

    assert!(wb.sheet("main").is_ok());
    assert!(wb.sheet("renamed").is_err());
    Ok(())
}

#[test]
fn rollback_after_clear_sheet_restores_rows() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut().clear_sheet("main")?;
    assert_eq!(tx.workbook_mut().row_count("main")?, 0);

    tx.rollback();

    assert_eq!(wb.row_count("main")?, 2);
    Ok(())
}

#[test]
fn commit_after_error_in_transaction_still_commits_previous_successful_ops()
-> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .insert_row("main", vec![Value::from(5_i64)])?;
    let err = tx
        .workbook_mut()
        .insert_row("main", vec![Value::from("wrong")]);
    assert!(err.is_err());

    tx.commit()?;
    assert_eq!(wb.row_count("main")?, 3);
    Ok(())
}

#[test]
fn rollback_after_error_restores_initial_state() -> Result<(), Box<dyn Error>> {
    let mut wb = workbook_with_table()?;
    let mut tx = Transaction::begin(&mut wb);

    tx.workbook_mut()
        .insert_row("main", vec![Value::from(5_i64)])?;
    let _err = tx
        .workbook_mut()
        .insert_row("main", vec![Value::from("wrong")]);

    tx.rollback();
    assert_eq!(wb.row_count("main")?, 2);
    Ok(())
}
