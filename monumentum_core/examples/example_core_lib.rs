use core::error::Error;
use fs2 as _;
use proptest as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::traits::StorageEngine;
use monumentum_handler::types::Text;

fn temp_db_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_{label}_{}_{}.db",
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
    println!(
        "  OK: Table created: {} ({} columns)",
        schema.name(),
        schema.columns().len()
    );
    Ok(())
}

fn test_create_duplicate_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 02] Duplicate table creation should fail");
    let schema = create_schema_employees()?;
    let result = storage.create_table(schema);
    match result {
        Err(e) => println!("  OK: Correctly rejected: {e}"),
        Ok(()) => return Err("duplicate table creation unexpectedly succeeded".into()),
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
    println!("  OK: Rows inserted and read back");
    Ok(())
}

fn test_get_row_out_of_bounds(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 04] Get row out of bounds returns None");
    let row = storage.get_row("employees", 999)?;
    println!("  Row 999: {:?}", row);
    assert_eq!(row, None);
    println!("  OK: Out-of-bounds row correctly returns None");
    Ok(())
}

fn test_get_row_by_primary_key(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 05] Lookup by primary key");
    let row = storage.get_row_by_key("employees", &Value::from(2_i64))?;
    println!("  Row with id=2: {:?}", row);
    assert!(row.is_some());
    println!("  OK: Primary key lookup works");
    Ok(())
}

fn test_primary_key_duplicate(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 06] Duplicate primary key should fail");
    let duplicate = Row::new(vec![
        Value::from(2_i64),
        text_value("Bob Duplicate")?,
        Value::try_from(0.0_f64)?,
    ]);
    let result = storage.insert_row("employees", &duplicate);
    match result {
        Err(e) => println!("  OK: Correctly rejected: {e}"),
        Ok(()) => return Err("duplicate primary key unexpectedly succeeded".into()),
    }
    Ok(())
}

fn update_cells(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 07] Update salary cell on table without primary key");

    {
        let schema = TableSchema::try_new(
            "temp_no_pk",
            vec![
                ColumnDef::new("id", DataType::Integer),
                ColumnDef::new("value", DataType::Text),
            ],
        )?;

        storage.create_table(schema)?;

        let row = Row::new(vec![Value::from(1_i64), text_value("old")?]);
        storage.insert_row("temp_no_pk", &row)?;

        storage.set_cell("temp_no_pk", 0, 1, text_value("new")?)?;

        let updated = storage.get_row("temp_no_pk", 0)?;
        if let Some(row) = &updated
            && let Some(val) = row.get(1)
        {
            assert_eq!(val, &text_value("new")?);
        } else {
            return Err("set_cell verification failed".into());
        }

        println!("  OK: set_cell works on table without primary key");

        storage.drop_table("temp_no_pk")?;
    }

    println!("[CASE 07b] Update name on table with primary key via replace_rows");
    let row0 = storage.get_row("employees", 0)?.ok_or("row0 missing")?;
    let row1 = storage.get_row("employees", 1)?.ok_or("row1 missing")?;

    let mut row1_values = row1.values().to_vec();
    if let Some(name_cell) = row1_values.get_mut(1) {
        *name_cell = text_value("Robert")?;
    } else {
        return Err("column index 1 missing".into());
    }
    let updated_row1 = Row::new(row1_values);

    let replacement_rows = [row0, updated_row1];
    storage.replace_rows("employees", replacement_rows.to_vec())?;

    let updated_name = storage.get_row("employees", 1)?;
    if let Some(row) = &updated_name
        && let Some(name) = row.get(1)
    {
        assert_eq!(name, &text_value("Robert")?);
    } else {
        return Err("name update failed".into());
    }
    println!("  OK: Name updated");
    Ok(())
}

fn test_replace_rows(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 08] Replace all rows");
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
    println!("  OK: Rows replaced successfully");
    Ok(())
}

fn test_delete_row_via_replace(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 09] Delete row by replacing with fewer rows");
    let remaining = [Row::new(vec![
        Value::from(11_i64),
        text_value("Eve")?,
        Value::try_from(9500.0_f64)?,
    ])];

    storage.replace_rows("employees", remaining.to_vec())?;
    let row0 = storage.get_row("employees", 0)?;
    let row1 = storage.get_row("employees", 1)?;
    println!("  Row 0 after delete: {:?}", row0);
    println!("  Row 1 after delete: {:?}", row1);

    assert_eq!(row0, remaining.first().cloned());
    assert_eq!(row1, None);
    println!("  OK: Row deletion verified");
    Ok(())
}

fn test_rename_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 10] Rename table 'employees' to 'staff'");
    storage.rename_table("employees", "staff")?;
    let old_exists = storage.get_table("employees");
    let new_exists = storage.get_table("staff");
    println!("  Old table exists: {}", old_exists.is_some());
    println!("  New table exists: {}", new_exists.is_some());
    assert!(old_exists.is_none());
    assert!(new_exists.is_some());
    storage.rename_table("staff", "employees")?;
    println!("  OK: Table renamed and renamed back");
    Ok(())
}

fn test_get_catalog(storage: &FileStorage) {
    println!("[CASE 11] Verify catalog contents");
    let catalog = storage.get_catalog();
    let table = catalog.get_table("employees");
    assert!(table.is_some());
    if let Some(table) = table {
        println!(
            "  Table 'employees' has {} columns",
            table.schema().columns().len()
        );
        assert_eq!(table.schema().columns().len(), 3);
    }
    println!("  OK: Catalog verified");
}

fn test_drop_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 12] Drop table 'employees'");
    storage.drop_table("employees")?;
    let result = storage.get_row("employees", 0);
    match result {
        Err(e) => println!("  OK: Correctly table not found: {e}"),
        Ok(_) => return Err("dropped table still accessible".into()),
    }
    Ok(())
}

fn test_drop_missing_table(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 13] Drop missing table should fail");
    let result = storage.drop_table("nonexistent");
    match result {
        Err(e) => println!("  OK: Correctly rejected: {e}"),
        Ok(()) => return Err("dropping missing table unexpectedly succeeded".into()),
    }
    Ok(())
}

fn test_missing_table_error(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("[CASE 14] Operations on missing table should fail");
    let result = storage.get_row("nonexistent", 0);
    match result {
        Err(e) => println!("  OK: Correctly rejected: {e}"),
        Ok(_) => return Err("missing table access unexpectedly succeeded".into()),
    }
    Ok(())
}

fn test_schema_validation() -> Result<(), Box<dyn Error>> {
    println!("[CASE 15] Schema validation (outside storage)");
    let schema = create_schema_employees()?;

    let valid = Row::new(vec![
        Value::from(1_i64),
        text_value("Alice")?,
        Value::try_from(7500.5_f64)?,
    ]);
    if let Err(e) = schema.validate_values(valid.values()) {
        println!("  FAIL: Valid row rejected: {e}");
        return Err(e.into());
    }
    println!("  OK: Valid row accepted");

    let invalid_count = Row::new(vec![Value::from(1_i64), text_value("Alice")?]);
    match schema.validate_values(invalid_count.values()) {
        Err(e) => println!("  OK: Invalid column count rejected: {e}"),
        Ok(()) => return Err("schema validation missed column count mismatch".into()),
    }

    let invalid_type = Row::new(vec![
        Value::from(1_i64),
        Value::try_from(1234.5_f64)?,
        Value::try_from(7500.5_f64)?,
    ]);
    match schema.validate_values(invalid_type.values()) {
        Err(e) => println!("  OK: Type mismatch rejected: {e}"),
        Ok(()) => return Err("schema validation missed type mismatch".into()),
    }

    Ok(())
}

fn test_file_reopen_persistence() -> Result<(), Box<dyn Error>> {
    println!("[CASE 16] Reopen database and verify persistence");
    let path = temp_db_path("persist");

    {
        let mut storage = FileStorage::open(&path, 10)?;
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
        let mut storage = FileStorage::open(&path, 10)?;
        let row = storage.get_row("employees", 0)?;
        println!("  Row after reopen: {:?}", row);
        assert!(row.is_some());
        println!("  OK: Data persisted and recovered");
        storage.close()?;
    }

    cleanup_files(&path);
    Ok(())
}

fn test_file_locking() -> Result<(), Box<dyn Error>> {
    println!("[CASE 17] File locking prevents concurrent writer");
    let path = temp_db_path("lock");

    let _first = FileStorage::open(&path, 10)?;
    let second = FileStorage::open(&path, 10);
    match second {
        Err(e) => println!("  OK: Second open correctly rejected: {e}"),
        Ok(_) => return Err("file locking failed".into()),
    }

    cleanup_files(&path);
    Ok(())
}

fn run_storage_tests(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut storage = FileStorage::open(path, 10)?;

    test_create_table_success(&mut storage)?;
    test_create_duplicate_table(&mut storage)?;
    test_insert_and_get_rows(&mut storage)?;
    test_get_row_out_of_bounds(&mut storage)?;
    test_get_row_by_primary_key(&mut storage)?;
    test_primary_key_duplicate(&mut storage)?;
    update_cells(&mut storage)?;
    test_replace_rows(&mut storage)?;
    test_delete_row_via_replace(&mut storage)?;
    test_rename_table(&mut storage)?;
    test_get_catalog(&storage);
    test_drop_table(&mut storage)?;
    test_drop_missing_table(&mut storage)?;
    test_missing_table_error(&mut storage)?;

    storage.checkpoint()?;
    storage.close()?;
    Ok(())
}

fn run_full_test_suite() -> Result<(), Box<dyn Error>> {
    println!("=== Monumentum Full Runtime CRUD Test Suite ===\n");

    let path = temp_db_path("main");
    println!("[SETUP] Database file: {}\n", path.display());

    run_storage_tests(&path)?;

    test_schema_validation()?;
    test_file_reopen_persistence()?;
    test_file_locking()?;

    println!("\n=== All 17 cases completed successfully ===");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_full_test_suite()
}
