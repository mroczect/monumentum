# monumentum_handler

**Core contracts, types, and error handling for the Monumentum database system.**  
This crate defines the foundational data structures, traits, and error types used by both storage engines (`monumentum_core`) and higher-level components. It provides a consistent API for schema definition, row storage, value representation, and type safety.

## Overview

`monumentum_handler` is the contract layer of the Monumentum database. It contains:

- **Constants** – system limits and sizing parameters.
- **Core types** – `Value`, `Row`, `TableSchema`, `ColumnDef`, and `DataType`.
- **Error types** – a rich `DbError` enum with `ErrorKind` classification.
- **Traits** – `StorageEngine`, `CatalogStore`, `Index`, and `TableStore`.
- **Type wrappers** – `Integer`, `Float`, `Text`, `Blob` with validation.
- **Validation utilities** – name validation for tables and columns.

All components are designed to be `no_std`-friendly (using `alloc` where needed) and enforce strict constraints on data sizes and schema correctness.

## Structure

```
src/
├── constants.rs
├── core/
│   ├── mod.rs
│   ├── row.rs
│   ├── schema/
│   │   ├── column.rs
│   │   ├── mod.rs
│   │   └── table_schema.rs
│   └── value.rs
├── error.rs
├── lib.rs
├── traits/
│   ├── catalog_store.rs
│   ├── index.rs
│   ├── mod.rs
│   ├── storage_engine.rs
│   └── table_store.rs
├── types/
│   ├── blob.rs
│   ├── float.rs
│   ├── integer.rs
│   ├── mod.rs
│   └── text.rs
└── validation.rs
```

---

## Module: `constants`

Global limits used across the system.

| Constant             | Value               | Description                                       |
| -------------------- | ------------------- | ------------------------------------------------- |
| `HASH_LENGTH`        | `64`                | Length of hash outputs.                           |
| `MAX_NAME_LENGTH`    | `255`               | Maximum length for table/column names in bytes.   |
| `MAX_COLUMNS`        | `1024`              | Maximum number of columns per table.              |
| `MAX_TEXT_SIZE`      | `16 * 1024 * 1024`  | Maximum bytes for text values.                    |
| `MAX_BLOB_SIZE`      | `64 * 1024 * 1024`  | Maximum bytes for blob values.                    |
| `MAX_ROWS_PER_TABLE` | `10_000_000`        | Soft limit of rows per table.                     |
| `MAX_TABLES`         | `1024`              | Maximum number of tables in a catalog.            |
| `MAX_RECORD_SIZE`    | `64 * 1024 * 1024`  | Maximum size of a WAL record.                     |
| `MAX_SNAPSHOT_SIZE`  | `256 * 1024 * 1024` | Maximum snapshot size in bytes.                   |
| `MAX_VEC_ELEMENTS`   | `1_000_000`         | Maximum number of elements in serialized vectors. |

---

## Module: `core`

### Submodule: `core::row`

#### Struct: `Row`

A single database row containing a vector of `Value`s.

```rust
pub struct Row {
    values: Vec<Value>,
}
```

| Method                                                                                | Description                                            |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `new(values: Vec<Value>) -> Self`                                                     | Creates a row from a vector of values.                 |
| `values(&self) -> &[Value]`                                                           | Returns a slice of all values.                         |
| `get(&self, index: usize) -> Option<&Value>`                                          | Returns the value at a given column index.             |
| `get_by_name<'a>(&'a self, schema: &'a TableSchema, name: &str) -> Option<&'a Value>` | Returns a value by column name using the table schema. |
| `len(&self) -> usize`                                                                 | Returns the number of values in the row.               |
| `is_empty(&self) -> bool`                                                             | Returns `true` if the row has no values.               |

**Example:**

```rust
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;

let row = Row::new(vec![Value::from(42i64), Value::from(true)]);
assert_eq!(row.len(), 2);
assert_eq!(row.get(0), Some(&Value::from(42i64)));
```

---

### Submodule: `core::schema`

#### Enum: `DataType`

Supported column data types.

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

| Method                          | Description                                           |
| ------------------------------- | ----------------------------------------------------- |
| `as_str(&self) -> &'static str` | Returns SQL‑like type name (`INTEGER`, `TEXT`, etc.). |
| `Display`                       | Implements `Display` for human‑readable output.       |

#### Enum: `ComparisonOp`

Comparison operators used in check constraints.

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

#### Struct: `CheckConstraint`

A check constraint applied to a column.

```rust
pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}
```

#### Struct: `ForeignKey`

Reference to another table’s column.

```rust
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}
```

#### Struct: `ColumnDef`

Definition of a single table column, including constraints.

```rust
pub struct ColumnDef {
    // private fields
}
```

| Method                                                        | Description                                                                 |
| ------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `new(name: impl Into<String>, data_type: DataType) -> Self`   | Creates a column with default settings (`nullable = true`, no constraints). |
| `name(&self) -> &str`                                         | Returns the column name.                                                    |
| `data_type(&self) -> &DataType`                               | Returns the data type.                                                      |
| `is_nullable(&self) -> bool`                                  | Whether `NULL` is allowed.                                                  |
| `is_primary_key(&self) -> bool`                               | Whether column is part of primary key.                                      |
| `is_unique(&self) -> bool`                                    | Whether column has unique constraint.                                       |
| `default_value(&self) -> Option<&Value>`                      | Returns default value if set.                                               |
| `check_constraint(&self) -> Option<&CheckConstraint>`         | Returns check constraint if set.                                            |
| `foreign_key(&self) -> Option<&ForeignKey>`                   | Returns foreign key if set.                                                 |
| `allowed_values(&self) -> Option<&Vec<Value>>`                | Returns allowed values list if set.                                         |
| `set_nullable(&mut self, value: bool)`                        | Sets nullable; automatically clears if primary key.                         |
| `set_primary_key(&mut self, value: bool)`                     | Sets primary key; also sets `nullable = false` and `unique = true`.         |
| `set_unique(&mut self, value: bool)`                          | Sets unique; ignored if primary key.                                        |
| `set_default(&mut self, value: Option<Value>)`                | Sets default value.                                                         |
| `set_check(&mut self, constraint: Option<CheckConstraint>)`   | Sets check constraint.                                                      |
| `set_foreign_key(&mut self, fk: Option<ForeignKey>)`          | Sets foreign key.                                                           |
| `set_allowed_values(&mut self, values: Option<Vec<Value>>)`   | Sets allowed values list.                                                   |
| `validate_value(&self, value: &Value) -> Result<(), DbError>` | Validates a value against column’s data type and constraints.               |

**Traits implemented:**

- `Column` – basic accessors.
- `ColumnIndex<T>` – index resolution for rows and schemas using `usize` or `&str`.

#### Struct: `TableSchema`

Schema for a table, containing name and ordered columns.

```rust
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
}
```

| Method                                                                               | Description                                                                              |
| ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `try_new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Result<Self, DbError>` | Creates and validates a schema. Checks name, column count, duplicates, and name lengths. |
| `name(&self) -> &str`                                                                | Returns table name.                                                                      |
| `columns(&self) -> &[ColumnDef]`                                                     | Returns column list.                                                                     |
| `column_index(&self, name: &str) -> Option<usize>`                                   | Case‑insensitive column lookup.                                                          |
| `get_column(&self, name: &str) -> Option<&ColumnDef>`                                | Returns column by name.                                                                  |
| `validate_values(&self, values: &[Value]) -> Result<(), DbError>`                    | Validates a full row against all column constraints.                                     |
| `get_column_by_index(&self, index: usize) -> Option<&ColumnDef>`                     | Returns column by numeric index.                                                         |

**Example:**

```rust
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;

let schema = TableSchema::try_new(
    "users",
    vec![
        ColumnDef::new("id", DataType::Integer),
        ColumnDef::new("email", DataType::Text),
    ],
)?;
```

---

### Submodule: `core::value`

#### Enum: `Value`

A dynamically typed database value.

```rust
#[non_exhaustive]
pub enum Value {
    Null,
    Integer(Integer),
    Float(Float),
    Text(Text),
    Blob(Blob),
    Boolean(bool),
}
```

| Method                                                                  | Description                                                   |
| ----------------------------------------------------------------------- | ------------------------------------------------------------- |
| `is_null`, `is_integer`, `is_float`, `is_text`, `is_blob`, `is_boolean` | Type check helpers.                                           |
| `type_name(&self) -> &'static str`                                      | Returns a human‑readable type name.                           |
| `as_integer`, `as_float`, `as_text`, `as_blob`, `as_boolean`            | Returns an `Option<&T>` reference to the inner value.         |
| `into_integer`, `into_float`, `into_text`, `into_blob`, `into_boolean`  | Consumes the `Value` and returns the inner value if matching. |
| `as_i64`, `as_f64`, `as_bool`, `as_str`                                 | Convenience accessors for primitive types.                    |

**Conversions:**

- `From<()>` → `Null`
- `From<Integer>`, `From<Float>`, `From<Text>`, `From<Blob>`, `From<bool>`, `From<i64>`
- `TryFrom<f64>`, `TryFrom<String>`, `TryFrom<&str>`, `TryFrom<Vec<u8>>`, `TryFrom<&[u8]>`

**Display:** Formats values in a SQL‑like style (e.g., text is single‑quoted, blobs shown as byte count).

---

## Module: `error`

### Enum: `ErrorKind`

Classifies the type of error.

```rust
#[non_exhaustive]
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

### Trait: `MonumentumError`

Implemented by `DbError`; provides methods to query error kind, message, constraint, and table.

| Method                                                                                                               | Description                                |
| -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `kind(&self) -> ErrorKind`                                                                                           | Returns the error category.                |
| `message(&self) -> &str`                                                                                             | Returns a human‑readable message.          |
| `constraint(&self) -> Option<&str>`                                                                                  | Returns the constraint name if applicable. |
| `table(&self) -> Option<&str>`                                                                                       | Returns the table name if applicable.      |
| `is_unique_violation`, `is_foreign_key_violation`, `is_not_null_violation`, `is_check_violation`, `is_type_mismatch` | Convenience predicate methods.             |

### Enum: `DbError`

The main error type.

```rust
#[non_exhaustive]
pub enum DbError {
    Io(Arc<std::io::Error>),
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

**Constructors:**

| Method                                                   | Description                                   |
| -------------------------------------------------------- | --------------------------------------------- |
| `table_not_found(name)`                                  | Creates a `TableNotFound` error.              |
| `column_not_found(name)`                                 | Creates a `ColumnNotFound` error.             |
| `type_mismatch(msg)`                                     | Creates a `TypeMismatch` error.               |
| `invalid_operation(msg)`                                 | Creates an `InvalidOperation` error.          |
| `invalid_query(msg)`                                     | Creates an `InvalidQuery` error.              |
| `unsupported(msg)`                                       | Creates an `Unsupported` error.               |
| `corruption(err)`                                        | Wraps any error as `Corruption`.              |
| `transaction(err)`                                       | Wraps any error as `Transaction`.             |
| `constraint_violation(kind, message, constraint, table)` | Creates a `ConstraintViolation`.              |
| `from_io(io_error)`                                      | Converts `std::io::Error` into `DbError::Io`. |

**Implementations:** `Display`, `Error`, `MonumentumError`, `PartialEq`, `From<std::io::Error>`.

---

## Module: `traits`

### Trait: `CatalogStore`

Defines catalog manipulation operations.

```rust
pub trait CatalogStore {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
}
```

### Trait: `Index`

Generic index interface.

```rust
pub trait Index {
    fn insert(&mut self, key: &Value, row_idx: usize);
    fn remove(&mut self, key: &Value, row_idx: usize);
    fn lookup(&self, key: &Value) -> Option<&[usize]>;
}
```

### Trait: `StorageEngine`

Full storage engine contract used by `monumentum_core`.

```rust
pub trait StorageEngine {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
    fn insert_row(&mut self, table: &str, row: &Row) -> Result<(), DbError>;
    fn get_row(&mut self, table: &str, row_idx: usize) -> Result<Option<Row>, DbError>;
    fn set_cell(&mut self, table: &str, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    fn replace_rows(&mut self, table: &str, rows: Vec<Row>) -> Result<(), DbError>;
    fn checkpoint(&mut self) -> Result<(), DbError>;
    fn get_row_by_key(&mut self, table: &str, key: &Value) -> Result<Option<Row>, DbError>;
}
```

### Trait: `TableStore`

Low‑level row storage operations.

```rust
pub trait TableStore {
    fn insert(&mut self, row: &Row) -> Result<(), DbError>;
    fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>;
}
```

---

## Module: `types`

### Struct: `Blob`

Immutable binary data with size limit.

```rust
pub struct Blob(Vec<u8>);
```

| Method                                             | Description                                |
| -------------------------------------------------- | ------------------------------------------ |
| `try_new(value: Vec<u8>) -> Result<Self, DbError>` | Creates a blob, enforcing `MAX_BLOB_SIZE`. |
| `as_slice(&self) -> &[u8]`                         | Returns underlying bytes.                  |
| `len(&self) -> usize`                              | Returns byte length.                       |
| `is_empty(&self) -> bool`                          | Checks if empty.                           |

**Conversions:** `TryFrom<Vec<u8>>`, `TryFrom<&[u8]>`, `AsRef<[u8]>`, `Display`.

### Struct: `Float`

Finite 64‑bit floating point value.

```rust
pub struct Float(f64);
```

| Method                                               | Description                                     |
| ---------------------------------------------------- | ----------------------------------------------- |
| `try_new(value: f64) -> Result<Self, DbError>`       | Creates a finite float (rejects NaN, infinity). |
| `as_f64(self) -> f64`                                | Returns raw value.                              |
| `total_cmp(&self, other: &Self) -> Ordering`         | Total order comparison.                         |
| `try_from_le_bytes([u8;8]) -> Result<Self, DbError>` | Creates from little‑endian bytes.               |
| `to_le_bytes(self) -> [u8;8]`                        | Serializes to little‑endian bytes.              |

**Conversions:** `TryFrom<f64>`, `TryFrom<&str>`, `Display`.

### Struct: `Integer`

64‑bit signed integer wrapper.

```rust
pub struct Integer(i64);
```

| Method                                                     | Description                            |
| ---------------------------------------------------------- | -------------------------------------- |
| `new(value: i64) -> Self`                                  | Creates a new integer.                 |
| `as_i64(self) -> i64`                                      | Returns raw value.                     |
| `checked_add`, `checked_sub`, `checked_mul`, `checked_div` | Overflow‑safe arithmetic.              |
| `to_le_bytes(self) -> [u8;8]`                              | Serializes to little‑endian bytes.     |
| `from_le_bytes([u8;8]) -> Self`                            | Deserializes from little‑endian bytes. |

**Conversions:** `From<i64>`, `TryFrom<&str>`, `Display`.

### Struct: `Text`

String data with size limit.

```rust
pub struct Text(String);
```

| Method                                              | Description                              |
| --------------------------------------------------- | ---------------------------------------- |
| `try_new(value: String) -> Result<Self, DbError>`   | Creates text, enforcing `MAX_TEXT_SIZE`. |
| `as_str(&self) -> &str`                             | Returns string slice.                    |
| `len(&self) -> usize`                               | Returns byte length.                     |
| `is_empty(&self) -> bool`                           | Checks if empty.                         |
| `to_lowercase(&self) -> Self`                       | Returns lowercase copy.                  |
| `to_uppercase(&self) -> Self`                       | Returns uppercase copy.                  |
| `contains_ignore_case(&self, needle: &str) -> bool` | Case‑insensitive substring search.       |
| `as_bytes(&self) -> &[u8]`                          | Returns raw bytes.                       |

**Conversions:** `TryFrom<String>`, `TryFrom<&str>`, `AsRef<str>`, `Display`.

---

## Module: `validation`

Standalone name validation functions.

| Function                                                  | Description                                                          |
| --------------------------------------------------------- | -------------------------------------------------------------------- |
| `validate_name(name: &str) -> Result<(), DbError>`        | Checks non‑empty, length ≤ `MAX_NAME_LENGTH`, no control characters. |
| `validate_column_name(name: &str) -> Result<(), DbError>` | Wraps `validate_name` with a column‑specific error message.          |
| `validate_table_name(name: &str) -> Result<(), DbError>`  | Wraps `validate_name` with a table‑specific error message.           |

---

## Error Handling

All operations that can fail return `Result<T, DbError>`. Use `DbError::kind()` to match specific categories, or the convenience methods like `is_unique_violation()`.

Example:

```rust
use monumentum_handler::error::{DbError, ErrorKind};

fn try_insert() -> Result<(), DbError> {
    Err(DbError::constraint_violation(
        ErrorKind::UniqueViolation,
        "duplicate key",
        Some("pk_users".to_string()),
        Some("users".to_string()),
    ))
}

match try_insert() {
    Err(e) if e.is_unique_violation() => println!("Duplicate entry!"),
    Err(e) => eprintln!("Other error: {e}"),
    Ok(()) => {}
}
```

---

## Examples

### Creating a Schema and Validating a Row

```rust
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = TableSchema::try_new(
        "employees",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;

    let row = Row::new(vec![
        Value::from(1i64),
        Value::try_from("Alice".to_string())?,
    ]);

    schema.validate_values(row.values())?;

    println!("Row is valid for schema '{}'", schema.name());
    Ok(())
}
```

### Using Type Wrappers

```rust
use monumentum_handler::types::{Integer, Float, Text};

let int = Integer::new(42);
let float = Float::try_new(3.14)?;
let text = Text::try_new("hello".to_string())?;

println!("{int}, {float}, {text}");
```

---

## Testing

Run tests for the entire workspace:

```bash
cargo test --workspace
```

Run only handler tests:

```bash
cargo test -p monumentum_handler
```

Run with clippy:

```bash
cargo clippy -p monumentum_handler --all-targets --all-features -- -D warnings
```

The test suite covers:

- Constant relationships.
- Schema validation (empty names, duplicate columns, too many columns, invalid characters).
- Column constraint validation (nullable, unique, primary key, check constraints, allowed values).
- Value conversions and type checking.
- Error display and source.
- Property‑based tests for name validation, integer/float/blob/text roundtrips.

---

## License

MIT
