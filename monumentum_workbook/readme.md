# Monumentum Workbook

**High-level spreadsheet operations built on `monumentum_db` and `monumentum_query`.**

`monumentum_workbook` provides an application-level API for managing spreadsheets as a collection of sheets, each containing rows and columns of typed cells. It supports cell editing, sheet management, sorting, filtering, formula evaluation, column/row insertion, protection, and transactions. All operations are built on top of the `monumentum_db` storage layer and the `monumentum_query` formula engine.

Part of the [Monumentum](https://github.com/mroczect/monumentum) workspace.

---

## Table of Contents

- [Overview](#overview)
- [Design Goals](#design-goals)
- [Installation](#installation)
- [Architecture](#architecture)
- [Workbook Struct](#workbook-struct)
- [Sheet Management](#sheet-management)
- [Data Operations](#data-operations)
- [Cell Editing](#cell-editing)
- [Formula Support](#formula-support)
- [Column and Row Insertion/Deletion](#column-and-row-insertiondeletion)
- [File Handling](#file-handling)
- [Transactions](#transactions)
- [Errors](#errors)
- [Examples](#examples)
  - [In-Memory Workbook](#in-memory-workbook)
  - [File-Based Workbook](#file-based-workbook)
- [Full API Reference](#full-api-reference)
  - [Workbook (lib.rs)](#workbook-librs)
  - [sheet.rs](#sheetrs)
  - [data.rs](#datars)
  - [edit.rs](#editrs)
  - [formula.rs](#formulars)
  - [insert.rs](#insertrs)
  - [file.rs](#filers)
  - [transaction.rs](#transactionrs)
  - [error.rs](#errorrs)
- [Testing](#testing)
- [Security](#security)
- [License](#license)

---

## Overview

`monumentum_workbook` offers a convenient interface for building spreadsheet-like applications in Rust. It abstracts away the low-level table and row handling of `monumentum_db` and integrates formula evaluation from `monumentum_query`. With this crate, you can:

- Create, rename, drop, and list sheets
- Insert and delete rows and columns
- Read and write individual cells
- Sort and filter data
- Set formulas and evaluate them on demand
- Protect sheets from modification
- Use transactions for atomic operations
- Save and load workbooks to/from files

The workbook automatically registers a set of common spreadsheet functions (`SUM`, `AVERAGE`, `MIN`, `MAX`, `IF`, `AND`, `OR`, `NOT`, `CONCAT`, `TRIM`, `UPPER`, `LOWER`, `LEN`) via `monumentum_functions`. Developers can also register custom functions.

---

## Design Goals

- **Ease of use** – provide high‑level methods for common spreadsheet tasks.
- **Data integrity** – every cell update validates against column constraints and updates unique indexes.
- **Formula support** – integrate the `monumentum_query` engine with circular reference detection and depth limits.
- **Atomic operations** – rename and replace operations are atomic where possible.
- **File persistence** – save workbooks to disk with WAL and snapshot support.
- **Safety** – all errors are explicit via `WorkbookError`; no panics on invalid input.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
monumentum_workbook = { git = "https://github.com/mroczect/monumentum" }
```

The crate depends on:

- `monumentum_db`
- `monumentum_query`
- `monumentum_functions`

They will be pulled automatically.

---

## Architecture

The crate is organized into modules under `src/menu/` for different operation groups:

```
monumentum_workbook/
├── src/
│   ├── lib.rs            # Workbook struct, core methods
│   ├── error.rs          # WorkbookError
│   ├── menu/
│   │   ├── sheet.rs      # sheet management
│   │   ├── data.rs       # sorting, filtering, distinct
│   │   ├── edit.rs       # cell read/write, find/replace
│   │   ├── formula.rs    # formula setting/evaluation
│   │   ├── insert.rs     # row/column insertion/deletion
│   │   ├── file.rs       # open/save/create
│   │   └── mod.rs
│   └── transaction.rs    # Transaction struct
```

The core type is `Workbook<S: StorageEngine>`, which holds a `Catalog`, a storage engine, and a `FunctionRegistry`.

---

## Workbook Struct

```rust
pub struct Workbook<S: StorageEngine> {
    catalog: Catalog,
    storage: S,
    functions: FunctionRegistry,
}
```

The `Workbook` is generic over `S: StorageEngine`, allowing both in-memory (`InMemoryStorage`) and file-based (`FileStorage`) backends.

### Core Methods

- `catalog()` – returns a reference to the internal `Catalog`.
- `catalog_mut()` – returns a mutable reference.
- `register_function(name, func)` – adds a custom function to the formula registry.
- `functions()` – returns the current `FunctionRegistry`.
- `sheet(name)` / `sheet_mut(name)` – get a table by name.
- `persist_catalog()` – saves the current catalog to the storage engine.
- `set_allowed_values(sheet, col_name, values)` – update allowed values for a column (with validation).
- `validate_cell(sheet, row_idx, col_idx)` – validate an existing cell against its column definition.
- `protect_sheet(sheet)`, `unprotect_sheet(sheet)`, `is_sheet_protected(sheet)` – manage sheet protection.

---

## Sheet Management

Methods in `menu/sheet.rs`:

- `sheet_names() -> Vec<String>` – list all sheet names.
- `create_sheet(name, columns) -> Result<(), WorkbookError>` – create a new sheet with the given column definitions.
- `drop_sheet(name) -> Result<(), WorkbookError>` – delete a sheet.
- `rename_sheet(old_name, new_name) -> Result<(), WorkbookError>` – rename a sheet atomically.
- `insert_row(sheet, values) -> Result<(), WorkbookError>` – append a row.
- `delete_row(sheet, index) -> Result<(), WorkbookError>` – remove a row by index.
- `clear_sheet(sheet) -> Result<(), WorkbookError>` – remove all rows.

---

## Data Operations

Methods in `menu/data.rs`:

- `sort_sheet(sheet, col_idx, ascending) -> Result<(), WorkbookError>` – sort rows by a column. If the sheet contains formulas, the operation is refused to prevent reference corruption.
- `filter_sheet(sheet, col_idx, value) -> Result<Vec<Row>, WorkbookError>` – return rows where the column equals the given value. Formula cells are evaluated before comparison.
- `distinct_values(sheet, col_idx) -> Result<Vec<Value>, WorkbookError>` – return unique evaluated values from a column.

---

## Cell Editing

Methods in `menu/edit.rs`:

- `row_count(sheet) -> Result<usize, WorkbookError>` – number of rows.
- `column_count(sheet) -> Result<usize, WorkbookError>` – number of columns.
- `get_cell(sheet, row_idx, col_idx) -> Option<&Value>` – raw cell value (may be a formula).
- `set_cell(sheet, row_idx, col_idx, value) -> Result<(), WorkbookError>` – set a cell value with validation and index update.
- `replace_in_sheet(sheet, old_value, new_value) -> Result<usize, WorkbookError>` – replace all occurrences of a value; uses `Table::set_cell` for each changed cell to maintain integrity.
- `find_in_sheet(sheet, value) -> Result<Vec<(usize, usize)>, WorkbookError>` – find all cells with a given value.

---

## Formula Support

Methods in `menu/formula.rs`:

- `set_formula(sheet, row_idx, col_idx, formula) -> Result<(), WorkbookError>` – assign a formula string to a cell. The formula is stored as `Value::Formula` after passing column validation (which allows formulas).
- `get_cell_value(sheet, row_idx, col_idx) -> Result<Value, WorkbookError>` – evaluate the cell; if it is a formula, parse and evaluate it using the workbook as context.

Formula evaluation includes:

- Circular reference detection (returns `WorkbookError::CircularReference`).
- Maximum evaluation depth of 128.
- Support for cross-sheet references (`Sheet2!A1`).
- Range references within functions.

---

## Column and Row Insertion/Deletion

Methods in `menu/insert.rs`:

- `insert_row_at(sheet, index, values) -> Result<(), WorkbookError>` – insert a row at a specific index.
- `insert_column(sheet, index, col_def) -> Result<(), WorkbookError>` – insert a new column. Returns `WorkbookError::InvalidArgument` if the column is non-nullable and has no default.
- `delete_column(sheet, index) -> Result<(), WorkbookError>` – remove a column.

---

## File Handling

Methods in `menu/file.rs` for `Workbook<FileStorage>`:

- `open(path) -> Result<Self, WorkbookError>` – open an existing workbook from a `.monumentum` file.
- `create_new(path) -> Result<Self, WorkbookError>` – create a new workbook file.
- `save(&mut self) -> Result<(), WorkbookError>` – save catalog and checkpoint.
- `save_as(&mut self, path) -> Result<(), WorkbookError>` – save to a new file.
- `save_a_copy(&self, path) -> Result<(), WorkbookError>` – save a copy without changing the current workbook.
- `reload(&mut self) -> Result<(), WorkbookError>` – reload catalog from storage.
- `close(self) -> Result<(), WorkbookError>` – close the workbook.

For `Workbook<InMemoryStorage>`:

- `new_in_memory() -> Self` – create an empty in-memory workbook.
- `load_in_memory(catalog) -> Self` – wrap an existing catalog.
- `save(&mut self) -> Result<(), WorkbookError>` – store catalog in memory.
- `reload(&mut self)` – reload (no-op for in-memory).
- `close(self) -> Result<(), WorkbookError>`.

---

## Transactions

`Transaction` in `transaction.rs` provides a simple snapshot-based rollback mechanism.

```rust
pub struct Transaction<'a, S: StorageEngine> { /* private */ }

impl<'a, S: StorageEngine> Transaction<'a, S> {
    pub fn begin(workbook: &'a mut Workbook<S>) -> Self;
    pub const fn workbook_mut(&mut self) -> &mut Workbook<S>;
    pub fn commit(self) -> Result<(), WorkbookError>;
    pub fn rollback(self);
}
```

- `begin` takes a snapshot of the catalog.
- `commit` persists the workbook.
- `rollback` restores the snapshot (does not automatically save).

---

## Errors

`WorkbookError` enum with variants covering spreadsheet-specific errors, database errors, and formula errors.

```rust
pub enum WorkbookError {
    CellTooNarrow,
    FormatOutOfRange,
    NotAvailable,
    InvalidCharacter,
    InvalidArgument,
    // ... many more spreadsheet error codes
    Db(String),
    Formula(String),
    FileExists,
    InvalidExtension,
}
```

Implements `Display`, `Error`, `From<DbError>`, and `From<FormulaError>`.

---

## Examples

### In-Memory Workbook

```rust
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::Workbook;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();

    // Create a sheet
    wb.create_sheet(
        "Data",
        vec![
            ColumnDef::new("Nama", DataType::Text),
            ColumnDef::new("Nilai", DataType::Integer),
        ],
    )?;

    // Insert rows
    wb.insert_row("Data", vec![Value::from("Alice"), Value::from(90_i64)])?;
    wb.insert_row("Data", vec![Value::from("Bob"), Value::from(80_i64)])?;

    // Set and evaluate a formula
    wb.insert_row("Data", vec![Value::from("Rata-rata"), Value::Null])?;
    let row_idx = wb.row_count("Data")? - 1;
    wb.set_formula("Data", row_idx, 1, "AVERAGE(B1:B2)")?;
    let result = wb.get_cell_value("Data", row_idx, 1)?;
    println!("Average: {:?}", result); // Float(85.0)

    Ok(())
}
```

### File-Based Workbook

```rust
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::FileStorage;
use monumentum_workbook::Workbook;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("demo.monumentum");
    let mut wb = Workbook::<FileStorage>::create_new(path)?;

    wb.create_sheet(
        "Siswa",
        vec![
            ColumnDef::new("Nama", DataType::Text),
            ColumnDef::new("Nilai", DataType::Integer),
        ],
    )?;

    wb.insert_row("Siswa", vec![Value::from("Alice"), Value::from(90_i64)])?;
    wb.save()?;
    wb.close()?;

    // Reopen
    let wb = Workbook::<FileStorage>::open(path)?;
    println!("Sheets: {:?}", wb.sheet_names());
    Ok(())
}
```

---

## Full API Reference

### Workbook (lib.rs)

```rust
pub struct Workbook<S: StorageEngine> { /* private */ }

impl<S: StorageEngine> Workbook<S> {
    pub fn catalog(&self) -> &Catalog;
    pub fn catalog_mut(&mut self) -> &mut Catalog;
    pub fn register_function(&mut self, name: &str, func: FunctionImpl);
    pub fn functions(&self) -> &FunctionRegistry;
    pub fn sheet(&self, name: &str) -> Result<&Table, WorkbookError>;
    pub fn sheet_mut(&mut self, name: &str) -> Result<&mut Table, WorkbookError>;
    pub fn persist_catalog(&mut self) -> Result<(), WorkbookError>;
    pub fn set_allowed_values(
        &mut self,
        sheet: &str,
        col_name: &str,
        values: Vec<Value>,
    ) -> Result<(), WorkbookError>;
    pub fn validate_cell(
        &self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<(), WorkbookError>;
    pub fn protect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>;
    pub fn unprotect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>;
    pub fn is_sheet_protected(&self, sheet: &str) -> Result<bool, WorkbookError>;
}
```

### sheet.rs

```rust
impl<S: StorageEngine> Workbook<S> {
    pub fn sheet_names(&self) -> Vec<String>;
    pub fn create_sheet(
        &mut self,
        name: &str,
        columns: Vec<ColumnDef>,
    ) -> Result<(), WorkbookError>;
    pub fn drop_sheet(&mut self, name: &str) -> Result<(), WorkbookError>;
    pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), WorkbookError>;
    pub fn insert_row(&mut self, sheet: &str, values: Vec<Value>) -> Result<(), WorkbookError>;
    pub fn delete_row(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError>;
    pub fn clear_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>;
}
```

### data.rs

```rust
impl<S: StorageEngine> Workbook<S> {
    pub fn sort_sheet(
        &mut self,
        sheet: &str,
        col_idx: usize,
        ascending: bool,
    ) -> Result<(), WorkbookError>;
    pub fn filter_sheet(
        &self,
        sheet: &str,
        col_idx: usize,
        value: &Value,
    ) -> Result<Vec<Row>, WorkbookError>;
    pub fn distinct_values(
        &self,
        sheet: &str,
        col_idx: usize,
    ) -> Result<Vec<Value>, WorkbookError>;
}
```

### edit.rs

```rust
impl<S: StorageEngine> Workbook<S> {
    pub fn row_count(&self, sheet: &str) -> Result<usize, WorkbookError>;
    pub fn column_count(&self, sheet: &str) -> Result<usize, WorkbookError>;
    pub fn get_cell(&self, sheet: &str, row_idx: usize, col_idx: usize) -> Option<&Value>;
    pub fn set_cell(
        &mut self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
        value: Value,
    ) -> Result<(), WorkbookError>;
    pub fn replace_in_sheet(
        &mut self,
        sheet: &str,
        old_value: &Value,
        new_value: &Value,
    ) -> Result<usize, WorkbookError>;
    pub fn find_in_sheet(
        &self,
        sheet: &str,
        value: &Value,
    ) -> Result<Vec<(usize, usize)>, WorkbookError>;
}
```

### formula.rs

```rust
impl<S: StorageEngine> Workbook<S> {
    pub fn set_formula(
        &mut self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
        formula: &str,
    ) -> Result<(), WorkbookError>;
    pub fn get_cell_value(
        &self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<Value, WorkbookError>;
}
```

### insert.rs

```rust
impl<S: StorageEngine> Workbook<S> {
    pub fn insert_row_at(
        &mut self,
        sheet: &str,
        index: usize,
        values: Vec<Value>,
    ) -> Result<(), WorkbookError>;
    pub fn insert_column(
        &mut self,
        sheet: &str,
        index: usize,
        col_def: &ColumnDef,
    ) -> Result<(), WorkbookError>;
    pub fn delete_column(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError>;
}
```

### file.rs

```rust
impl Workbook<FileStorage> {
    pub fn open(path: &Path) -> Result<Self, WorkbookError>;
    pub fn create_new(path: &Path) -> Result<Self, WorkbookError>;
    pub fn save(&mut self) -> Result<(), WorkbookError>;
    pub fn save_as(&mut self, path: &Path) -> Result<(), WorkbookError>;
    pub fn save_a_copy(&self, path: &Path) -> Result<(), WorkbookError>;
    pub fn reload(&mut self) -> Result<(), WorkbookError>;
    pub fn close(self) -> Result<(), WorkbookError>;
}

impl Workbook<InMemoryStorage> {
    pub fn new_in_memory() -> Self;
    pub fn load_in_memory(catalog: Catalog) -> Self;
    pub fn save(&mut self) -> Result<(), WorkbookError>;
    pub fn reload(&mut self) -> Result<(), WorkbookError>;
    pub fn close(self) -> Result<(), WorkbookError>;
}
```

### transaction.rs

```rust
pub struct Transaction<'a, S: StorageEngine> { /* private */ }

impl<'a, S: StorageEngine> Transaction<'a, S> {
    pub fn begin(workbook: &'a mut Workbook<S>) -> Self;
    pub const fn workbook_mut(&mut self) -> &mut Workbook<S>;
    pub fn commit(self) -> Result<(), WorkbookError>;
    pub fn rollback(self);
}
```

### error.rs

```rust
pub enum WorkbookError {
    CellTooNarrow,
    FormatOutOfRange,
    NotAvailable,
    InvalidCharacter,
    InvalidArgument,
    InvalidFloatingPointOperation,
    ParameterListError,
    PairMissing,
    MissingOperator,
    MissingVariable,
    MissingVariableForFunction,
    FormulaOverflow,
    StringOverflow,
    InternalOverflow,
    InternalSyntaxError,
    MatrixExpected,
    UnknownCode,
    VariableNotAvailable,
    NoValue,
    Null,
    CircularReference,
    NoConvergence,
    InvalidReference,
    InvalidName,
    ReferenceTooEncapsulated,
    AddInNotFound,
    MacroNotFound,
    DivisionByZero,
    NestedArraysNotSupported,
    ArraySizeExceeded,
    UnsupportedInlineArrayContent,
    ExternalContentDisabled,
    Db(String),
    Formula(String),
    FileExists,
    InvalidExtension,
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
- **Resource limits** – formula evaluation has depth and range limits.
- **Data integrity** – cell updates validate against column constraints and update unique indexes.
- **Atomic file operations** – save and rename use atomic strategies.
- **File extension validation** – only `.monumentum` files are accepted.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
