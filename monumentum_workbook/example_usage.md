# Monumentum Workbook: Complete API Reference and Usage Examples

This document is a **complete reference** for the `monumentum_workbook` crate. It describes every public type, method, and trait, and provides **runnable examples** for each major feature. The examples are drawn from the crate’s own test suite and are guaranteed to work with the current version.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Core Types](#core-types)
   - [`Workbook`](#workbook)
   - [`WorkbookError`](#workbookerror)
   - [`Transaction`](#transaction)
3. [Module `menu`](#module-menu)
   - [`data`](#menu-data)
   - [`edit`](#menu-edit)
   - [`export`](#menu-export)
   - [`file`](#menu-file)
   - [`formula`](#menu-formula)
   - [`import`](#menu-import)
   - [`insert`](#menu-insert)
   - [`sheet`](#menu-sheet)
4. [Module `query`](#module-query)
   - [`Query`](#query)
   - [`QueryBuilder`](#querybuilder)
   - [`QueryAs`](#queryas)
   - [`Map`](#map)
   - [`FromRow` and `FromValue`](#fromrow-and-fromvalue)
5. [Module `transaction`](#module-transaction)
6. [Advanced Topics](#advanced-topics)
   - [Custom Functions](#custom-functions)
   - [Error Handling Patterns](#error-handling-patterns)
   - [Protecting Sheets](#protecting-sheets)
7. [Complete Example Application](#complete-example-application)

---

## Introduction

`monumentum_workbook` provides a high‑level interface for working with spreadsheet‑like data. It supports in‑memory and file‑backed storage, formulas, import/export, queries, and transactions.

**Key types:**

- `Workbook<S>` – the main entry point.
- `WorkbookError` – error type for all operations.
- `Transaction` – atomic grouping of operations.
- `Query` / `QueryBuilder` / `QueryAs` – fluent data access.

**Storage backends:**

- `InMemoryStorage` – ephemeral, for tests and simple apps.
- `FileStorage` – persistent, uses `.monumentum` files.

---

## Core Types

### `Workbook`

```rust
pub struct Workbook<S: StorageEngine> { /* private fields */ }
```

`Workbook` is generic over the storage engine `S`. It holds a `Catalog` of tables (sheets), a `FunctionRegistry`, and the storage engine.

#### Methods – Detailed with Examples

##### `catalog` / `catalog_mut`

```rust
let wb = Workbook::<InMemoryStorage>::new_in_memory();
let catalog: &Catalog = wb.catalog();
// Mutably access catalog to manipulate tables directly if needed.
let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
let catalog_mut: &mut Catalog = wb.catalog_mut();
```

##### `register_function`

Register a custom formula function.

```rust
fn my_add(args: &[Value]) -> Result<Value, FormulaError> {
    // implementation...
}

wb.register_function("MYADD", my_add);
```

##### `sheet` / `sheet_mut`

```rust
let table: &Table = wb.sheet("Sheet1")?;
let table_mut: &mut Table = wb.sheet_mut("Sheet1")?;
```

##### `persist_catalog`

For `FileStorage`, writes the catalog to disk. For `InMemoryStorage`, saves to memory (no‑op effectively).

```rust
wb.persist_catalog()?;
```

##### `set_allowed_values`

Restrict a column to a set of allowed values.

```rust
wb.set_allowed_values("Sheet1", "status", vec![
    Value::from("active"),
    Value::from("inactive"),
])?;
```

##### `validate_cell`

Manually validate a cell against its column’s constraints.

```rust
wb.validate_cell("Sheet1", 0, 0)?;
```

##### `protect_sheet` / `unprotect_sheet` / `is_sheet_protected`

```rust
wb.protect_sheet("Sheet1")?;
assert!(wb.is_sheet_protected("Sheet1")?);
wb.unprotect_sheet("Sheet1")?;
```

##### `query`

Returns a `Query` builder for fluent queries.

```rust
let query = wb.query("Sheet1");
let rows = query.filter(|r| r.get(0).map_or(false, |v| v > &Value::from(10))).fetch_all()?;
```

---

### `WorkbookError`

```rust
pub enum WorkbookError { /* variants */ }
```

Represents all possible errors from workbook operations. Each variant corresponds to a specific error condition.

#### Variants (with Display Strings)

| Variant                         | Display String / Description                    |
| ------------------------------- | ----------------------------------------------- |
| `CellTooNarrow`                 | `###: cell too narrow`                          |
| `FormatOutOfRange`              | `#FMT: value outside format limits`             |
| `NotAvailable`                  | `#N/A: not available`                           |
| `InvalidCharacter`              | `Err:501: invalid character`                    |
| `InvalidArgument`               | `Err:502: invalid argument`                     |
| `InvalidFloatingPointOperation` | `#NUM!: invalid floating point operation`       |
| `ParameterListError`            | `Err:504: parameter list error`                 |
| `PairMissing`                   | `Err:507/508: pair missing`                     |
| `MissingOperator`               | `Err:509: missing operator`                     |
| `MissingVariable`               | `Err:510: missing variable`                     |
| `MissingVariableForFunction`    | `Err:511: function requires more variables`     |
| `FormulaOverflow`               | `Err:512: formula overflow`                     |
| `StringOverflow`                | `Err:513: string overflow`                      |
| `InternalOverflow`              | `Err:514: internal overflow`                    |
| `InternalSyntaxError`           | `Err:515: internal syntax error`                |
| `MatrixExpected`                | `Err:516: matrix expected`                      |
| `UnknownCode`                   | `Err:517: unknown code`                         |
| `VariableNotAvailable`          | `Err:518: variable not available`               |
| `NoValue`                       | `#VALUE!: no value`                             |
| `Null`                          | `#NULL!: null`                                  |
| `CircularReference`             | `Err:522: circular reference`                   |
| `NoConvergence`                 | `Err:523: no convergence`                       |
| `InvalidReference`              | `#REF!: invalid reference`                      |
| `InvalidName`                   | `#NAME?: invalid names`                         |
| `ReferenceTooEncapsulated`      | `Err:527: reference too encapsulated`           |
| `AddInNotFound`                 | `Err:530: add-in not found`                     |
| `MacroNotFound`                 | `Err:531: macro not found`                      |
| `DivisionByZero`                | `#DIV/0!: division by zero`                     |
| `NestedArraysNotSupported`      | `Err:533: nested arrays not supported`          |
| `ArraySizeExceeded`             | `Err:538: array size exceeded`                  |
| `UnsupportedInlineArrayContent` | `Err:539: unsupported inline array content`     |
| `ExternalContentDisabled`       | `Err:540: external content disabled`            |
| `Db(DbError)`                   | Wraps database errors.                          |
| `Formula(FormulaError)`         | Wraps formula evaluation errors.                |
| `FileExists`                    | File already exists.                            |
| `InvalidExtension`              | Invalid file extension; expected `.monumentum`. |

#### Trait Implementations

- **`Display`** – human‑readable message (see table above).
- **`std::error::Error`** – `source()` returns the underlying `DbError` or `FormulaError` if applicable.
- **`MonumentumError`** – provides `kind()`, `message()`, `constraint()`, `table()`.
- **Conversions**:
  - `From<DbError>`
  - `From<FormulaError>`
  - `From<std::io::Error>`

#### Example: Error Handling

```rust
fn handle_error(e: WorkbookError) {
    match e {
        WorkbookError::Db(DbError::TableNotFound(name)) => println!("Sheet {name} not found"),
        WorkbookError::Formula(FormulaError::DivisionByZero) => println!("Division by zero"),
        WorkbookError::InvalidExtension => println!("Wrong file extension"),
        other => println!("{}", other),
    }
}
```

---

### `Transaction`

```rust
pub struct Transaction<'a, S: StorageEngine> { /* private fields */ }
```

Allows grouping multiple operations into a single atomic unit. When a transaction is created, a snapshot of the current catalog is taken. You can `commit` to persist changes, or `rollback` to discard them.

#### Methods

| Method         | Signature                                                  | Description                              |
| -------------- | ---------------------------------------------------------- | ---------------------------------------- |
| `begin`        | `pub fn begin(workbook: &'a mut Workbook<S>) -> Self`      | Creates a transaction from a workbook.   |
| `commit`       | `pub fn commit(mut self) -> Result<(), WorkbookError>`     | Persists changes and closes transaction. |
| `rollback`     | `pub fn rollback(mut self)`                                | Reverts changes and closes transaction.  |
| `workbook_mut` | `pub const fn workbook_mut(&mut self) -> &mut Workbook<S>` | Access the underlying workbook mutably.  |

#### Deref / DerefMut

`Transaction` dereferences to `Workbook<S>`, so you can call workbook methods directly on the transaction object.

#### Drop Behavior

If a transaction is dropped without calling `commit` or `rollback`, it automatically performs a **rollback**.

#### Example: Commit and Rollback

```rust
let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
wb.create_sheet("main", vec![ColumnDef::new("id", DataType::Integer)])?;

{
    let mut tx = Transaction::begin(&mut wb);
    tx.insert_row("main", vec![Value::from(1_i64)])?;
    tx.commit()?;   // changes are now permanent
}
assert_eq!(wb.row_count("main")?, 1);

{
    let mut tx = Transaction::begin(&mut wb);
    tx.insert_row("main", vec![Value::from(2_i64)])?;
    tx.rollback();  // row not added
}
assert_eq!(wb.row_count("main")?, 1);
```

---

## Module `menu`

The `menu` module groups together all the operations you would expect from a spreadsheet application: data manipulation, cell editing, file operations, formulas, import/export, row/column insertion, and sheet management.

### `menu::data`

These methods work with entire columns of data.

#### `sort_sheet`

```rust
pub fn sort_sheet(&mut self, sheet: &str, col_idx: usize, ascending: bool) -> Result<(), WorkbookError>
```

Sorts all rows by the values in the specified column.

- **Errors**:
  - `WorkbookError::Db(TableNotFound)` if sheet doesn’t exist.
  - `WorkbookError::InvalidReference` if `col_idx` is out of bounds.
  - `WorkbookError::Formula(...)` if any cell in the sheet is a formula.

**Example:**

```rust
let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
wb.create_sheet("nums", vec![ColumnDef::new("val", DataType::Integer)])?;
wb.insert_row("nums", vec![Value::from(5_i64)])?;
wb.insert_row("nums", vec![Value::from(3_i64)])?;
wb.insert_row("nums", vec![Value::from(8_i64)])?;

wb.sort_sheet("nums", 0, true)?; // ascending
assert_eq!(wb.get_cell("nums", 0, 0), Some(&Value::from(3_i64)));
```

#### `filter_sheet`

```rust
pub fn filter_sheet(&self, sheet: &str, col_idx: usize, value: &Value) -> Result<Vec<Row>, WorkbookError>
```

Returns all rows where the value in column `col_idx` equals `value`.

**Example:**

```rust
let wb = /* ... */;
let matches = wb.filter_sheet("nums", 0, &Value::from(5_i64))?;
assert_eq!(matches.len(), 1);
```

#### `distinct_values`

```rust
pub fn distinct_values(&self, sheet: &str, col_idx: usize) -> Result<Vec<Value>, WorkbookError>
```

Returns a sorted, deduplicated list of all values in the column.

**Example:**

```rust
let distinct = wb.distinct_values("nums", 0)?;
// e.g., [3, 5, 8]
```

---

### `menu::edit`

Methods for cell‑level access and modification.

#### `row_count`, `column_count`

```rust
let rows = wb.row_count("Sheet1")?;
let cols = wb.column_count("Sheet1")?;
```

#### `get_cell`

```rust
let value: Option<&Value> = wb.get_cell("Sheet1", row, col);
```

Returns `None` if sheet, row, or column does not exist.

#### `set_cell`

```rust
wb.set_cell("Sheet1", row, col, Value::from(42_i64))?;
```

Validates the new value against column constraints (type, allowed values, etc.).

#### `replace_in_sheet`

```rust
let count = wb.replace_in_sheet("Sheet1", &Value::from(1_i64), &Value::from(100_i64))?;
```

Replaces all occurrences of `old_value` with `new_value`. Returns the number of replacements.

#### `find_in_sheet`

```rust
let positions = wb.find_in_sheet("Sheet1", &Value::from("needle"))?;
// positions: Vec<(row_idx, col_idx)>
```

#### `get_cell_by_name`, `set_cell_by_name`

Convenience methods that use column names instead of indices.

#### `get_row`

```rust
let row: &Row = wb.get_row("Sheet1", 0)?;
```

---

### `menu::export`

Export sheet data to CSV or JSON.

#### `export_csv`

```rust
pub fn export_csv<W: Write>(&self, sheet: &str, writer: W) -> Result<(), WorkbookError>
```

Writes a CSV representation of the sheet to `writer`. The first line is the header (column names). Blob values are not supported and will cause an error.

**Example:**

```rust
let mut buffer = Vec::new();
wb.export_csv("Sheet1", &mut buffer)?;
let csv_text = String::from_utf8(buffer)?;
println!("{}", csv_text);
```

#### `export_json`

```rust
pub fn export_json<W: Write>(&self, sheet: &str, writer: W) -> Result<(), WorkbookError>
```

Writes a JSON array of objects, where each object represents a row. Keys are column names. Blob values cause an error.

**Example:**

```rust
let mut buffer = Vec::new();
wb.export_json("Sheet1", &mut buffer)?;
let json_text = String::from_utf8(buffer)?;
println!("{}", json_text);
```

---

### `menu::file`

Methods specific to `FileStorage` and `InMemoryStorage`.

#### `Workbook<FileStorage>`

| Method        | Description                                                                            |
| ------------- | -------------------------------------------------------------------------------------- |
| `open`        | Opens an existing `.monumentum` file. If not present, creates an empty one? (see test) |
| `create_new`  | Creates a new empty workbook file. Errors if file exists or bad extension.             |
| `save`        | Saves the current catalog and checkpoints.                                             |
| `save_as`     | Saves to a new file without affecting the original.                                    |
| `save_a_copy` | Saves a copy to a new file; original remains open and unchanged.                       |
| `reload`      | Discards unsaved changes and reloads from disk.                                        |
| `close`       | Closes the file and releases any lock.                                                 |

**Examples:**

```rust
let path = std::path::Path::new("book.monumentum");
let mut wb = Workbook::<FileStorage>::create_new(path)?;
wb.create_sheet("Data", vec![ColumnDef::new("x", DataType::Integer)])?;
wb.save()?;
wb.close()?;

// Later...
let wb2 = Workbook::<FileStorage>::open(path)?;
// ...
wb2.close()?;
```

#### `Workbook<InMemoryStorage>`

| Method           | Description                                                        |
| ---------------- | ------------------------------------------------------------------ |
| `new_in_memory`  | Creates an empty in‑memory workbook.                               |
| `load_in_memory` | Creates a workbook from an existing `Catalog`.                     |
| `save`           | Saves catalog to memory (no‑op).                                   |
| `reload`         | Reloads from memory (effectively no‑op unless you changed memory). |
| `close`          | No‑op.                                                             |

---

### `menu::formula`

Work with formulas.

#### `set_formula`

```rust
wb.set_formula("Sheet1", row_idx, col_idx, "SUM(A1:A10)")?;
```

The formula string is stored as a `Value::Formula`.

#### `get_cell_value`

```rust
let result: Value = wb.get_cell_value("Sheet1", row_idx, col_idx)?;
```

Evaluates the cell. If the cell contains a formula, it is parsed, evaluated, and the result is returned. Formula dependencies are resolved recursively, with circular reference detection.

**Example:**

```rust
wb.set_formula("Sheet1", 2, 1, "SUM(B1:B2)")?;
let sum = wb.get_cell_value("Sheet1", 2, 1)?;
// sum is Value::Integer(30) if B1=10, B2=20
```

---

### `menu::import`

Import data from CSV or JSON.

#### `import_csv`

```rust
pub fn import_csv<R: Read>(&mut self, sheet: &str, reader: R) -> Result<(), WorkbookError>
```

Replaces all rows in the sheet with data from the CSV stream. The first line must contain the exact column names, and each subsequent line must have the same number of fields.

**Example:**

```rust
let csv_data = b"name,age\nAlice,30\nBob,25\n";
wb.import_csv("people", Cursor::new(csv_data))?;
```

#### `import_json`

```rust
pub fn import_json<R: Read>(&mut self, sheet: &str, reader: R) -> Result<(), WorkbookError>
```

Replaces rows with data from a JSON array of objects. Each object must contain all required fields (by column name).

**Example:**

```rust
let json_data = br#"[{"name":"Carol","age":28}]"#;
wb.import_json("people", Cursor::new(json_data))?;
```

---

### `menu::insert`

Add or remove rows and columns.

#### `insert_row_at`

```rust
pub fn insert_row_at(&mut self, sheet: &str, index: usize, values: Vec<Value>) -> Result<(), WorkbookError>
```

Inserts a new row at the given index. If `index` is greater than the current number of rows, it is appended at the end. The number of values must match the column count.

#### `insert_column`

```rust
pub fn insert_column(&mut self, sheet: &str, index: usize, col_def: &ColumnDef) -> Result<(), WorkbookError>
```

Inserts a new column at the specified index. Existing rows are filled with the column’s default value (if any) or `Value::Null`.

#### `delete_column`

```rust
pub fn delete_column(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError>
```

Removes the column at `index`. You cannot delete the last remaining column.

---

### `menu::sheet`

Manage sheets.

#### `sheet_names`

```rust
let names: Vec<String> = wb.sheet_names();
```

#### `create_sheet`

```rust
wb.create_sheet("NewSheet", vec![ColumnDef::new("id", DataType::Integer)])?;
```

#### `drop_sheet`

```rust
wb.drop_sheet("OldSheet")?;
```

#### `rename_sheet`

```rust
wb.rename_sheet("OldName", "NewName")?;
```

#### `insert_row` (append)

```rust
wb.insert_row("Sheet1", vec![Value::from(1_i64)])?;
```

#### `delete_row`

```rust
wb.delete_row("Sheet1", 0)?;
```

#### `clear_sheet`

```rust
wb.clear_sheet("Sheet1")?; // removes all rows, keeps columns
```

---

## Module `query`

The `query` module provides a SQL‑like builder for retrieving data.

### `Query`

The basic query object. It is created via `Workbook::query` or `Query::new`.

#### Example: Basic Query

```rust
let rows = wb.query("Sheet1")
    .filter(|row| row.get(0).map_or(false, |v| v > &Value::from(10)))
    .order_by(1, true)
    .limit(5)
    .fetch_all()?;
```

#### Aggregate Functions

`sum`, `avg`, `min`, `max` are available on `Query`.

```rust
let total = wb.query("Sheet1").sum(1)?;
let avg   = wb.query("Sheet1").avg(1)?;
let min   = wb.query("Sheet1").min(1)?;
let max   = wb.query("Sheet1").max(1)?;
```

#### Mapping

Use `map` or `try_map` to transform rows.

```rust
let names: Vec<String> = wb.query("Sheet1")
    .map(|row| {
        let value = row.get(0).cloned().unwrap_or(Value::Null);
        value.to_string()
    })
    .fetch_all()?;
```

### `QueryBuilder`

Provides a more ergonomic way to build queries, especially when you need to chain many options.

```rust
let query = QueryBuilder::new(&wb, "Sheet1")
    .select(vec![0, 2])
    .filter(|row| /* condition */)
    .order_by(0, true)
    .limit(10)
    .build();
```

It can also produce typed queries with `build_query_as`.

### `QueryAs`

Allows mapping each row to a custom type that implements `FromRow`.

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: i64,
}

impl FromRow for Person {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        let name = String::from_value(row.get(0).ok_or(WorkbookError::InvalidReference)?)?;
        let age = i64::from_value(row.get(1).ok_or(WorkbookError::InvalidReference)?)?;
        Ok(Person { name, age })
    }
}

let people: Vec<Person> = wb.query_as::<Person>("Sheet1").fetch_all()?;
```

(Note: `QueryAs::new` is also available.)

### `Map`

The result of `map`/`try_map`. It supports further mapping and fetching.

```rust
let mapped = wb.query("Sheet1")
    .map(|row| row.get(0).cloned().unwrap_or(Value::Null))
    .map(|v| v.to_string())
    .fetch_all()?;
```

### `FromRow` and `FromValue`

Traits for converting `Row` and `Value` into Rust types.

**Built‑in implementations** exist for:

- `Value`
- `String`
- `i64`
- `f64`
- `bool`
- `Vec<Value>`
- Tuples up to 9 elements

You can implement `FromRow` for your own structs.

---

## Module `transaction`

Already covered in [Core Types](#core-types). The module simply exports the `Transaction` type.

---

## Advanced Topics

### Custom Functions

You can register custom formula functions that can be used in formulas.

```rust
fn custom_double(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity("DOUBLE".into()));
    }
    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer((i.as_i64() * 2).into())),
        Value::Float(f)   => Ok(Value::Float((f.as_f64() * 2.0).try_into().unwrap())),
        _ => Err(FormulaError::TypeMismatch("expected number".into())),
    }
}

let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
wb.register_function("DOUBLE", custom_double);
```

Now you can use `=DOUBLE(A1)` in a formula.

### Error Handling Patterns

Because `WorkbookError` is a rich enum, you can match on specific error conditions.

```rust
match wb.set_cell("Sheet1", 0, 0, Value::from("wrong type")) {
    Ok(()) => println!("Set succeeded"),
    Err(WorkbookError::Db(DbError::TypeMismatch(_))) => println!("Type mismatch!"),
    Err(e) => println!("Other error: {}", e),
}
```

### Protecting Sheets

You can mark a sheet as read‑only to prevent accidental modifications.

```rust
wb.protect_sheet("Config")?;
// Any attempt to modify will return an error
let result = wb.set_cell("Config", 0, 0, Value::from(1_i64));
assert!(result.is_err());
```

---

## Complete Example Application

Below is a complete, runnable example that demonstrates many features together.

```rust
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::{Workbook, WorkbookError, transaction::Transaction};
use std::io::Cursor;

fn main() -> Result<(), WorkbookError> {
    // Create an in-memory workbook
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();

    // Create a sheet
    wb.create_sheet("employees", vec![
        ColumnDef::new("name", DataType::Text),
        ColumnDef::new("age", DataType::Integer),
        ColumnDef::new("salary", DataType::Float),
    ])?;

    // Insert data
    wb.insert_row("employees", vec![
        Value::from("Alice"),
        Value::from(30_i64),
        Value::try_from(50000.0)?,
    ])?;
    wb.insert_row("employees", vec![
        Value::from("Bob"),
        Value::from(25_i64),
        Value::try_from(45000.0)?,
    ])?;

    // Add a formula to compute average salary
    let avg_row = wb.row_count("employees")?;
    wb.insert_row("employees", vec![
        Value::from("Average"),
        Value::Null,
        Value::Null,
    ])?;
    wb.set_formula("employees", avg_row, 2, "AVERAGE(C1:C2)")?;

    // Evaluate formula
    let avg = wb.get_cell_value("employees", avg_row, 2)?;
    println!("Average salary: {:?}", avg);

    // Query
    let adults = wb.query("employees")
        .filter(|row| row.get(1).map_or(false, |v| v > &Value::from(20_i64)))
        .fetch_all()?;
    println!("Adults: {:?}", adults);

    // Export to CSV
    let mut csv = Vec::new();
    wb.export_csv("employees", &mut csv)?;
    println!("CSV:\n{}", String::from_utf8(csv).unwrap());

    // Transaction
    {
        let mut tx = Transaction::begin(&mut wb);
        tx.insert_row("employees", vec![
            Value::from("Carol"),
            Value::from(28_i64),
            Value::try_from(48000.0)?,
        ])?;
        // If we forget to commit, it will rollback automatically
        // tx.commit()?;
    }
    println!("Rows after rolled-back transaction: {}", wb.row_count("employees")?);

    Ok(())
}
```
