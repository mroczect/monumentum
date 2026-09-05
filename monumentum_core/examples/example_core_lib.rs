use fs2 as _;
use proptest as _;

use core::error::Error;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::traits::StorageEngine;
use monumentum_handler::types::Text;


fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_crud_{}_{}.db",
        std::process::id(),
        nanos
    ))
}

fn text_value(s: &str) -> Result<Value, Box<dyn Error>> {
    let text = Text::try_new(s.to_string())?;
    Ok(Value::from(text))
}

fn create_schema_employees() -> Result<TableSchema, Box<dyn Error>> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    Ok(TableSchema::try_new(
        "employees",
        vec![
            id_col,
            ColumnDef::new("name", DataType::Text),
            ColumnDef::new("salary", DataType::Float),
        ],
    )?)
}

fn insert_sample_rows(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    let rows = [
        Row::new(vec![
            Value::from(1_i64),
            text_value("Alice")?,
            Value::try_from(7500.5_f64)?,
        ]),
        Row::new(vec![
            Value::from(2_i64),
            text_value("Bob")?,
            Value::try_from(6200.0_f64)?,
        ]),
        Row::new(vec![
            Value::from(3_i64),
            text_value("Charlie")?,
            Value::try_from(8000.0_f64)?,
        ]),
    ];

    for row in &rows {
        storage.insert_row("employees", row)?;
    }
    Ok(())
}

fn cleanup_files(path: &Path) {
    let wal_path = path.with_extension("wal");
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(wal_path);
}


fn test_create_table_success(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 01] Create table 'employees'");
    let schema = create_schema_employees()?;
    storage.create_table(schema.clone())?;
    println!("  ✅ Table created: {} ({} columns)", schema.name(), schema.columns().len());
    Ok(())
}

fn test_create_duplicate_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 02] Duplicate table creation should fail");
    let schema = create_schema_employees()?;
    let result = storage.create_table(schema);
    match result {
        Err(e) => println!("  ✅ Correctly rejected: {e}"),
        Ok(()) => {
            println!("  ❌ Unexpected success");
            return Err("duplicate table creation unexpectedly succeeded".into());
        }
    }
    Ok(())
}

fn test_insert_and_get_rows(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 03] Insert and read rows");
    insert_sample_rows(storage)?;
    let row0 = storage.get_row("employees", 0)?;
    let row1 = storage.get_row("employees", 1)?;
    println!("  Row 0: {:?}", row0);
    println!("  Row 1: {:?}", row1);
    assert!(row0.is_some());
    assert!(row1.is_some());
    println!("  ✅ Rows inserted and read back");
    Ok(())
}

fn test_get_row_by_primary_key(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 04] Lookup by primary key");
    let row = storage.get_row_by_key("employees", &Value::from(2_i64))?;
    println!("  Row with id=2: {:?}", row);
    assert!(row.is_some());
    println!("  ✅ Primary key lookup works");
    Ok(())
}

fn test_primary_key_duplicate(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 05] Duplicate primary key should fail");
    let duplicate = Row::new(vec![
        Value::from(2_i64),
        text_value("Bob Duplicate")?,
        Value::try_from(0.0_f64)?,
    ]);
    let result = storage.insert_row("employees", &duplicate);
    match result {
        Err(e) => println!("  ✅ Correctly rejected: {e}"),
        Ok(()) => {
            println!("  ❌ Unexpected success");
            return Err("duplicate primary key unexpectedly succeeded".into());
        }
    }
    Ok(())
}

fn test_update_cell(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 06] Update cell (salary and name)");
    storage.set_cell("employees", 1, 2, Value::try_from(6500.0_f64)?)?;
    storage.set_cell("employees", 1, 1, text_value("Robert")?)?;

    let updated = storage.get_row("employees", 1)?;
    println!("  Updated row: {:?}", updated);
    if let Some(row) = updated {
        assert_eq!(row.get(2), Some(&Value::try_from(6500.0_f64)?));
        assert_eq!(row.get(1), Some(&text_value("Robert")?));
    } else {
        return Err("expected updated row".into());
    }
    println!("  ✅ Cell update verified");
    Ok(())
}

fn test_replace_rows(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 07] Replace all rows");
    let new_rows = [
        Row::new(vec![
            Value::from(10_i64),
            text_value("Dave")?,
            Value::try_from(9000.0_f64)?,
        ]),
        Row::new(vec![
            Value::from(11_i64),
            text_value("Eve")?,
            Value::try_from(9500.0_f64)?,
        ]),
    ];

    storage.replace_rows("employees", new_rows.to_vec())?;
    let row0 = storage.get_row("employees", 0)?;
    let row1 = storage.get_row("employees", 1)?;
    println!("  Row 0: {:?}", row0);
    println!("  Row 1: {:?}", row1);

    assert_eq!(row0, new_rows.first().cloned());
    assert_eq!(row1, new_rows.get(1).cloned());
    println!("  ✅ Rows replaced successfully");
    Ok(())
}

fn test_delete_row_via_replace(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 08] Delete row by replacing with fewer rows");
    let remaining = [
        Row::new(vec![
            Value::from(11_i64),
            text_value("Eve")?,
            Value::try_from(9500.0_f64)?,
        ]),
    ];

    storage.replace_rows("employees", remaining.to_vec())?;
    let row0 = storage.get_row("employees", 0)?;
    let row1 = storage.get_row("employees", 1)?;
    println!("  Row 0 after delete: {:?}", row0);
    println!("  Row 1 after delete: {:?}", row1);

    assert_eq!(row0, remaining.first().cloned());
    assert_eq!(row1, None);
    println!("  ✅ Row deletion verified");
    Ok(())
}

fn test_drop_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 09] Drop table 'employees'");
    storage.drop_table("employees")?;
    let result = storage.get_row("employees", 0);
    match result {
        Err(e) => println!("  ✅ Correctly table not found: {e}"),
        Ok(_) => {
            println!("  ❌ Unexpected success reading dropped table");
            return Err("dropped table still accessible".into());
        }
    }
    Ok(())
}

fn test_missing_table_error(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 10] Operations on missing table should fail");
    let result = storage.get_row("nonexistent", 0);
    match result {
        Err(e) => println!("  ✅ Correctly rejected: {e}"),
        Ok(_) => {
            println!("  ❌ Unexpected success");
            return Err("missing table access unexpectedly succeeded".into());
        }
    }
    Ok(())
}

fn test_schema_validation() -> Result<(), Box<dyn Error>> {
    println!("[CASE 11] Schema validation (outside storage)");
    let schema = create_schema_employees()?;

    let valid = Row::new(vec![
        Value::from(1_i64),
        text_value("Alice")?,
        Value::try_from(7500.5_f64)?,
    ]);
    if let Err(e) = schema.validate_values(valid.values()) {
        println!("  ❌ Valid row rejected: {e}");
        return Err(e.into());
    }
    println!("  ✅ Valid row accepted");

    let invalid_count = Row::new(vec![Value::from(1_i64), text_value("Alice")?]);
    match schema.validate_values(invalid_count.values()) {
        Err(e) => println!("  ✅ Invalid column count rejected: {e}"),
        Ok(()) => {
            println!("  ❌ Invalid column count accepted");
            return Err("schema validation missed column count mismatch".into());
        }
    }

    let invalid_type = Row::new(vec![
        Value::from(1_i64),
        Value::try_from(1234.5_f64)?, 
        Value::try_from(7500.5_f64)?,
    ]);
    match schema.validate_values(invalid_type.values()) {
        Err(e) => println!("  ✅ Type mismatch rejected: {e}"),
        Ok(()) => {
            println!("  ❌ Type mismatch accepted");
            return Err("schema validation missed type mismatch".into());
        }
    }

    Ok(())
}

fn test_file_reopen_persistence(path: &Path) -> Result<(), Box<dyn Error>> {
    println!("[CASE 12] Reopen database and verify persistence");
    {
        let mut storage = FileStorage::open(path, 10)?;
        let schema = create_schema_employees()?;
        storage.create_table(schema)?;
        let row = Row::new(vec![
            Value::from(1_i64),
            text_value("Alice")?,
            Value::try_from(7500.5_f64)?,
        ]);
        storage.insert_row("employees", &row)?;
        storage.checkpoint()?;
        storage.close()?;
    }

    {
        let mut storage = FileStorage::open(path, 10)?;
        let row = storage.get_row("employees", 0)?;
        println!("  Row after reopen: {:?}", row);
        assert!(row.is_some());
        println!("  ✅ Data persisted and recovered");
        storage.close()?;
    }
    Ok(())
}

fn test_file_locking(path: &Path) -> Result<(), Box<dyn Error>> {
    println!("[CASE 13] File locking prevents concurrent writer");
    let _first = FileStorage::open(path, 10)?;
    let second = FileStorage::open(path, 10);
    match second {
        Err(e) => println!("  ✅ Second open correctly rejected: {e}"),
        Ok(_) => {
            println!("  ❌ Concurrent write was not prevented");
            return Err("file locking failed".into());
        }
    }
    Ok(())
}


fn run_full_test_suite() -> Result<(), Box<dyn Error>> {
    println!("=== Monumentum Full Runtime CRUD Test Suite ===\n");

    let path = temp_db_path();
    println!("[SETUP] Database file: {}\n", path.display());

    {
        let mut storage = FileStorage::open(&path, 10)?;

        test_create_table_success(&mut storage)?;
        test_create_duplicate_table(&mut storage)?;
        test_insert_and_get_rows(&mut storage)?;
        test_get_row_by_primary_key(&mut storage)?;
        test_primary_key_duplicate(&mut storage)?;
        test_update_cell(&mut storage)?;
        test_replace_rows(&mut storage)?;
        test_delete_row_via_replace(&mut storage)?;
        test_drop_table(&mut storage)?;
        test_missing_table_error(&mut storage)?;

        storage.checkpoint()?;
        storage.close()?;
    }

    test_schema_validation()?;

    test_file_reopen_persistence(&path)?;
    test_file_locking(&path)?;

    cleanup_files(&path);

    println!("\n=== All 13 cases completed successfully ===");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_full_test_suite()
}