# monumentum_handler

A pure contracts crate for the Monumentum database system.  
It defines the fundamental types, traits, constants, validation functions, and error types used across the ecosystem.  
No concrete implementations are included here; implementors are expected to provide backends in separate crates (e.g., `monumentum_core`).

## Features

- **Resource limits** – Hard `MAX_*` constants prevent memory exhaustion.
- **Core data types** – `Value`, `Row`, `TableSchema`, `ColumnDef`, `DataType`, `ComparisonOp`, `CheckConstraint`, `ForeignKey`.
- **Wrapper types** – `Blob`, `Float`, `Integer`, `Text` with size and finiteness checks.
- **Unified error handling** – `DbError`, `ErrorKind`, and the `MonumentumError` trait.
- **Contract traits** – `CatalogStore`, `TableStore`, `StorageEngine`, `Index`.
- **Validation functions** – Name, column name, and table name validation.
- **No `unsafe`** – `#![forbid(unsafe_code)]`
- **No external dependencies** – Uses only `std` and `alloc`.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
monumentum_handler = "0.1"
```

## API Reference

### Constants (`constants.rs`)

| Constant             | Value               | Description                         |
| -------------------- | ------------------- | ----------------------------------- |
| `HASH_LENGTH`        | `64`                | Hash length in bytes                |
| `MAX_NAME_LENGTH`    | `255`               | Maximum name length in bytes        |
| `MAX_COLUMNS`        | `1024`              | Maximum columns per table           |
| `MAX_TEXT_SIZE`      | `16 * 1024 * 1024`  | Maximum text size (16 MiB)          |
| `MAX_BLOB_SIZE`      | `64 * 1024 * 1024`  | Maximum blob size (64 MiB)          |
| `MAX_ROWS_PER_TABLE` | `10_000_000`        | Maximum rows per table              |
| `MAX_TABLES`         | `1024`              | Maximum tables per catalog          |
| `MAX_RECORD_SIZE`    | `64 * 1024 * 1024`  | Maximum WAL record size             |
| `MAX_SNAPSHOT_SIZE`  | `256 * 1024 * 1024` | Maximum snapshot file size (bytes)  |
| `MAX_VEC_ELEMENTS`   | `1_000_000`         | Maximum elements in decoded vectors |

### Core Types

#### `Value` (`core/value.rs`)

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

Methods:

- Type checks: `is_null()`, `is_integer()`, `is_float()`, `is_text()`, `is_blob()`, `is_boolean()`
- Accessors: `as_integer()`, `as_float()`, `as_text()`, `as_blob()`, `as_boolean()`
- Conversions: `as_i64()`, `as_f64()`, `as_bool()`, `as_str()`
- Consuming conversions: `into_integer()`, `into_float()`, `into_text()`, `into_blob()`, `into_boolean()`
- `Display` formatting

Implements: `Debug`, `Clone`, `PartialEq`, `PartialOrd`, `Default`, `Display`, `From` for multiple types, `TryFrom<f64>`.

#### `Row` (`core/row.rs`)

```rust
pub struct Row {
    values: Vec<Value>,
}
```

Methods:

- `new(values: Vec<Value>) -> Self`
- `values(&self) -> &[Value]`
- `get<I: ColumnIndex<Self>>(&self, index: &I) -> Option<&Value>`
- `len()`, `is_empty()`, `values_mut()`, `get_mut(index: usize)`

#### `DataType` (`core/schema/column.rs`)

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

Methods: `as_str() -> &'static str`

#### `ComparisonOp`

```rust
pub enum ComparisonOp {
    Eq, NotEq, Lt, Lte, Gt, Gte,
}
```

#### `CheckConstraint`

```rust
pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}
```

#### `ForeignKey`

```rust
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}
```

#### `ColumnDef`

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

Methods:

- `new(name, data_type) -> Self`
- `name()`, `data_type()`, `is_nullable()`, `is_primary_key()`, `is_unique()`
- `default_value()`, `check_constraint()`, `foreign_key()`, `allowed_values()`
- Setters: `set_nullable`, `set_primary_key`, `set_unique`, `set_default`, `set_check`, `set_foreign_key`, `set_allowed_values`
- `validate_value(&self, value: &Value) -> Result<(), DbError>`

Implements `Column` trait.

#### `TableSchema`

```rust
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
}
```

Methods:

- `try_new(name, columns) -> Result<Self, DbError>`
- `name()`, `columns()`, `get_column_mut()`, `column_index()`, `get_column()`
- `validate_values(&self, values: &[Value]) -> Result<(), DbError>`
- `get_column_by_index<I>(&self, index: &I) -> Option<&ColumnDef>`

### Traits

#### `CatalogStore`

```rust
pub trait CatalogStore {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
}
```

#### `TableStore`

```rust
pub trait TableStore {
    fn insert(&mut self, row: &Row) -> Result<(), DbError>;
    fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>;
}
```

#### `StorageEngine`

```rust
pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<(), DbError>;
    fn save_catalog(&mut self) -> Result<(), DbError>;
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
}
```

#### `Index`

```rust
pub trait Index {
    fn insert(&mut self, key: &Value, row_idx: usize);
    fn remove(&mut self, key: &Value, row_idx: usize);
    fn lookup(&self, key: &Value) -> Option<&[usize]>;
}
```

### Wrapper Types

#### `Blob` (`types/blob.rs`)

```rust
pub struct Blob(Vec<u8>);
```

Methods:

- `new(Vec<u8>) -> Self`
- `try_new(Vec<u8>) -> Result<Self, DbError>`
- `as_slice() -> &[u8]`
- `len() -> usize`
- `is_empty() -> bool`

#### `Float` (`types/float.rs`)

```rust
pub struct Float(f64); // guaranteed finite
```

Methods:

- `try_new(f64) -> Result<Self, DbError>`
- `as_f64() -> f64`
- `total_cmp(&self, &Self) -> Ordering`
- `try_from_le_bytes([u8; 8]) -> Result<Self, DbError>`
- `to_le_bytes() -> [u8; 8]`

#### `Integer` (`types/integer.rs`)

```rust
pub struct Integer(i64);
```

Methods:

- `new(i64) -> Self`
- `as_i64() -> i64`
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div`
- `to_le_bytes`, `from_le_bytes`

#### `Text` (`types/text.rs`)

```rust
pub struct Text(String);
```

Methods:

- `new(String) -> Self`
- `try_new(String) -> Result<Self, DbError>`
- `as_str()`, `len()`, `is_empty()`
- `to_lowercase()`, `to_uppercase()`
- `contains_ignore_case(&str) -> bool`
- `as_bytes() -> &[u8]`

### Validation

```rust
pub fn validate_name(name: &str) -> Result<(), DbError>
pub fn validate_column_name(name: &str) -> Result<(), DbError>
pub fn validate_table_name(name: &str) -> Result<(), DbError>
```

### Error Handling

#### `ErrorKind`

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

#### `DbError`

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

Constructors:

- `table_not_found(name)`
- `column_not_found(name)`
- `type_mismatch(msg)`
- `invalid_operation(msg)`
- `invalid_query(msg)`
- `unsupported(msg)`
- `corruption(err)`
- `transaction(err)`
- `constraint_violation(kind, message, constraint, table)`
- `from_io(io_error)`

Implements: `Display`, `Error`, `Clone`, `PartialEq`, `MonumentumError`.

#### `MonumentumError` trait

```rust
pub trait MonumentumError: Error + Send + Sync {
    fn kind(&self) -> ErrorKind;
    fn message(&self) -> &str;
    fn constraint(&self) -> Option<&str> { None }
    fn table(&self) -> Option<&str> { None }
    // Helper methods: is_unique_violation, is_foreign_key_violation, etc.
}
```

## Usage Example

```rust
use monumentum_handler::{
    core::value::Value,
    types::{Integer, Text},
    validation::validate_name,
};

fn main() -> Result<(), monumentum_handler::error::DbError> {
    let int_val = Value::from(42_i64);
    let text_val = Value::from("Halo");

    validate_name("tabel_1")?;

    let big_text = Text::try_new("x".repeat(10_000))?;

    Ok(())
}
```

## Testing

```bash
cargo test
```

## License

MIT
