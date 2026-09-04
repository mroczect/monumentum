# Monumentum DB: Example Usage Guide

This guide provides practical examples for using the `monumentum_db` crate. It covers core data structures, in‑memory and file‑backed storage, schema constraints, and error handling. The examples are self‑contained and ready to adapt.

---

## 1. Setting Up

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
monumentum_db = "0.1.0"
```

Import the necessary items:

```rust
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::store::storage::{FileStorage, InMemoryStorage};
```

---

## 2. Core Data Structures

### 2.1 Creating an In‑Memory Catalog

```rust
let mut catalog = Catalog::new();

// Define a schema
let schema = TableSchema::try_new(
    "users",
    vec![
        ColumnDef::new("id", DataType::Integer),
        ColumnDef::new("name", DataType::Text),
        ColumnDef::new("age", DataType::Integer),
    ],
)?;

// Add the table to the catalog
catalog.create_table(schema)?;
```

### 2.2 Inserting Rows

```rust
// Get mutable access to the table
let table = catalog.get_table_mut("users").unwrap();

// Insert a row
table.insert(Row::new(vec![
    Value::from(1_i64),
    Value::from("Alice"),
    Value::from(30_i64),
]))?;

// Insert another
table.insert(Row::new(vec![
    Value::from(2_i64),
    Value::from("Bob"),
    Value::from(25_i64),
]))?;
```

### 2.3 Reading Rows and Cells

```rust
let table = catalog.get_table("users").unwrap();

// Row count
assert_eq!(table.len(), 2);

// Get a row
let row = table.get(0).unwrap();
assert_eq!(row.get(0), Some(&Value::from(1_i64)));

// Access by column name (requires `ColumnIndex` trait)
let name: Option<&Value> = row.get("name");
assert_eq!(name, Some(&Value::from("Alice")));
```

### 2.4 Updating a Cell

```rust
let table = catalog.get_table_mut("users").unwrap();
table.set_cell(0, 2, Value::from(31_i64))?; // Alice's age becomes 31
```

### 2.5 Deleting a Row

There is no direct `delete_row` method on `Table`. Instead, use `replace_rows` with a filtered vector.

```rust
let table = catalog.get_table_mut("users").unwrap();
let rows: Vec<Row> = table.rows().iter()
    .filter(|row| row.get(0) != Some(&Value::from(1_i64))) // keep everyone except id=1
    .cloned()
    .collect();
table.replace_rows(rows)?;
```

---

## 3. Schema and Constraints

### 3.1 Defining a Column with Constraints

```rust
let mut id_col = ColumnDef::new("id", DataType::Integer);
id_col.set_primary_key(true);       // sets nullable=false, unique=true
id_col.set_default(Some(Value::from(0_i64))); // optional

let mut name_col = ColumnDef::new("name", DataType::Text);
name_col.set_nullable(false);       // cannot be NULL

let schema = TableSchema::try_new("users", vec![id_col, name_col])?;
```

### 3.2 Using Check Constraints

```rust
use monumentum_db::core::schema::column::{CheckConstraint, ComparisonOp};

let mut age_col = ColumnDef::new("age", DataType::Integer);
age_col.set_check(Some(CheckConstraint {
    column: "age".to_string(),
    op: ComparisonOp::Gte,
    value: Value::from(0_i64),
}));
```

### 3.3 Allowed Values

```rust
let mut status_col = ColumnDef::new("status", DataType::Text);
status_col.set_allowed_values(Some(vec![
    Value::from("active"),
    Value::from("inactive"),
]));
```

---

## 4. Using File Storage

### 4.1 Creating and Saving a File‑Backed Database

```rust
use std::path::Path;

let path = Path::new("mydb.monumentum");
let mut storage = FileStorage::open(path)?;

// Create a catalog and save it
let mut catalog = Catalog::new();
let schema = TableSchema::try_new("items", vec![ColumnDef::new("id", DataType::Integer)])?;
catalog.create_table(schema)?;

storage.save_catalog(&catalog)?;
storage.checkpoint()?;   // write snapshot and clear WAL
storage.close()?;        // release lock
```

### 4.2 Loading a Database

```rust
let mut storage = FileStorage::open(path)?;
let catalog = storage.load_catalog()?;
assert!(catalog.get_table("items").is_some());
storage.close()?;
```

### 4.3 Reloading from Disk (Discarding Unsaved Changes)

```rust
let mut storage = FileStorage::open(path)?;
let catalog = storage.reload_from_disk()?;
storage.close()?;
```

---

## 5. In‑Memory Storage

The `InMemoryStorage` is useful for tests or ephemeral data.

```rust
let mut storage = InMemoryStorage::new();

// Save a catalog
let catalog = Catalog::new();
storage.save_catalog(&catalog)?;

// Load it back
let loaded = storage.load_catalog()?;
assert_eq!(loaded.len(), 0);
```

---

## 6. Error Handling

`DbError` provides rich error information. You can match on specific variants or use the `MonumentumError` trait methods.

```rust
use monumentum_db::error::MonumentumError;

fn handle_result(result: Result<(), DbError>) {
    match result {
        Ok(()) => println!("Success"),
        Err(e) => {
            eprintln!("Error: {}", e);
            match e.kind() {
                monumentum_db::error::ErrorKind::UniqueViolation => println!("Unique violation"),
                monumentum_db::error::ErrorKind::TypeMismatch => println!("Type mismatch"),
                _ => {}
            }
        }
    }
}
```

### Example: Duplicate Primary Key

```rust
let mut catalog = Catalog::new();
let mut id_col = ColumnDef::new("id", DataType::Integer);
id_col.set_primary_key(true);
let schema = TableSchema::try_new("t", vec![id_col])?;
catalog.create_table(schema)?;

let table = catalog.get_table_mut("t").unwrap();
table.insert(Row::new(vec![Value::from(1_i64)]))?;
let result = table.insert(Row::new(vec![Value::from(1_i64)]));
assert!(result.is_err());
// The error will be DbError::InvalidOperation (duplicate value)
```

---

## 7. Serialization (Advanced)

The crate can serialize and deserialize a `Catalog` to a binary format.

```rust
use monumentum_db::store::serde::{encode_catalog, decode_catalog};

// Assuming `catalog` exists
let bytes = encode_catalog(&catalog)?;
let restored = decode_catalog(&bytes)?;
assert_eq!(catalog, restored);
```

---

## 8. Write‑Ahead Log (WAL)

If you need custom WAL handling (normally internal), you can use the `Wal` type directly.

```rust
use monumentum_db::store::wal::Wal;

let mut wal = Wal::open(Path::new("custom.wal"))?;
wal.append(b"record1")?;
wal.append(b"record2")?;

let records = wal.read_all()?;
assert_eq!(records, vec![b"record1".to_vec(), b"record2".to_vec()]);

wal.truncate()?;   // clear the log
wal.unlock()?;
```

---

## 9. Summary

This guide covers the most important operations in `monumentum_db`:

- Creating catalogs, schemas, and tables.
- Inserting, reading, and modifying rows.
- Applying constraints (primary key, unique, check, allowed values).
- Using in‑memory and file‑backed storage.
- Error handling with `DbError` and `MonumentumError`.
- Serialization and WAL.

For a complete list of all methods and types, refer to the API documentation.
