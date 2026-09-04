# Monumentum DB API Documentation

This document provides a complete reference for the `monumentum_db` crate. It covers all public types, traits, functions, and modules exposed by the crate. The crate offers in‑memory and file‑backed table storage, schema management, serialization, and error handling.

---

## Table of Contents

1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [Core Module](#core-module)
   - [`Catalog`](#catalog)
   - [`Table`](#table)
   - [`Row`](#row)
   - [`Value`](#value)
   - [`ColumnDef`](#columndef)
   - [`DataType`](#datatype)
   - [`ComparisonOp`](#comparisonop)
   - [`CheckConstraint`](#checkconstraint)
   - [`ForeignKey`](#foreignkey)
   - [`TableSchema`](#tableschema)
   - [`Column` Trait](#column-trait)
   - [`ColumnIndex` Trait](#columnindex-trait)
4. [Error Module](#error-module)
   - [`ErrorKind`](#errorkind)
   - [`DbError`](#dberror)
   - [`MonumentumError` Trait](#monumentumerror-trait)
5. [Store Module](#store-module)
   - [`StorageEngine` Trait](#storageengine-trait)
   - [`FileStorage`](#filestorage)
   - [`InMemoryStorage`](#inmemorystorage)
   - [`Wal`](#wal)
   - [`RecoveryResult`](#recoveryresult)
   - Functions
     - [`recover_wal`](#recover_wal)
     - [`open_or_create`](#open_or_create)
     - [`read_file`](#read_file)
     - [`write_all_atomic`](#write_all_atomic)
     - [`append_to_file`](#append_to_file)
     - [`sync_file`](#sync_file)
     - [`append_record`](#append_record)
     - [`read_records`](#read_records)
     - [`encode_catalog`](#encode_catalog)
     - [`decode_catalog`](#decode_catalog)
6. [Types Module](#types-module)
   - [`Blob`](#blob)
   - [`Float`](#float)
   - [`Integer`](#integer)
   - [`Text`](#text)
7. [Trait Implementations Summary](#trait-implementations-summary)

---

## Overview

`monumentum_db` provides a lightweight database engine for spreadsheet‑like applications. It supports:

- In‑memory and file‑based storage.
- Tables with schemas, constraints, unique indexes, and read‑only protection.
- Serialization to/from a binary format with checksums.
- Write‑ahead logging (WAL) and snapshot checkpoints.
- Error handling with kind and source information.

The crate is designed to be embedded in larger systems such as the `monumentum_workbook` crate.

---

## Module Structure

The crate is organized into four top‑level modules, all re‑exported at the crate root:

- `core` – data structures for tables, rows, values, and schemas.
- `error` – error types and traits.
- `store` – storage engines, file handling, and serialization.
- `types` – wrapper types for primitive values (integer, float, text, blob).

```rust
pub mod core;
pub mod error;
pub mod store;
pub mod types;

pub use core::*;
pub use error::*;
pub use store::*;
pub use types::*;
```

---

## Core Module

### `Catalog`

A collection of named tables, stored in a `BTreeMap`. It provides methods to create, drop, replace, and rename tables, and to iterate over them.

```rust
pub struct Catalog { /* private fields */ }
```

#### Methods

| Method          | Signature                                                                               | Description                                                                               |
| --------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `new`           | `pub fn new() -> Self`                                                                  | Creates an empty catalog.                                                                 |
| `create_table`  | `pub fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>`            | Adds a new table from a schema. Errors if the table name is empty or already exists.      |
| `drop_table`    | `pub fn drop_table(&mut self, name: &str) -> Result<(), DbError>`                       | Removes a table by name. Returns `DbError::TableNotFound` if the table does not exist.    |
| `replace_table` | `pub fn replace_table(&mut self, name: &str, table: Table) -> Result<(), DbError>`      | Replaces an existing table with a new one. The new table’s schema name must match `name`. |
| `get_table`     | `pub fn get_table(&self, name: &str) -> Option<&Table>`                                 | Returns a reference to a table if it exists.                                              |
| `get_table_mut` | `pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>`                     | Returns a mutable reference to a table.                                                   |
| `tables`        | `pub fn tables(&self) -> impl Iterator<Item = (&str, &Table)>`                          | Returns an iterator over all tables as `(name, table)` pairs.                             |
| `len`           | `pub fn len(&self) -> usize`                                                            | Returns the number of tables.                                                             |
| `is_empty`      | `pub fn is_empty(&self) -> bool`                                                        | Returns `true` if no tables exist.                                                        |
| `rename_table`  | `pub fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>` | Renames a table. If `old_name == new_name`, the operation is a no‑op.                     |

#### Trait Implementations

- `Debug`
- `Default`
- `Clone`
- `PartialEq`

---

### `Table`

Represents a table with a schema, rows, and optional unique indexes. Tables can be marked read‑only.

```rust
pub struct Table { /* private fields */ }
```

#### Methods

| Method                      | Signature                                                                                                        | Description                                                                                   |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `new`                       | `pub fn new(schema: TableSchema) -> Self`                                                                        | Creates an empty table from a schema.                                                         |
| `rename_schema`             | `pub fn rename_schema(&mut self, new_name: &str) -> Result<(), DbError>`                                         | Renames the table’s schema.                                                                   |
| `schema`                    | `pub fn schema(&self) -> &TableSchema`                                                                           | Returns a reference to the table’s schema.                                                    |
| `rows`                      | `pub fn rows(&self) -> &[Row]`                                                                                   | Returns all rows as a slice.                                                                  |
| `insert`                    | `pub fn insert(&mut self, row: Row) -> Result<(), DbError>`                                                      | Inserts a row, validating against the schema, applying defaults, and updating unique indexes. |
| `len`                       | `pub fn len(&self) -> usize`                                                                                     | Returns the number of rows.                                                                   |
| `is_empty`                  | `pub fn is_empty(&self) -> bool`                                                                                 | Returns `true` if the table has no rows.                                                      |
| `get`                       | `pub fn get(&self, index: usize) -> Option<&Row>`                                                                | Returns a reference to a row by index.                                                        |
| `replace_rows`              | `pub fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>`                                          | Replaces all rows, validating each and rebuilding indexes.                                    |
| `set_cell`                  | `pub fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>`                | Sets a single cell’s value, updating indexes and validating constraints.                      |
| `set_column_allowed_values` | `pub fn set_column_allowed_values(&mut self, col_idx: usize, values: Option<Vec<Value>>) -> Result<(), DbError>` | Updates the allowed values for a column, validating existing rows.                            |
| `lookup_by_unique`          | `pub fn lookup_by_unique(&self, col_idx: usize, value: &Value) -> Option<&Row>`                                  | Looks up a row by a unique column value, using the index if available.                        |
| `is_read_only`              | `pub const fn is_read_only(&self) -> bool`                                                                       | Returns `true` if the table is marked read‑only.                                              |
| `set_read_only`             | `pub fn set_read_only(&mut self, value: bool)`                                                                   | Sets or clears the read‑only flag.                                                            |
| `get_column_by_name`        | `pub fn get_column_by_name(&self, name: &str) -> Option<&ColumnDef>`                                             | Returns a column definition by name (case‑insensitive).                                       |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`

---

### `Row`

A single row containing a vector of `Value`s.

```rust
pub struct Row { /* private fields */ }
```

#### Methods

| Method       | Signature                                                                     | Description                                              |
| ------------ | ----------------------------------------------------------------------------- | -------------------------------------------------------- |
| `new`        | `pub fn new(values: Vec<Value>) -> Self`                                      | Creates a new row from a vector of values.               |
| `values`     | `pub fn values(&self) -> &[Value]`                                            | Returns the values as a slice.                           |
| `get`        | `pub fn get<I>(&self, index: I) -> Option<&Value> where I: ColumnIndex<Self>` | Returns a value by column index (usize or `&str`).       |
| `len`        | `pub fn len(&self) -> usize`                                                  | Returns the number of values.                            |
| `is_empty`   | `pub fn is_empty(&self) -> bool`                                              | Returns `true` if there are no values.                   |
| `values_mut` | `pub fn values_mut(&mut self) -> &mut Vec<Value>`                             | Returns a mutable reference to the values.               |
| `get_mut`    | `pub fn get_mut(&mut self, index: usize) -> Option<&mut Value>`               | Returns a mutable reference to a value by numeric index. |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `PartialOrd`

---

### `Value`

A dynamically typed value that can hold `Null`, `Integer`, `Float`, `Text`, `Blob`, `Boolean`, or `Formula`.

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

#### Methods

| Method         | Signature                                      | Description                                                      |
| -------------- | ---------------------------------------------- | ---------------------------------------------------------------- |
| `is_null`      | `pub fn is_null(&self) -> bool`                | Returns `true` if the value is `Null`.                           |
| `is_integer`   | `pub fn is_integer(&self) -> bool`             | Returns `true` if the value is `Integer`.                        |
| `is_float`     | `pub fn is_float(&self) -> bool`               | Returns `true` if the value is `Float`.                          |
| `is_text`      | `pub fn is_text(&self) -> bool`                | Returns `true` if the value is `Text`.                           |
| `is_blob`      | `pub fn is_blob(&self) -> bool`                | Returns `true` if the value is `Blob`.                           |
| `is_boolean`   | `pub fn is_boolean(&self) -> bool`             | Returns `true` if the value is `Boolean`.                        |
| `is_formula`   | `pub fn is_formula(&self) -> bool`             | Returns `true` if the value is `Formula`.                        |
| `type_name`    | `pub fn type_name(&self) -> &'static str`      | Returns the type name as a string.                               |
| `as_integer`   | `pub fn as_integer(&self) -> Option<&Integer>` | Returns a reference to the integer if applicable.                |
| `as_float`     | `pub fn as_float(&self) -> Option<&Float>`     | Returns a reference to the float if applicable.                  |
| `as_text`      | `pub fn as_text(&self) -> Option<&Text>`       | Returns a reference to the text if applicable.                   |
| `as_blob`      | `pub fn as_blob(&self) -> Option<&Blob>`       | Returns a reference to the blob if applicable.                   |
| `as_boolean`   | `pub fn as_boolean(&self) -> Option<bool>`     | Returns the boolean if applicable.                               |
| `as_formula`   | `pub fn as_formula(&self) -> Option<&str>`     | Returns the formula string if applicable.                        |
| `into_integer` | `pub fn into_integer(self) -> Option<Integer>` | Consumes the value and returns the integer if applicable.        |
| `into_float`   | `pub fn into_float(self) -> Option<Float>`     | Consumes the value and returns the float if applicable.          |
| `into_text`    | `pub fn into_text(self) -> Option<Text>`       | Consumes the value and returns the text if applicable.           |
| `into_blob`    | `pub fn into_blob(self) -> Option<Blob>`       | Consumes the value and returns the blob if applicable.           |
| `into_boolean` | `pub fn into_boolean(self) -> Option<bool>`    | Consumes the value and returns the boolean if applicable.        |
| `into_formula` | `pub fn into_formula(self) -> Option<String>`  | Consumes the value and returns the formula string if applicable. |
| `as_i64`       | `pub fn as_i64(&self) -> Option<i64>`          | Returns the integer as `i64` if applicable.                      |
| `as_f64`       | `pub fn as_f64(&self) -> Option<f64>`          | Returns the float (or integer converted to f64) if applicable.   |
| `as_bool`      | `pub fn as_bool(&self) -> Option<bool>`        | Returns the boolean if applicable.                               |
| `as_str`       | `pub fn as_str(&self) -> Option<&str>`         | Returns the text as a string slice if applicable.                |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `PartialOrd`
- `Default` (defaults to `Value::Null`)
- `Display`
- `From<()>` → `Value::Null`
- `From<Integer>`, `From<Float>`, `From<Text>`, `From<Blob>`, `From<bool>`, `From<i64>`, `From<String>`, `From<&str>`, `From<Vec<u8>>`, `From<&[u8]>`
- `TryFrom<f64>` (errors if not finite)

---

### `ColumnDef`

Defines a column with name, data type, constraints, and optional default value.

```rust
pub struct ColumnDef { /* private fields */ }
```

#### Methods

| Method               | Signature                                                            | Description                                                                                     |
| -------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `new`                | `pub fn new(name: impl Into<String>, data_type: DataType) -> Self`   | Creates a new column with default flags (`nullable = true`, no primary key, no unique).         |
| `name`               | `pub fn name(&self) -> &str`                                         | Returns the column name.                                                                        |
| `data_type`          | `pub const fn data_type(&self) -> &DataType`                         | Returns the data type.                                                                          |
| `is_nullable`        | `pub const fn is_nullable(&self) -> bool`                            | Returns whether the column allows null values.                                                  |
| `is_primary_key`     | `pub const fn is_primary_key(&self) -> bool`                         | Returns whether the column is a primary key.                                                    |
| `is_unique`          | `pub const fn is_unique(&self) -> bool`                              | Returns whether the column enforces uniqueness.                                                 |
| `default_value`      | `pub const fn default_value(&self) -> Option<&Value>`                | Returns the default value if set.                                                               |
| `check_constraint`   | `pub const fn check_constraint(&self) -> Option<&CheckConstraint>`   | Returns the check constraint if present.                                                        |
| `foreign_key`        | `pub const fn foreign_key(&self) -> Option<&ForeignKey>`             | Returns the foreign key reference if present.                                                   |
| `allowed_values`     | `pub const fn allowed_values(&self) -> Option<&Vec<Value>>`          | Returns the allowed values list if set.                                                         |
| `set_nullable`       | `pub fn set_nullable(&mut self, value: bool)`                        | Sets the nullable flag. If set to `true`, primary key is cleared.                               |
| `set_primary_key`    | `pub fn set_primary_key(&mut self, value: bool)`                     | Sets the primary key flag. If `true`, sets `nullable = false` and `unique = true`.              |
| `set_unique`         | `pub fn set_unique(&mut self, value: bool)`                          | Sets the unique flag.                                                                           |
| `set_default`        | `pub fn set_default(&mut self, value: Option<Value>)`                | Sets or clears the default value.                                                               |
| `set_check`          | `pub fn set_check(&mut self, constraint: Option<CheckConstraint>)`   | Sets or clears the check constraint.                                                            |
| `set_foreign_key`    | `pub fn set_foreign_key(&mut self, fk: Option<ForeignKey>)`          | Sets or clears the foreign key.                                                                 |
| `set_allowed_values` | `pub fn set_allowed_values(&mut self, values: Option<Vec<Value>>)`   | Sets or clears the allowed values list.                                                         |
| `validate_value`     | `pub fn validate_value(&self, value: &Value) -> Result<(), DbError>` | Validates a value against the column’s type, nullability, check constraint, and allowed values. |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `Column` (trait)

---

### `DataType`

Enum representing possible column data types.

```rust
pub enum DataType {
    Null,
    Integer,
    Float,
    Text,
    Blob,
    Boolean,
}
```

#### Methods

| Method   | Signature                                    | Description                     |
| -------- | -------------------------------------------- | ------------------------------- |
| `as_str` | `pub const fn as_str(&self) -> &'static str` | Returns the SQL‑like type name. |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`
- `Display`

---

### `ComparisonOp`

Enum for comparison operators used in check constraints.

```rust
pub enum ComparisonOp {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}
```

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`

---

### `CheckConstraint`

Represents a check constraint on a column.

```rust
pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}
```

#### Fields

| Field    | Type           | Description                                  |
| -------- | -------------- | -------------------------------------------- |
| `column` | `String`       | Name of the column the constraint refers to. |
| `op`     | `ComparisonOp` | Comparison operator.                         |
| `value`  | `Value`        | Value to compare against.                    |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`

---

### `ForeignKey`

Represents a foreign key reference.

```rust
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}
```

#### Fields

| Field    | Type     | Description             |
| -------- | -------- | ----------------------- |
| `table`  | `String` | Referenced table name.  |
| `column` | `String` | Referenced column name. |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`

---

### `TableSchema`

Describes the structure of a table: its name and column definitions.

```rust
pub struct TableSchema { /* private fields */ }
```

#### Methods

| Method                | Signature                                                                                         | Description                                               |
| --------------------- | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `try_new`             | `pub fn try_new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Result<Self, DbError>`       | Creates a schema, validating name and column definitions. |
| `name`                | `pub fn name(&self) -> &str`                                                                      | Returns the table name.                                   |
| `columns`             | `pub fn columns(&self) -> &[ColumnDef]`                                                           | Returns the column definitions.                           |
| `get_column_mut`      | `pub fn get_column_mut(&mut self, index: usize) -> Option<&mut ColumnDef>`                        | Returns a mutable column by index.                        |
| `column_index`        | `pub fn column_index(&self, name: &str) -> Option<usize>`                                         | Returns the index of a column by name (case‑insensitive). |
| `get_column`          | `pub fn get_column(&self, name: &str) -> Option<&ColumnDef>`                                      | Returns a column reference by name.                       |
| `validate_values`     | `pub fn validate_values(&self, values: &[Value]) -> Result<(), DbError>`                          | Validates a slice of values against all columns.          |
| `get_column_by_index` | `pub fn get_column_by_index<I>(&self, index: I) -> Option<&ColumnDef> where I: ColumnIndex<Self>` | Returns a column by an index type (usize or `&str`).      |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`

---

### `Column` Trait

A trait implemented by types that act as column definitions.

```rust
pub trait Column {
    fn name(&self) -> &str;
    fn data_type(&self) -> &DataType;
    fn is_nullable(&self) -> bool;
    fn is_primary_key(&self) -> bool;
    fn is_unique(&self) -> bool;
}
```

`ColumnDef` implements this trait.

---

### `ColumnIndex` Trait

A trait for types that can be used to index into a container to obtain a column index.

```rust
pub trait ColumnIndex<T: ?Sized> {
    fn index(&self, container: &T) -> Result<usize, DbError>;
}
```

Implementations exist for:

- `usize` for `Row`, `TableSchema`, and `Table`.
- `&str` for `TableSchema` and `Table`.

---

## Error Module

### `ErrorKind`

An enum categorizing the kind of error.

```rust
pub enum ErrorKind {
    UniqueViolation,
    ForeignKeyViolation,
    NotNullViolation,
    CheckViolation,
    TypeMismatch,
    InvalidOperation,
    InvalidQuery,
    Io,
    Corruption,
    Unsupported,
    Other,
}
```

#### Trait Implementations

- `Debug`
- `Clone`
- `Copy`
- `PartialEq`
- `Eq`

---

### `DbError`

The main error type for database operations.

```rust
pub enum DbError {
    Io(std::io::Error),
    Corruption(Arc<dyn Error + Send + Sync>),
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    InvalidOperation(String),
    InvalidQuery(String),
    Transaction(Arc<dyn Error + Send + Sync>),
    Unsupported(String),
    ConstraintViolation {
        kind: ErrorKind,
        message: String,
        constraint: Option<String>,
        table: Option<String>,
    },
}
```

#### Constructors

| Method                 | Signature                                                                                                                             | Description                                  |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| `table_not_found`      | `pub fn table_not_found(name: impl Into<String>) -> Self`                                                                             | Creates a `TableNotFound` error.             |
| `column_not_found`     | `pub fn column_not_found(name: impl Into<String>) -> Self`                                                                            | Creates a `ColumnNotFound` error.            |
| `type_mismatch`        | `pub fn type_mismatch(msg: impl Into<String>) -> Self`                                                                                | Creates a `TypeMismatch` error.              |
| `invalid_operation`    | `pub fn invalid_operation(msg: impl Into<String>) -> Self`                                                                            | Creates an `InvalidOperation` error.         |
| `invalid_query`        | `pub fn invalid_query(msg: impl Into<String>) -> Self`                                                                                | Creates an `InvalidQuery` error.             |
| `unsupported`          | `pub fn unsupported(msg: impl Into<String>) -> Self`                                                                                  | Creates an `Unsupported` error.              |
| `corruption`           | `pub fn corruption<E>(err: E) -> Self where E: Error + Send + Sync + 'static`                                                         | Creates a `Corruption` error from any error. |
| `transaction`          | `pub fn transaction<E>(err: E) -> Self where E: Error + Send + Sync + 'static`                                                        | Creates a `Transaction` error.               |
| `constraint_violation` | `pub fn constraint_violation(kind: ErrorKind, message: impl Into<String>, constraint: Option<String>, table: Option<String>) -> Self` | Creates a `ConstraintViolation` error.       |

#### Trait Implementations

- `Debug`
- `Clone`
- `PartialEq`
- `Display`
- `std::error::Error`
- `MonumentumError`
- `From<std::io::Error>`

---

### `MonumentumError` Trait

A trait for errors that can provide a kind, message, and optional constraint/table context.

```rust
pub trait MonumentumError: Error + Send + Sync {
    fn kind(&self) -> ErrorKind;
    fn message(&self) -> &str;
    fn constraint(&self) -> Option<&str> { None }
    fn table(&self) -> Option<&str> { None }

    fn is_unique_violation(&self) -> bool { ... }
    fn is_foreign_key_violation(&self) -> bool { ... }
    fn is_not_null_violation(&self) -> bool { ... }
    fn is_check_violation(&self) -> bool { ... }
    fn is_type_mismatch(&self) -> bool { ... }
}
```

`DbError` implements this trait. The helper methods use `kind()` to quickly test for specific categories.

---

## Store Module

### `StorageEngine` Trait

Defines the interface for storage backends.

```rust
pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
}
```

Implementations are provided for `FileStorage` and `InMemoryStorage`.

---

### `FileStorage`

A file‑backed storage engine that uses a snapshot file and a write‑ahead log.

```rust
pub struct FileStorage { /* private fields */ }
```

#### Methods

| Method             | Signature                                                        | Description                                                |
| ------------------ | ---------------------------------------------------------------- | ---------------------------------------------------------- |
| `open`             | `pub fn open(path: &Path) -> Result<Self, DbError>`              | Opens (or initializes) the storage at the given path.      |
| `sync`             | `pub fn sync(&mut self) -> Result<(), DbError>`                  | Forces synchronization of the WAL to disk.                 |
| `checkpoint`       | `pub fn checkpoint(&mut self) -> Result<(), DbError>`            | Writes a snapshot and truncates the WAL.                   |
| `reload_from_disk` | `pub fn reload_from_disk(&mut self) -> Result<Catalog, DbError>` | Reloads the catalog from disk, discarding unsaved changes. |
| `close`            | `pub fn close(mut self) -> Result<(), DbError>`                  | Releases the file lock and closes resources.               |

#### Trait Implementations

- `Debug`
- `StorageEngine`

---

### `InMemoryStorage`

A simple in‑memory storage backend that stores the catalog in memory.

```rust
pub struct InMemoryStorage { /* private fields */ }
```

#### Methods

| Method | Signature              | Description                         |
| ------ | ---------------------- | ----------------------------------- |
| `new`  | `pub fn new() -> Self` | Creates an empty in‑memory storage. |

#### Trait Implementations

- `Debug`
- `Default`
- `StorageEngine`

---

### `Wal`

Write‑ahead log for durability. It uses file locking to prevent concurrent access.

```rust
pub struct Wal { /* private fields */ }
```

#### Methods

| Method     | Signature                                                         | Description                                        |
| ---------- | ----------------------------------------------------------------- | -------------------------------------------------- |
| `open`     | `pub fn open(path: &Path) -> Result<Self, DbError>`               | Opens the WAL file and acquires an exclusive lock. |
| `append`   | `pub fn append(&mut self, payload: &[u8]) -> Result<(), DbError>` | Appends a record.                                  |
| `sync`     | `pub fn sync(&self) -> Result<(), DbError>`                       | Forces data to disk.                               |
| `read_all` | `pub fn read_all(&mut self) -> Result<Vec<Vec<u8>>, DbError>`     | Reads all records.                                 |
| `truncate` | `pub fn truncate(&mut self) -> Result<(), DbError>`               | Empties the log.                                   |
| `unlock`   | `pub fn unlock(&mut self) -> Result<(), DbError>`                 | Releases the file lock.                            |

#### Trait Implementations

- `Debug`
- `Drop` (automatically unlocks if not already done)

---

### `RecoveryResult`

Result of WAL recovery.

```rust
pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}
```

#### Fields

| Field     | Type           | Description                    |
| --------- | -------------- | ------------------------------ |
| `records` | `Vec<Vec<u8>>` | The records read from the WAL. |

#### Trait Implementations

- `Debug` (derived automatically)

---

### Functions

#### `recover_wal`

```rust
pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError>
```

Opens a WAL at `path` and reads all records into a `RecoveryResult`.

#### `open_or_create`

```rust
pub fn open_or_create(path: &Path) -> Result<File, DbError>
```

Opens a file for reading and writing, creating it if it doesn’t exist. On Unix, sets permissions to `0600` and uses `O_NOFOLLOW`.

#### `read_file`

```rust
pub fn read_file(path: &Path) -> Result<Vec<u8>, DbError>
```

Reads the entire contents of a file into a `Vec<u8>`.

#### `write_all_atomic`

```rust
pub fn write_all_atomic(path: &Path, data: &[u8]) -> Result<(), DbError>
```

Atomically writes `data` to `path` by writing to a temporary file and renaming.

#### `append_to_file`

```rust
pub fn append_to_file(file: &mut File, data: &[u8]) -> Result<(), DbError>
```

Appends `data` to an open file.

#### `sync_file`

```rust
pub fn sync_file(file: &File) -> Result<(), DbError>
```

Calls `sync_all()` on the file.

#### `append_record`

```rust
pub fn append_record(file: &mut File, payload: &[u8]) -> Result<(), DbError>
```

Appends a log record with header (magic, version, length, CRC32) and payload.

#### `read_records`

```rust
pub fn read_records(file: &mut File) -> Result<Vec<Vec<u8>>, DbError>
```

Reads all records from a log file, verifying headers and checksums.

#### `encode_catalog`

```rust
pub fn encode_catalog(catalog: &Catalog) -> Result<Vec<u8>, DbError>
```

Serializes a catalog to a binary buffer.

#### `decode_catalog`

```rust
pub fn decode_catalog(data: &[u8]) -> Result<Catalog, DbError>
```

Deserializes a catalog from a binary buffer.

---

## Types Module

### `Blob`

A wrapper around `Vec<u8>` for binary data.

```rust
pub struct Blob(Vec<u8>);
```

#### Methods

| Method     | Signature                            | Description                          |
| ---------- | ------------------------------------ | ------------------------------------ |
| `new`      | `pub fn new(value: Vec<u8>) -> Self` | Creates a new blob.                  |
| `as_slice` | `pub fn as_slice(&self) -> &[u8]`    | Returns a byte slice.                |
| `len`      | `pub fn len(&self) -> usize`         | Returns the number of bytes.         |
| `is_empty` | `pub fn is_empty(&self) -> bool`     | Returns `true` if the blob is empty. |

#### Trait Implementations

- `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
- `Display`
- `From<Vec<u8>>`, `From<&[u8]>`
- `AsRef<[u8]>`

---

### `Float`

A wrapper around `f64` that guarantees the value is finite.

```rust
pub struct Float(f64);
```

#### Methods

| Method              | Signature                                                           | Description                                                  |
| ------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------ |
| `try_new`           | `pub fn try_new(value: f64) -> Result<Self, DbError>`               | Creates a `Float`, rejecting NaN and infinity.               |
| `as_f64`            | `pub const fn as_f64(self) -> f64`                                  | Returns the inner `f64`.                                     |
| `total_cmp`         | `pub fn total_cmp(&self, other: &Self) -> Ordering`                 | Provides total ordering (including NaN handling).            |
| `try_from_le_bytes` | `pub fn try_from_le_bytes(bytes: [u8; 8]) -> Result<Self, DbError>` | Deserializes from little‑endian bytes, rejecting non‑finite. |
| `to_le_bytes`       | `pub const fn to_le_bytes(self) -> [u8; 8]`                         | Serializes to little‑endian bytes.                           |

#### Trait Implementations

- `Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`
- `Display`
- `TryFrom<f64>`, `TryFrom<&str>`

---

### `Integer`

A wrapper around `i64` providing checked arithmetic.

```rust
pub struct Integer(i64);
```

#### Methods

| Method          | Signature                                                   | Description                                            |
| --------------- | ----------------------------------------------------------- | ------------------------------------------------------ |
| `new`           | `pub const fn new(value: i64) -> Self`                      | Creates a new integer.                                 |
| `as_i64`        | `pub const fn as_i64(self) -> i64`                          | Returns the inner `i64`.                               |
| `checked_add`   | `pub const fn checked_add(self, rhs: Self) -> Option<Self>` | Checked addition.                                      |
| `checked_sub`   | `pub const fn checked_sub(self, rhs: Self) -> Option<Self>` | Checked subtraction.                                   |
| `checked_mul`   | `pub const fn checked_mul(self, rhs: Self) -> Option<Self>` | Checked multiplication.                                |
| `checked_div`   | `pub const fn checked_div(self, rhs: Self) -> Option<Self>` | Checked division (returns `None` on division by zero). |
| `to_le_bytes`   | `pub const fn to_le_bytes(self) -> [u8; 8]`                 | Serializes to little‑endian bytes.                     |
| `from_le_bytes` | `pub const fn from_le_bytes(bytes: [u8; 8]) -> Self`        | Deserializes from little‑endian bytes.                 |

#### Trait Implementations

- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
- `Display`
- `From<i64>`
- `TryFrom<&str>`

---

### `Text`

A wrapper around `String` providing convenient text operations.

```rust
pub struct Text(String);
```

#### Methods

| Method                 | Signature                                                  | Description                        |
| ---------------------- | ---------------------------------------------------------- | ---------------------------------- |
| `new`                  | `pub fn new(value: String) -> Self`                        | Creates a new text.                |
| `as_str`               | `pub fn as_str(&self) -> &str`                             | Returns a string slice.            |
| `len`                  | `pub fn len(&self) -> usize`                               | Returns the number of bytes.       |
| `is_empty`             | `pub fn is_empty(&self) -> bool`                           | Returns `true` if empty.           |
| `to_lowercase`         | `pub fn to_lowercase(&self) -> Self`                       | Returns a lowercase copy.          |
| `to_uppercase`         | `pub fn to_uppercase(&self) -> Self`                       | Returns an uppercase copy.         |
| `contains_ignore_case` | `pub fn contains_ignore_case(&self, needle: &str) -> bool` | Case‑insensitive substring search. |
| `as_bytes`             | `pub fn as_bytes(&self) -> &[u8]`                          | Returns the underlying bytes.      |

#### Trait Implementations

- `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
- `Display`
- `From<String>`, `From<&str>`
- `AsRef<str>`

---

## Trait Implementations Summary

The following traits are implemented for the main types:

| Type              | Debug | Clone | Copy | PartialEq | Eq  | PartialOrd | Ord | Hash | Display | Default | From/TryFrom                    |
| ----------------- | ----- | ----- | ---- | --------- | --- | ---------- | --- | ---- | ------- | ------- | ------------------------------- |
| `Catalog`         | ✔     | ✔     |      | ✔         |     |            |     |      |         | ✔       |                                 |
| `Table`           | ✔     | ✔     |      | ✔         |     |            |     |      |         |         |                                 |
| `Row`             | ✔     | ✔     |      | ✔         |     | ✔          |     |      |         |         |                                 |
| `Value`           | ✔     | ✔     |      | ✔         |     | ✔          |     |      | ✔       | ✔       | Many `From`/`TryFrom`           |
| `ColumnDef`       | ✔     | ✔     |      | ✔         |     |            |     |      |         |         |                                 |
| `DataType`        | ✔     | ✔     |      | ✔         | ✔   |            |     |      | ✔       |         |                                 |
| `ComparisonOp`    | ✔     | ✔     |      | ✔         | ✔   |            |     |      |         |         |                                 |
| `CheckConstraint` | ✔     | ✔     |      | ✔         |     |            |     |      |         |         |                                 |
| `ForeignKey`      | ✔     | ✔     |      | ✔         | ✔   |            |     |      |         |         |                                 |
| `TableSchema`     | ✔     | ✔     |      | ✔         |     |            |     |      |         |         |                                 |
| `ErrorKind`       | ✔     | ✔     | ✔    | ✔         | ✔   |            |     |      |         |         |                                 |
| `DbError`         | ✔     | ✔     |      | ✔         |     |            |     |      | ✔       |         | `From<std::io::Error>`          |
| `FileStorage`     | ✔     |       |      |           |     |            |     |      |         |         |                                 |
| `InMemoryStorage` | ✔     |       |      |           |     |            |     |      |         | ✔       |                                 |
| `Wal`             | ✔     |       |      |           |     |            |     |      |         |         |                                 |
| `Blob`            | ✔     | ✔     |      | ✔         | ✔   | ✔          | ✔   | ✔    | ✔       |         | `From<Vec<u8>>`, `From<&[u8]>`  |
| `Float`           | ✔     | ✔     | ✔    | ✔         |     | ✔          |     |      | ✔       |         | `TryFrom<f64>`, `TryFrom<&str>` |
| `Integer`         | ✔     | ✔     | ✔    | ✔         | ✔   | ✔          | ✔   | ✔    | ✔       |         | `From<i64>`, `TryFrom<&str>`    |
| `Text`            | ✔     | ✔     |      | ✔         | ✔   | ✔          | ✔   | ✔    | ✔       |         | `From<String>`, `From<&str>`    |
