# Monumentum DB

## Overview

**Monumentum DB** is a lightweight, embedded database engine written in Rust. It provides a strongly typed, transactional (via WAL) storage layer with support for both in-memory and file-based persistence. The engine is designed with a focus on data integrity, explicit constraint enforcement, and resistance to common storage corruption and injection vectors.

The project is part of a workspace containing two crates:

- `monumentum_db` – the core storage engine (documented here)
- `monumentum_query` – a future query layer that will sit on top of the storage engine (currently under development)

**Repository:** [https://github.com/mroczect/monumentum](https://github.com/mroczect/monumentum)

---

## Design Goals

- **Safety** – No `unsafe` code, all errors are explicit via `DbError`.
- **Durability** – Write‑ahead logging (WAL) with CRC32 checksums and atomic snapshot replacement.
- **Integrity** – Constraints (primary key, unique, null, default, check, foreign key) are enforced at the storage layer.
- **Portability** – Standard library only; platform‑specific features (file locking, `/dev/urandom`) have graceful fallbacks.
- **Clarity** – Modular architecture with a small, well‑documented public API.

---

## Architecture

The crate is organised into three core modules:

### `core`

Defines the in‑memory data model:

- `Catalog` – owns all tables in an ordered map.
- `Table` – holds a schema, rows, and optional unique indexes.
- `Row` – a list of `Value`s.
- `Value` – a strongly typed enum for scalar data.
- `schema::ColumnDef` – describes a column’s name, data type, and constraints.
- `schema::TableSchema` – validates a collection of column definitions.
- `index::HashIndex` – internal hash map for unique constraint enforcement.

### `store`

Handles persistence:

- `StorageEngine` – trait abstracting load/save operations.
- `FileStorage` – durable storage using a snapshot file plus a write‑ahead log.
- `InMemoryStorage` – transient storage for tests or caching.
- `wal::Wal` – manages the append‑only log with file locking.
- `append_log` – low‑level record format with CRC32 checksums.
- `serialize` – binary serialization/deserialization for all data structures.
- `file` – safe file operations (atomic write, random temp names, no‑follow flags).
- `recovery` – utilities for recovering from WAL.

### `types`

Thin, safe wrappers around primitive types:

- `Integer` – checked arithmetic.
- `Float` – finite‑only floating point.
- `Text` – UTF‑8 string with case‑insensitive helpers.
- `Blob` – byte array.

---

## Data Model

### `Value`

Enum with variants:

```rust
pub enum Value {
    Null,
    Integer(Integer),
    Float(Float),
    Text(Text),
    Blob(Blob),
}
```

All non‑null variants hold a corresponding `types` wrapper. The `Value` type implements `Display`, `Default`, `From` conversions for `()`, `i64`, `String`, `&str`, `Vec<u8>`, `&[u8]`, and `TryFrom<f64>`. It also provides accessor methods (`as_integer`, `as_float`, etc.) and ownership‑taking methods (`into_integer`, etc.).

### `Row`

A `Row` is a simple wrapper around `Vec<Value>`. It ensures value order and provides indexing, length, and emptiness checks. Rows are validated against a schema before insertion.

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
}
```

- `DataType` enum: `Null`, `Integer`, `Float`, `Text`, `Blob`.
- `ComparisonOp` enum: `Eq`, `NotEq`, `Lt`, `Lte`, `Gt`, `Gte`.
- `CheckConstraint` struct: column name, operator, comparison value.
- `ForeignKey` struct: referenced table and column.

### `TableSchema`

A schema contains a table name and a list of column definitions. It validates:

- Table name is non‑empty.
- At least one column exists.
- Maximum 1024 columns.
- Column names are non‑empty and unique (case‑insensitive).
- Values passed to `validate_values` match the column count, data types, nullability, and check constraints.

### `Table`

A table holds a schema, a vector of rows, and optional `HashIndex` instances for columns marked `unique` or `primary_key`. Methods include:

- `new(schema)`
- `insert(row)` – validates, applies defaults, checks duplicates, updates index.
- `replace_rows(rows)` – validates all rows, checks duplicates, rebuilds indexes.
- `lookup_by_unique(col_idx, value)` – fast lookup via index (falls back to linear scan).
- `schema()`, `rows()`, `len()`, `is_empty()`, `get(index)`.

### `Catalog`

An ordered map of table names to `Table` objects. Provides:

- `create_table(schema)`
- `drop_table(name)`
- `get_table(name)`, `get_table_mut(name)`
- `tables()` iterator
- `len()`, `is_empty()`

---

## Storage Layer

### `StorageEngine` Trait

```rust
pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
}
```

Implementations:

- `InMemoryStorage` – stores a `Catalog` directly in memory. `save_catalog` simply clones and stores.
- `FileStorage` – persists via snapshot and WAL.

### `FileStorage`

On `open(path)`:

1. Opens (or creates) the WAL file at `path.with_extension("wal")` and locks it.
2. If the snapshot file exists, reads and deserializes it, applying a 256 MB size limit.
3. Reads all WAL records, deserializes each snapshot, and picks the one with the highest sequence number.
4. Returns a `FileStorage` instance ready for operations.

`save_catalog` increments the sequence number (checked addition), appends the serialized snapshot to the WAL, and updates the in‑memory catalog.

`checkpoint` atomically writes the current catalog to the snapshot file and truncates the WAL.

`close` explicitly unlocks the WAL file.

### WAL and Append Log

- The WAL uses a custom record format: `[magic u32][version u32][length u64][checksum u32][payload]`.
- CRC32 checksum verifies payload integrity.
- Maximum record size: 64 MB.
- `append_record` and `read_records` are available directly from the `append_log` module.

### Serialization

The `serialize` module provides functions to encode/decode:

- `encode_catalog`, `decode_catalog`
- `encode_table`, `decode_table`
- `encode_row`, `decode_row`
- `encode_column_def`, `decode_column_def`
- `encode_table_schema`, `decode_table_schema`
- `encode_value`, `decode_value`

All encoding uses little‑endian primitive values with length prefixes. Decoding enforces maximum lengths and returns `DbError::Corruption` for invalid data.

### File Operations

The `file` module exposes:

- `open_or_create(path)` – opens with `O_NOFOLLOW`, mode `0600`.
- `read_file(path)`
- `write_all_atomic(path, data)` – writes to a temp file with `O_EXCL`/`O_NOFOLLOW`, fsyncs, renames, fsyncs directory.
- `append_to_file(file, data)`
- `sync_file(file)`

Temp file names are generated using 16 random bytes from `/dev/urandom` (if available), falling back to a pseudo‑random fallback.

---

## Error Handling

All fallible functions return `Result<_, DbError>`.

`DbError` is an enum:

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

Constructors are provided (e.g., `DbError::table_not_found("users")`). The error implements `Display`, `Error`, and `From<std::io::Error>`.

---

## Types

### `Integer`

Wrapper around `i64` with:

- `new(i64)`
- `as_i64()`
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div`
- `to_le_bytes`, `from_le_bytes`
- `Display`, `From<i64>`, `TryFrom<&str>`

### `Float`

Wrapper around `f64` that guarantees finiteness.

- `try_new(f64) -> Result<Float, DbError>`
- `as_f64()`
- `total_cmp(&self, &Float) -> Ordering`
- `try_from_le_bytes`, `to_le_bytes`
- `Display`, `TryFrom<f64>`, `TryFrom<&str>`

### `Text`

Wrapper around `String`.

- `new(String)`
- `as_str()`
- `len()`, `is_empty()`
- `to_lowercase()`, `to_uppercase()`
- `contains_ignore_case(needle)`
- `as_bytes()`
- `Display`, `From<String>`, `From<&str>`, `AsRef<str>`

### `Blob`

Wrapper around `Vec<u8>`.

- `new(Vec<u8>)`
- `as_slice()`
- `len()`, `is_empty()`
- `Display`, `From<Vec<u8>>`, `From<&[u8]>`, `AsRef<[u8]>`

---

## Full API Reference

### `core::catalog::Catalog`

```rust
pub struct Catalog { /* private */ }

impl Catalog {
    pub fn new() -> Self;
    pub fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    pub fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
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
    pub fn lookup_by_unique(&self, col_idx: usize, value: &Value) -> Option<&Row>;
    pub fn schema(&self) -> &TableSchema;
    pub fn rows(&self) -> &[Row];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, index: usize) -> Option<&Row>;
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
}
```

### `core::value::Value`

```rust
pub enum Value { Null, Integer(Integer), Float(Float), Text(Text), Blob(Blob) }

impl Value {
    pub fn is_null(&self) -> bool;
    pub fn is_integer(&self) -> bool;
    pub fn is_float(&self) -> bool;
    pub fn is_text(&self) -> bool;
    pub fn is_blob(&self) -> bool;
    pub fn type_name(&self) -> &'static str;
    pub fn as_integer(&self) -> Option<&Integer>;
    pub fn as_float(&self) -> Option<&Float>;
    pub fn as_text(&self) -> Option<&Text>;
    pub fn as_blob(&self) -> Option<&Blob>;
    pub fn into_integer(self) -> Option<Integer>;
    pub fn into_float(self) -> Option<Float>;
    pub fn into_text(self) -> Option<Text>;
    pub fn into_blob(self) -> Option<Blob>;
}

// Display, Default, From<()>, From<Integer>, From<Float>, From<Text>, From<Blob>,
// From<i64>, From<String>, From<&str>, From<Vec<u8>>, From<&[u8]>, TryFrom<f64>
```

### `core::schema::column`

```rust
pub enum DataType { Null, Integer, Float, Text, Blob }
impl DataType { pub const fn as_str(&self) -> &'static str; }
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
    pub fn set_nullable(&mut self, value: bool);
    pub fn set_primary_key(&mut self, value: bool);
    pub fn set_unique(&mut self, value: bool);
    pub fn set_default(&mut self, value: Option<Value>);
    pub fn set_check(&mut self, constraint: Option<CheckConstraint>);
    pub fn set_foreign_key(&mut self, fk: Option<ForeignKey>);
}
```

### `core::schema::table_schema`

```rust
pub struct TableSchema { /* private */ }
impl TableSchema {
    pub fn try_new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Result<Self, DbError>;
    pub fn name(&self) -> &str;
    pub fn columns(&self) -> &[ColumnDef];
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
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
}

pub struct FileStorage { /* private */ }
impl FileStorage {
    pub fn open(path: &Path) -> Result<Self, DbError>;
    pub fn sync(&mut self) -> Result<(), DbError>;
    pub fn checkpoint(&mut self) -> Result<(), DbError>;
    pub fn close(self) -> Result<(), DbError>;
}
impl StorageEngine for FileStorage;
impl Drop for FileStorage;

pub struct InMemoryStorage { /* private */ }
impl InMemoryStorage { pub fn new() -> Self; }
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

Public encode/decode functions for catalog, table, schema, row, column, and value. (See source for exact signatures.)

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
impl Display, From<i64>, TryFrom<&str>;
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
impl Display, TryFrom<f64>, TryFrom<&str>;
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
impl Display, From<String>, From<&str>, AsRef<str>;
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
impl Display, From<Vec<u8>>, From<&[u8]>, AsRef<[u8]>;
```

---

## Getting Started

### Prerequisites

- Rust stable (1.60+ recommended)
- Cargo
- Unix-like OS for full file locking; non-Unix platforms have graceful degradation.

### Installation

Add the dependency via Git:

```toml
[dependencies]
monumentum_db = { git = "https://github.com/mroczect/monumentum" }
```

### Usage Example: In‑Memory

```rust
use monumentum_db::{
    core::{catalog::Catalog, schema::column::{ColumnDef, DataType}, schema::table_schema::TableSchema, row::Row, value::Value},
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

    // Insert a row
    let table = catalog.get_table_mut("users").unwrap();
    table.insert(Row::new(vec![Value::from(1_i64), Value::from("Alice")]))?;

    let mut storage = InMemoryStorage::new();
    storage.save_catalog(&catalog)?;
    let loaded = storage.load_catalog()?;
    assert_eq!(loaded.get_table("users").unwrap().len(), 1);
    Ok(())
}
```

### Usage Example: File‑Based

```rust
use monumentum_db::{
    core::{catalog::Catalog, schema::column::{ColumnDef, DataType}, schema::table_schema::TableSchema},
    store::storage::{FileStorage, StorageEngine},
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("mydb");
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

    // Later, to reduce WAL size
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

The suite includes 299 integration tests covering core logic, serialization round‑trips, storage recovery, error handling, and injection resistance.

---

## Security Considerations

- **WAL and snapshot integrity** – CRC32 checksums and version fields detect corruption.
- **Atomic snapshots** – temp file + rename prevents partial writes.
- **File permissions** – database files are created with `0600`.
- **Symlink attack prevention** – `O_NOFOLLOW` and `O_EXCL` are used on Unix.
- **Resource limits** – maximum column count, row count, record size, and snapshot size.
- **Input validation** – all identifiers and values are validated; no raw string queries.

---

## Limitations

- No SQL or query language yet (see `monumentum_query`).
- Single‑threaded; users must provide synchronization for shared access.
- Best performance on Linux/Unix; locking is a no‑op elsewhere.
- Full catalog serialization may not scale to extremely large datasets.

---

## Contributing

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Ensure all tests pass.
4. Follow Rust idiomatic style and document public API changes.
5. Open an issue for discussion before adding significant features.

---

## License

```txt
The MIT License (MIT)

Copyright (c) 2026 mroczect

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```
