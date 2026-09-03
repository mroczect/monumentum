# Monumentum DB

**A lightweight, embedded database engine written in Rust.**

`monumentum_db` is a strongly typed storage layer with in-memory and file-based persistence, write-ahead logging (WAL), and explicit constraint enforcement. It serves as the foundation for the Monumentum workspace and can be used standalone for applications that need a safe, embedded data store.

---

## Table of Contents

- [Overview](#overview)
- [Design Goals](#design-goals)
- [Installation](#installation)
- [Architecture](#architecture)
- [Data Model](#data-model)
  - [Value](#value)
  - [Row](#row)
  - [ColumnDef](#columndef)
  - [TableSchema](#tableschema)
  - [Table](#table)
  - [Catalog](#catalog)
- [Storage Layer](#storage-layer)
  - [StorageEngine](#storageengine)
  - [FileStorage](#filestorage)
  - [InMemoryStorage](#inmemorystorage)
  - [WAL & Append Log](#wal--append-log)
  - [Serialization](#serialization)
  - [File Operations](#file-operations)
  - [Recovery](#recovery)
- [Error Handling](#error-handling)
- [Types](#types)
  - [Integer](#integer)
  - [Float](#float)
  - [Text](#text)
  - [Blob](#blob)
- [Full API Reference](#full-api-reference)
- [Examples](#examples)
  - [In-Memory Database](#in-memory-database)
  - [File-Based Database](#file-based-database)
- [Testing](#testing)
- [Security](#security)
- [Limitations](#limitations)
- [License](#license)

---

## Overview

`monumentum_db` provides:

- Typed tables with schemas, columns, rows, and cells.
- Constraints: primary key, unique, not null, default, check, and allowed values.
- In-memory and file-backed storage engines.
- Durable file storage using snapshots + write-ahead log.
- Serialization for all data structures.
- Safe file operations (atomic writes, file locking, symlink protection).
- A single error type (`DbError`) for explicit error handling.

The crate is part of the Monumentum workspace:

- `monumentum_db` – core storage engine (this crate)
- `monumentum_query` – formula engine for spreadsheet calculations
- `monumentum_workbook` – high-level spreadsheet API
- `monumentum_functions` – preset functions for formula evaluation

---

## Design Goals

- **Safety** – no `unsafe` code; all fallible operations return `Result`.
- **Durability** – WAL with CRC32 checksums and atomic snapshot replacement.
- **Integrity** – constraints are enforced at the storage layer.
- **Portability** – standard library only; platform-specific features degrade gracefully.
- **Clarity** – small, well-documented public API.

---

## Installation

Add via Git:

```toml
[dependencies]
monumentum_db = { git = "https://github.com/mroczect/monumentum" }
```

---

## Architecture

The crate is organized into three main modules:

### `core`

Defines the in-memory data model:

- `Catalog` – ordered map of table names to `Table` objects.
- `Table` – schema + rows + optional unique indexes.
- `Row` – ordered list of `Value`s.
- `Value` – strongly typed scalar value.
- `schema::ColumnDef` – column definition with constraints.
- `schema::TableSchema` – validated list of columns.
- `index::HashIndex` – internal map for unique constraint enforcement.

### `store`

Handles persistence:

- `StorageEngine` – trait for load/save.
- `FileStorage` – file-based durable storage.
- `InMemoryStorage` – transient storage.
- `wal::Wal` – append-only log with locking.
- `append_log` – low-level record format with CRC32.
- `serialize` – binary serialization/deserialization.
- `file` – safe file operations.
- `recovery` – WAL recovery utilities.

### `types`

Safe wrappers:

- `Integer` – checked arithmetic on `i64`.
- `Float` – finite-only `f64`.
- `Text` – UTF-8 string helper.
- `Blob` – byte array.

---

## Data Model

### `Value`

```rust
pub enum Value {
    Null,
    Integer(Integer),
    Float(Float),
    Text(Text),
    Blob(Blob),
    Boolean(bool),
    Formula(String),
}
```

- `Null` represents missing data.
- `Integer`, `Float`, `Text`, `Blob` wrap the corresponding `types` wrappers.
- `Boolean` holds a `bool`.
- `Formula` holds a raw formula string (used by higher layers).

`Value` implements:

- `Display`
- `Default` (defaults to `Null`)
- `From` for `()`, `i64`, `String`, `&str`, `Vec<u8>`, `&[u8]`, `bool`, `Integer`, `Float`, `Text`, `Blob`
- `TryFrom<f64>` (rejects non-finite values)

Accessor methods:

- `is_null`, `is_integer`, `is_float`, `is_text`, `is_blob`, `is_boolean`, `is_formula`
- `as_integer`, `as_float`, `as_text`, `as_blob`, `as_boolean`, `as_formula`
- `into_integer`, `into_float`, `into_text`, `into_blob`, `into_boolean`, `into_formula`
- `type_name` – returns a static string description.

### `Row`

```rust
pub struct Row {
    values: Vec<Value>,
}
```

Methods:

- `new(values)` – create a row.
- `values()` – slice of all values.
- `get(index)` – optional access to a value.
- `len()`, `is_empty()` – size checks.
- `values_mut()`, `get_mut(index)` – mutable access (used internally).

### `ColumnDef`

```rust
pub struct ColumnDef {
    name: String,
    data_type: DataType,
    nullable: bool,
    primary_key: bool,
    unique: bool,
    default_value: Option<Value>,
    check_constraint: Option<CheckConstraint>,
    foreign_key: Option<ForeignKey>,
    allowed_values: Option<Vec<Value>>,
}
```

Where:

```rust
pub enum DataType { Null, Integer, Float, Text, Blob }

pub enum ComparisonOp { Eq, NotEq, Lt, Lte, Gt, Gte }

pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}

pub struct ForeignKey {
    pub table: String,
    pub column: String,
}
```

Methods:

- `new(name, data_type)` – creates a nullable, non-key column.
- Getters for all fields.
- Setters:
  - `set_nullable`
  - `set_primary_key` (forces `nullable = false`, `unique = true`)
  - `set_unique`
  - `set_default`
  - `set_check`
  - `set_foreign_key`
  - `set_allowed_values`
- `validate_value(&Value)` – checks formula allowance, nullability, data type, check constraint, and allowed values.

### `TableSchema`

```rust
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
}
```

Methods:

- `try_new(name, columns)` – validates:
  - non-empty table name
  - at least one column
  - max 1024 columns
  - non-empty unique column names (case-insensitive)
- `name()`, `columns()`
- `column_index(name)` – case-insensitive lookup
- `get_column(name)`
- `get_column_mut(index)` – used internally for schema changes
- `validate_values(&[Value])` – validates a row against all columns.

### `Table`

```rust
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
    unique_indexes: Vec<Option<HashIndex>>,
    read_only: bool,
}
```

Methods:

- `new(schema)` – creates an empty table.
- `insert(row)` – validates, applies defaults, checks duplicates, updates indexes.
- `replace_rows(rows)` – replaces all rows after validation and duplicate check.
- `set_cell(row_idx, col_idx, value)` – updates one cell, maintaining unique indexes and validation.
- `set_column_allowed_values(col_idx, values)` – updates allowed values with full validation of existing rows.
- `lookup_by_unique(col_idx, value)` – fast lookup via unique index or linear scan.
- `schema()`, `rows()`, `len()`, `is_empty()`, `get(index)`
- `rename_schema(new_name)` – renames the table (used by catalog rename).
- `is_read_only()`, `set_read_only(bool)`

### `Catalog`

```rust
pub struct Catalog {
    tables: BTreeMap<String, Table>,
}
```

Methods:

- `new()` – empty catalog.
- `create_table(schema)` – inserts a new table.
- `drop_table(name)` – removes a table.
- `replace_table(name, table)` – replaces an existing table atomically.
- `rename_table(old_name, new_name)` – atomically renames a table and updates schema name.
- `get_table(name)`, `get_table_mut(name)`
- `tables()` – iterator over `(&str, &Table)`.
- `len()`, `is_empty()`

---

## Storage Layer

### `StorageEngine`

```rust
pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
}
```

Implementations:

- `InMemoryStorage` – stores a `Catalog` in memory.
- `FileStorage` – persists via snapshot and WAL.

### `FileStorage`

On `open(path)`:

1. Opens or creates the WAL file (`path` with `.wal` extension) and locks it.
2. If the snapshot file exists, reads and decodes it (limit 256 MB).
3. Reads all WAL records, decodes each snapshot, and applies the one with the highest sequence number.
4. Returns a ready-to-use `FileStorage`.

Methods:

- `open(path)` – open or create.
- `save_catalog(catalog)` – append a new snapshot to WAL with incremented sequence.
- `checkpoint()` – atomically write current catalog to snapshot and truncate WAL.
- `sync()` – flush WAL to disk.
- `reload_from_disk()` – discard in-memory state and reload from snapshot + WAL.
- `close(self)` – unlock WAL.

### `InMemoryStorage`

Simple in-memory implementation of `StorageEngine`. `save_catalog` clones and stores the catalog; `load_catalog` returns a clone.

### WAL & Append Log

The WAL uses a custom record format:

```
[magic: u32][version: u32][length: u64][checksum: u32][payload: bytes]
```

- `MAGIC = 0x4D4F4E55`
- `VERSION = 1`
- `HEADER_SIZE = 20`
- `MAX_RECORD_SIZE = 64 MiB`

Functions:

- `append_record(file, payload)` – appends a validated record with CRC32.
- `read_records(file)` – reads all records, validating magic, version, length, checksum.

### Serialization

The `serialize` module provides encode/decode functions for:

- `encode_catalog` / `decode_catalog`
- `encode_table` / `decode_table`
- `encode_table_schema` / `decode_table_schema`
- `encode_column_def` / `decode_column_def`
- `encode_row` / `decode_row`
- `encode_value` / `decode_value`

All encoding is little-endian with length prefixes. Decoding enforces maximum lengths and returns `DbError::Corruption` for invalid data.

### File Operations

The `file` module provides:

- `open_or_create(path)` – opens with `O_NOFOLLOW` and mode `0600`.
- `read_file(path)` – reads entire file.
- `write_all_atomic(path, data)` – writes to a temp file, fsyncs, renames, fsyncs directory.
- `append_to_file(file, data)`
- `sync_file(file)`

Temp files use 16 random bytes from `/dev/urandom` for uniqueness.

### Recovery

```rust
pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}

pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError>;
```

Reads all records from a WAL file.

---

## Error Handling

All fallible functions return `Result<T, DbError>`.

```rust
pub enum DbError {
    Io(std::io::Error),
    Corruption(Box<dyn Error + Send + Sync>),
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    InvalidOperation(String),
    InvalidQuery(String),
    Transaction(Box<dyn Error + Send + Sync>),
    Unsupported(String),
}
```

Constructors:

- `DbError::table_not_found(name)`
- `DbError::column_not_found(name)`
- `DbError::type_mismatch(msg)`
- `DbError::invalid_operation(msg)`
- `DbError::invalid_query(msg)`
- `DbError::unsupported(msg)`
- `DbError::corruption(err)`
- `DbError::transaction(err)`

Implements `Display`, `Error`, and `From<std::io::Error>`.

---

## Types

### Integer

Wrapper around `i64` with checked arithmetic.

```rust
pub struct Integer(i64);

impl Integer {
    pub const fn new(value: i64) -> Self;
    pub const fn as_i64(self) -> i64;
    pub const fn checked_add(self, rhs: Self) -> Option<Self>;
    pub const fn checked_sub(self, rhs: Self) -> Option<Self>;
    pub const fn checked_mul(self, rhs: Self) -> Option<Self>;
    pub const fn checked_div(self, rhs: Self) -> Option<Self>;
    pub const fn to_le_bytes(self) -> [u8; 8];
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self;
}
```

Implements `Display`, `From<i64>`, `TryFrom<&str>`.

### Float

Wrapper around `f64` that guarantees finiteness.

```rust
pub struct Float(f64);

impl Float {
    pub fn try_new(value: f64) -> Result<Self, DbError>;
    pub const fn as_f64(self) -> f64;
    pub fn total_cmp(&self, other: &Self) -> Ordering;
    pub fn try_from_le_bytes(bytes: [u8; 8]) -> Result<Self, DbError>;
    pub const fn to_le_bytes(self) -> [u8; 8];
}
```

Implements `Display`, `TryFrom<f64>`, `TryFrom<&str>`.

### Text

Wrapper around `String`.

```rust
pub struct Text(String);

impl Text {
    pub fn new(value: String) -> Self;
    pub fn as_str(&self) -> &str;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn to_lowercase(&self) -> Self;
    pub fn to_uppercase(&self) -> Self;
    pub fn contains_ignore_case(&self, needle: &str) -> bool;
    pub fn as_bytes(&self) -> &[u8];
}
```

Implements `Display`, `From<String>`, `From<&str>`, `AsRef<str>`.

### Blob

Wrapper around `Vec<u8>`.

```rust
pub struct Blob(Vec<u8>);

impl Blob {
    pub fn new(value: Vec<u8>) -> Self;
    pub fn as_slice(&self) -> &[u8];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Implements `Display`, `From<Vec<u8>>`, `From<&[u8]>`, `AsRef<[u8]>`.

---

## Full API Reference

### `core::catalog::Catalog`

```rust
pub struct Catalog { /* private */ }

impl Catalog {
    pub fn new() -> Self;
    pub fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    pub fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    pub fn replace_table(&mut self, name: &str, table: Table) -> Result<(), DbError>;
    pub fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
    pub fn get_table(&self, name: &str) -> Option<&Table>;
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
    pub fn tables(&self) -> impl Iterator<Item = (&str, &Table)>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### `core::table::Table`

```rust
pub struct Table { /* private */ }

impl Table {
    pub fn new(schema: TableSchema) -> Self;
    pub fn insert(&mut self, row: Row) -> Result<(), DbError>;
    pub fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>;
    pub fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    pub fn set_column_allowed_values(
        &mut self,
        col_idx: usize,
        values: Option<Vec<Value>>,
    ) -> Result<(), DbError>;
    pub fn lookup_by_unique(&self, col_idx: usize, value: &Value) -> Option<&Row>;
    pub fn schema(&self) -> &TableSchema;
    pub fn rows(&self) -> &[Row];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, index: usize) -> Option<&Row>;
    pub fn rename_schema(&mut self, new_name: &str) -> Result<(), DbError>;
    pub const fn is_read_only(&self) -> bool;
    pub fn set_read_only(&mut self, value: bool);
}
```

### `core::row::Row`

```rust
pub struct Row { /* private */ }

impl Row {
    pub fn new(values: Vec<Value>) -> Self;
    pub fn values(&self) -> &[Value];
    pub fn get(&self, index: usize) -> Option<&Value>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn values_mut(&mut self) -> &mut Vec<Value>;
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value>;
}
```

### `core::value::Value`

```rust
pub enum Value {
    Null,
    Integer(Integer),
    Float(Float),
    Text(Text),
    Blob(Blob),
    Boolean(bool),
    Formula(String),
}

impl Value {
    pub fn is_null(&self) -> bool;
    pub fn is_integer(&self) -> bool;
    pub fn is_float(&self) -> bool;
    pub fn is_text(&self) -> bool;
    pub fn is_blob(&self) -> bool;
    pub fn is_boolean(&self) -> bool;
    pub fn is_formula(&self) -> bool;
    pub fn type_name(&self) -> &'static str;
    pub fn as_integer(&self) -> Option<&Integer>;
    pub fn as_float(&self) -> Option<&Float>;
    pub fn as_text(&self) -> Option<&Text>;
    pub fn as_blob(&self) -> Option<&Blob>;
    pub fn as_boolean(&self) -> Option<bool>;
    pub fn as_formula(&self) -> Option<&str>;
    pub fn into_integer(self) -> Option<Integer>;
    pub fn into_float(self) -> Option<Float>;
    pub fn into_text(self) -> Option<Text>;
    pub fn into_blob(self) -> Option<Blob>;
    pub fn into_boolean(self) -> Option<bool>;
    pub fn into_formula(self) -> Option<String>;
}
```

### `core::schema::column`

```rust
pub enum DataType { Null, Integer, Float, Text, Blob }
impl DataType {
    pub const fn as_str(&self) -> &'static str;
}
impl Display for DataType;

pub enum ComparisonOp { Eq, NotEq, Lt, Lte, Gt, Gte }

pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}

pub struct ForeignKey {
    pub table: String,
    pub column: String,
}

pub struct ColumnDef { /* private */ }
impl ColumnDef {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self;
    pub fn name(&self) -> &str;
    pub const fn data_type(&self) -> &DataType;
    pub const fn is_nullable(&self) -> bool;
    pub const fn is_primary_key(&self) -> bool;
    pub const fn is_unique(&self) -> bool;
    pub const fn default_value(&self) -> Option<&Value>;
    pub const fn check_constraint(&self) -> Option<&CheckConstraint>;
    pub const fn foreign_key(&self) -> Option<&ForeignKey>;
    pub const fn allowed_values(&self) -> Option<&Vec<Value>>;
    pub fn set_nullable(&mut self, value: bool);
    pub fn set_primary_key(&mut self, value: bool);
    pub fn set_unique(&mut self, value: bool);
    pub fn set_default(&mut self, value: Option<Value>);
    pub fn set_check(&mut self, constraint: Option<CheckConstraint>);
    pub fn set_foreign_key(&mut self, fk: Option<ForeignKey>);
    pub fn set_allowed_values(&mut self, values: Option<Vec<Value>>);
    pub fn validate_value(&self, value: &Value) -> Result<(), DbError>;
}
```

### `core::schema::table_schema`

```rust
pub struct TableSchema { /* private */ }
impl TableSchema {
    pub fn try_new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Result<Self, DbError>;
    pub fn name(&self) -> &str;
    pub fn columns(&self) -> &[ColumnDef];
    pub fn get_column_mut(&mut self, index: usize) -> Option<&mut ColumnDef>;
    pub fn column_index(&self, name: &str) -> Option<usize>;
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef>;
    pub fn validate_values(&self, values: &[Value]) -> Result<(), DbError>;
}
```

### `store::storage`

```rust
pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
}

pub struct FileStorage { /* private */ }
impl FileStorage {
    pub fn open(path: &Path) -> Result<Self, DbError>;
    pub fn sync(&mut self) -> Result<(), DbError>;
    pub fn checkpoint(&mut self) -> Result<(), DbError>;
    pub fn reload_from_disk(&mut self) -> Result<Catalog, DbError>;
    pub fn close(self) -> Result<(), DbError>;
}
impl StorageEngine for FileStorage;
impl Drop for FileStorage;

pub struct InMemoryStorage { /* private */ }
impl InMemoryStorage {
    pub fn new() -> Self;
}
impl StorageEngine for InMemoryStorage;
```

### `store::wal`

```rust
pub struct Wal { /* private */ }
impl Wal {
    pub fn open(path: &Path) -> Result<Self, DbError>;
    pub fn append(&mut self, payload: &[u8]) -> Result<(), DbError>;
    pub fn sync(&self) -> Result<(), DbError>;
    pub fn read_all(&mut self) -> Result<Vec<Vec<u8>>, DbError>;
    pub fn truncate(&mut self) -> Result<(), DbError>;
    pub fn unlock(&mut self) -> Result<(), DbError>;
}
impl Drop for Wal;
```

### `store::append_log`

```rust
pub fn append_record(file: &mut File, payload: &[u8]) -> Result<(), DbError>;
pub fn read_records(file: &mut File) -> Result<Vec<Vec<u8>>, DbError>;
```

### `store::file`

```rust
pub fn open_or_create(path: &Path) -> Result<File, DbError>;
pub fn read_file(path: &Path) -> Result<Vec<u8>, DbError>;
pub fn write_all_atomic(path: &Path, data: &[u8]) -> Result<(), DbError>;
pub fn append_to_file(file: &mut File, data: &[u8]) -> Result<(), DbError>;
pub fn sync_file(file: &File) -> Result<(), DbError>;
```

### `store::recovery`

```rust
pub struct RecoveryResult { pub records: Vec<Vec<u8>> }
pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError>;
```

### `store::serialize`

Public encode/decode functions for catalog, table, schema, row, column, and value. See source for exact signatures.

### `error::DbError`

```rust
pub enum DbError { ... }
impl DbError {
    pub fn table_not_found(name: impl Into<String>) -> Self;
    pub fn column_not_found(name: impl Into<String>) -> Self;
    pub fn type_mismatch(msg: impl Into<String>) -> Self;
    pub fn invalid_operation(msg: impl Into<String>) -> Self;
    pub fn invalid_query(msg: impl Into<String>) -> Self;
    pub fn unsupported(msg: impl Into<String>) -> Self;
    pub fn corruption<E>(err: E) -> Self where E: Error + Send + Sync + 'static;
    pub fn transaction<E>(err: E) -> Self where E: Error + Send + Sync + 'static;
}
impl Display for DbError;
impl Error for DbError;
impl From<std::io::Error> for DbError;
```

### `types::integer`

```rust
pub struct Integer(i64);
impl Integer {
    pub const fn new(value: i64) -> Self;
    pub const fn as_i64(self) -> i64;
    pub const fn checked_add(self, rhs: Self) -> Option<Self>;
    pub const fn checked_sub(self, rhs: Self) -> Option<Self>;
    pub const fn checked_mul(self, rhs: Self) -> Option<Self>;
    pub const fn checked_div(self, rhs: Self) -> Option<Self>;
    pub const fn to_le_bytes(self) -> [u8; 8];
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self;
}
```

### `types::float`

```rust
pub struct Float(f64);
impl Float {
    pub fn try_new(value: f64) -> Result<Self, DbError>;
    pub const fn as_f64(self) -> f64;
    pub fn total_cmp(&self, other: &Self) -> Ordering;
    pub fn try_from_le_bytes(bytes: [u8; 8]) -> Result<Self, DbError>;
    pub const fn to_le_bytes(self) -> [u8; 8];
}
```

### `types::text`

```rust
pub struct Text(String);
impl Text {
    pub fn new(value: String) -> Self;
    pub fn as_str(&self) -> &str;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn to_lowercase(&self) -> Self;
    pub fn to_uppercase(&self) -> Self;
    pub fn contains_ignore_case(&self, needle: &str) -> bool;
    pub fn as_bytes(&self) -> &[u8];
}
```

### `types::blob`

```rust
pub struct Blob(Vec<u8>);
impl Blob {
    pub fn new(value: Vec<u8>) -> Self;
    pub fn as_slice(&self) -> &[u8];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

---

## Examples

### In-Memory Database

```rust
use monumentum_db::{
    core::{catalog::Catalog, row::Row, schema::column::{ColumnDef, DataType},
            schema::table_schema::TableSchema, value::Value},
    store::storage::{InMemoryStorage, StorageEngine},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;

    let mut catalog = Catalog::new();
    catalog.create_table(schema)?;

    if let Some(table) = catalog.get_table_mut("users") {
        table.insert(Row::new(vec![
            Value::from(1_i64),
            Value::from("Alice"),
        ]))?;
    }

    let mut storage = InMemoryStorage::new();
    storage.save_catalog(&catalog)?;
    let loaded = storage.load_catalog()?;

    assert_eq!(loaded.get_table("users").unwrap().len(), 1);
    Ok(())
}
```

### File-Based Database

```rust
use monumentum_db::{
    core::{catalog::Catalog, schema::column::{ColumnDef, DataType},
            schema::table_schema::TableSchema},
    store::storage::{FileStorage, StorageEngine},
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("mydb.monumentum");
    let mut storage = FileStorage::open(path)?;

    let schema = TableSchema::try_new(
        "items",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("description", DataType::Text),
        ],
    )?;

    let mut catalog = Catalog::new();
    catalog.create_table(schema)?;

    storage.save_catalog(&catalog)?;
    storage.sync()?;
    storage.checkpoint()?;
    storage.close()?;

    Ok(())
}
```

---

## Testing

Run the full test suite:

```bash
cargo test --workspace --all-targets --all-features
```

Run clippy with warnings denied:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## Security

- **No unsafe code** – all memory safety guarantees are maintained.
- **WAL & snapshot integrity** – CRC32 checksums and version fields detect corruption.
- **Atomic snapshots** – temp file + rename prevents partial writes.
- **File permissions** – files are created with `0600`.
- **Symlink attack prevention** – `O_NOFOLLOW` and `O_EXCL` are used on Unix.
- **Resource limits** – max columns (1024), max rows (10,000,000), max record size (64 MiB), max snapshot size (256 MiB).
- **Input validation** – all identifiers and values are validated; no raw string queries.

---

## Limitations

- No SQL or query language yet (see `monumentum_query`).
- Single-threaded; users must provide synchronization for shared access.
- Full catalog serialization may not scale to extremely large datasets.
- File locking is a no-op on non-Unix platforms.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
