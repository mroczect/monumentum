# Monumentum Workbook API Documentation

This document provides a complete API reference for the `monumentum_workbook` crate. It describes all public types, traits, functions, and methods exposed by the crate, organized by module.

---

## Table of Contents

1. [Core Types](#core-types)
   - [`Workbook`](#workbook)
   - [`WorkbookError`](#workbookerror)
   - [`Transaction`](#transaction)
2. [Module `menu`](#module-menu)
   - [`data`](#menu-data)
   - [`edit`](#menu-edit)
   - [`export`](#menu-export)
   - [`file`](#menu-file)
   - [`formula`](#menu-formula)
   - [`import`](#menu-import)
   - [`insert`](#menu-insert)
   - [`sheet`](#menu-sheet)
3. [Module `query`](#module-query)
   - [`Query`](#query)
   - [`QueryBuilder`](#querybuilder)
   - [`QueryAs`](#queryas)
   - [`Map`](#map)
   - [`FromRow` and `FromValue`](#fromrow-and-fromvalue)
4. [Module `transaction`](#module-transaction)

---

## Core Types

### `Workbook`

A generic workbook that holds tables (sheets) and provides operations to manipulate them. The generic parameter `S` must implement `StorageEngine` (e.g., `FileStorage` or `InMemoryStorage`).

```rust
pub struct Workbook<S: StorageEngine> { /* private fields */ }
```

#### Methods

| Method               | Signature                                                                                                            | Description                                                                                |
| -------------------- | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `default_registry`   | `pub(crate) fn default_registry() -> FunctionRegistry`                                                               | (internal) Creates a function registry with all built‑in functions registered.             |
| `catalog`            | `pub const fn catalog(&self) -> &Catalog`                                                                            | Returns a reference to the underlying catalog.                                             |
| `catalog_mut`        | `pub const fn catalog_mut(&mut self) -> &mut Catalog`                                                                | Returns a mutable reference to the catalog.                                                |
| `register_function`  | `pub fn register_function(&mut self, name: &str, func: FunctionImpl)`                                                | Registers a custom function in the workbook’s function registry.                           |
| `functions`          | `pub const fn functions(&self) -> &FunctionRegistry`                                                                 | Returns a reference to the function registry.                                              |
| `sheet`              | `pub fn sheet(&self, name: &str) -> Result<&Table, WorkbookError>`                                                   | Returns an immutable reference to the sheet with the given name, or an error if not found. |
| `sheet_mut`          | `pub fn sheet_mut(&mut self, name: &str) -> Result<&mut Table, WorkbookError>`                                       | Returns a mutable reference to the sheet, or an error.                                     |
| `persist_catalog`    | `pub fn persist_catalog(&mut self) -> Result<(), WorkbookError>`                                                     | Saves the current catalog to the underlying storage.                                       |
| `set_allowed_values` | `pub fn set_allowed_values(&mut self, sheet: &str, col_name: &str, values: Vec<Value>) -> Result<(), WorkbookError>` | Sets the allowed values for a column.                                                      |
| `validate_cell`      | `pub fn validate_cell(&self, sheet: &str, row_idx: usize, col_idx: usize) -> Result<(), WorkbookError>`              | Validates the value in a specific cell against the column constraints.                     |
| `protect_sheet`      | `pub fn protect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>`                                          | Marks a sheet as read‑only.                                                                |
| `unprotect_sheet`    | `pub fn unprotect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>`                                        | Removes read‑only protection from a sheet.                                                 |
| `is_sheet_protected` | `pub fn is_sheet_protected(&self, sheet: &str) -> Result<bool, WorkbookError>`                                       | Checks if a sheet is read‑only.                                                            |
| `ensure_writable`    | `pub(crate) fn ensure_writable(&self, sheet: &str) -> Result<(), WorkbookError>`                                     | (internal) Ensures the sheet is not read‑only.                                             |
| `query`              | `pub fn query<'a>(&'a self, sheet: &str) -> query::Query<'a, S>`                                                     | Returns a `Query` builder for the given sheet.                                             |

---

### `WorkbookError`

The error type for all workbook operations.

```rust
pub enum WorkbookError { /* variants */ }
```

#### Variants

| Variant                         | Description                                        |
| ------------------------------- | -------------------------------------------------- |
| `CellTooNarrow`                 | `###` – cell too narrow to display value.          |
| `FormatOutOfRange`              | `#FMT` – value outside format limits.              |
| `NotAvailable`                  | `#N/A` – not available.                            |
| `InvalidCharacter`              | `Err:501` – invalid character.                     |
| `InvalidArgument`               | `Err:502` – invalid argument.                      |
| `InvalidFloatingPointOperation` | `#NUM!` – invalid floating point operation.        |
| `ParameterListError`            | `Err:504` – parameter list error.                  |
| `PairMissing`                   | `Err:507/508` – pair missing.                      |
| `MissingOperator`               | `Err:509` – missing operator.                      |
| `MissingVariable`               | `Err:510` – missing variable.                      |
| `MissingVariableForFunction`    | `Err:511` – function requires more variables.      |
| `FormulaOverflow`               | `Err:512` – formula overflow.                      |
| `StringOverflow`                | `Err:513` – string overflow.                       |
| `InternalOverflow`              | `Err:514` – internal overflow.                     |
| `InternalSyntaxError`           | `Err:515` – internal syntax error.                 |
| `MatrixExpected`                | `Err:516` – matrix expected.                       |
| `UnknownCode`                   | `Err:517` – unknown code.                          |
| `VariableNotAvailable`          | `Err:518` – variable not available.                |
| `NoValue`                       | `#VALUE!` – no value.                              |
| `Null`                          | `#NULL!` – null value.                             |
| `CircularReference`             | `Err:522` – circular reference.                    |
| `NoConvergence`                 | `Err:523` – no convergence.                        |
| `InvalidReference`              | `#REF!` – invalid reference.                       |
| `InvalidName`                   | `#NAME?` – invalid names.                          |
| `ReferenceTooEncapsulated`      | `Err:527` – reference too encapsulated.            |
| `AddInNotFound`                 | `Err:530` – add‑in not found.                      |
| `MacroNotFound`                 | `Err:531` – macro not found.                       |
| `DivisionByZero`                | `#DIV/0!` – division by zero.                      |
| `NestedArraysNotSupported`      | `Err:533` – nested arrays not supported.           |
| `ArraySizeExceeded`             | `Err:538` – array size exceeded.                   |
| `UnsupportedInlineArrayContent` | `Err:539` – unsupported inline array content.      |
| `ExternalContentDisabled`       | `Err:540` – external content disabled.             |
| `Db`                            | Wraps a `monumentum_db::error::DbError`.           |
| `Formula`                       | Wraps a `monumentum_query::formula::FormulaError`. |
| `FileExists`                    | File already exists.                               |
| `InvalidExtension`              | Invalid file extension; expected `.monumentum`.    |

#### Trait Implementations

- `Display` – Human‑readable error message.
- `Error` – Implements `std::error::Error`; `source()` returns underlying `DbError` or `FormulaError` if present.
- `MonumentumError` – Provides `kind()`, `message()`, `constraint()`, and `table()`.
- `From<DbError>`, `From<FormulaError>`, `From<std::io::Error>` – Conversions into `WorkbookError`.

---

### `Transaction`

Provides transactional semantics for a workbook. All modifications during the transaction can be rolled back or committed.

```rust
pub struct Transaction<'a, S: StorageEngine> { /* private fields */ }
```

#### Methods

| Method         | Signature                                                  | Description                                                          |
| -------------- | ---------------------------------------------------------- | -------------------------------------------------------------------- |
| `begin`        | `pub fn begin(workbook: &'a mut Workbook<S>) -> Self`      | Starts a new transaction; creates a snapshot of the current catalog. |
| `commit`       | `pub fn commit(mut self) -> Result<(), WorkbookError>`     | Commits the transaction by persisting the current catalog.           |
| `rollback`     | `pub fn rollback(mut self)`                                | Rolls back all changes, restoring the snapshot.                      |
| `workbook_mut` | `pub const fn workbook_mut(&mut self) -> &mut Workbook<S>` | Returns a mutable reference to the underlying workbook.              |

#### Deref / DerefMut

`Transaction` dereferences to `Workbook<S>`, allowing direct use of workbook methods within the transaction.

#### Drop Behavior

If the transaction is dropped without calling `commit` or `rollback`, it automatically rolls back.

---

## Module `menu`

The `menu` module contains submodules that implement various workbook operations.

### `menu::data`

Functions for data manipulation (sorting, filtering, distinct values).

#### Methods on `Workbook`

| Method            | Signature                                                                                                   | Description                                                             |
| ----------------- | ----------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `sort_sheet`      | `pub fn sort_sheet(&mut self, sheet: &str, col_idx: usize, ascending: bool) -> Result<(), WorkbookError>`   | Sorts rows by the specified column. Rejects sheets containing formulas. |
| `filter_sheet`    | `pub fn filter_sheet(&self, sheet: &str, col_idx: usize, value: &Value) -> Result<Vec<Row>, WorkbookError>` | Returns rows where the column value equals the given value.             |
| `distinct_values` | `pub fn distinct_values(&self, sheet: &str, col_idx: usize) -> Result<Vec<Value>, WorkbookError>`           | Returns a sorted, de‑duplicated list of values in a column.             |

---

### `menu::edit`

Methods for reading and modifying individual cells.

#### Methods on `Workbook`

| Method             | Signature                                                                                                                    | Description                                                                                   |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `row_count`        | `pub fn row_count(&self, sheet: &str) -> Result<usize, WorkbookError>`                                                       | Returns the number of rows in a sheet.                                                        |
| `column_count`     | `pub fn column_count(&self, sheet: &str) -> Result<usize, WorkbookError>`                                                    | Returns the number of columns in a sheet.                                                     |
| `get_cell`         | `pub fn get_cell(&self, sheet: &str, row_idx: usize, col_idx: usize) -> Option<&Value>`                                      | Returns a reference to a cell value, or `None` if out of bounds.                              |
| `set_cell`         | `pub fn set_cell(&mut self, sheet: &str, row_idx: usize, col_idx: usize, value: Value) -> Result<(), WorkbookError>`         | Sets the value of a cell, validating constraints.                                             |
| `replace_in_sheet` | `pub fn replace_in_sheet(&mut self, sheet: &str, old_value: &Value, new_value: &Value) -> Result<usize, WorkbookError>`      | Replaces all occurrences of `old_value` with `new_value`. Returns the number of replacements. |
| `find_in_sheet`    | `pub fn find_in_sheet(&self, sheet: &str, value: &Value) -> Result<Vec<(usize, usize)>, WorkbookError>`                      | Returns coordinates of all cells matching the value.                                          |
| `get_cell_by_name` | `pub fn get_cell_by_name(&self, sheet: &str, row_idx: usize, col_name: &str) -> Result<Value, WorkbookError>`                | Gets a cell value by column name.                                                             |
| `set_cell_by_name` | `pub fn set_cell_by_name(&mut self, sheet: &str, row_idx: usize, col_name: &str, value: Value) -> Result<(), WorkbookError>` | Sets a cell value by column name.                                                             |
| `get_row`          | `pub fn get_row(&self, sheet: &str, row_idx: usize) -> Result<&Row, WorkbookError>`                                          | Returns a reference to a row.                                                                 |

---

### `menu::export`

Methods for exporting sheets to CSV or JSON.

#### Methods on `Workbook`

| Method        | Signature                                                                                      | Description                                                              |
| ------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `export_csv`  | `pub fn export_csv<W: Write>(&self, sheet: &str, mut writer: W) -> Result<(), WorkbookError>`  | Writes the sheet as CSV to the writer. Blob values cause an error.       |
| `export_json` | `pub fn export_json<W: Write>(&self, sheet: &str, mut writer: W) -> Result<(), WorkbookError>` | Writes the sheet as a JSON array of objects. Blob values cause an error. |

---

### `menu::file`

Methods for creating, opening, saving, and managing workbooks stored on disk.

#### `Workbook<FileStorage>`

| Method        | Signature                                                             | Description                                                               |
| ------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `open`        | `pub fn open(path: &Path) -> Result<Self, WorkbookError>`             | Opens an existing workbook from a `.monumentum` file.                     |
| `create_new`  | `pub fn create_new(path: &Path) -> Result<Self, WorkbookError>`       | Creates a new empty workbook file. Fails if the file already exists.      |
| `save`        | `pub fn save(&mut self) -> Result<(), WorkbookError>`                 | Saves the current workbook to the same file, including a checkpoint.      |
| `save_as`     | `pub fn save_as(&mut self, path: &Path) -> Result<(), WorkbookError>` | Saves the workbook to a new file without changing the original file path. |
| `save_a_copy` | `pub fn save_a_copy(&self, path: &Path) -> Result<(), WorkbookError>` | Saves a copy of the workbook to a new file.                               |
| `reload`      | `pub fn reload(&mut self) -> Result<(), WorkbookError>`               | Reloads the workbook from disk, discarding unsaved changes.               |
| `close`       | `pub fn close(self) -> Result<(), WorkbookError>`                     | Closes the underlying file storage.                                       |

#### `Workbook<InMemoryStorage>`

| Method           | Signature                                               | Description                                             |
| ---------------- | ------------------------------------------------------- | ------------------------------------------------------- |
| `new_in_memory`  | `pub fn new_in_memory() -> Self`                        | Creates a new empty in‑memory workbook.                 |
| `load_in_memory` | `pub fn load_in_memory(catalog: Catalog) -> Self`       | Creates an in‑memory workbook from an existing catalog. |
| `save`           | `pub fn save(&mut self) -> Result<(), WorkbookError>`   | Saves the catalog to in‑memory storage (no‑op).         |
| `reload`         | `pub fn reload(&mut self) -> Result<(), WorkbookError>` | Reloads the catalog from in‑memory storage.             |
| `close`          | `pub fn close(self) -> Result<(), WorkbookError>`       | Closes (no‑op).                                         |

---

### `menu::formula`

Methods for evaluating formulas.

#### Methods on `Workbook`

| Method           | Signature                                                                                                                | Description                                                      |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------- |
| `set_formula`    | `pub fn set_formula(&mut self, sheet: &str, row_idx: usize, col_idx: usize, formula: &str) -> Result<(), WorkbookError>` | Sets a cell to a formula string.                                 |
| `get_cell_value` | `pub fn get_cell_value(&self, sheet: &str, row_idx: usize, col_idx: usize) -> Result<Value, WorkbookError>`              | Evaluates the cell, following formula dependencies if necessary. |

---

### `menu::import`

Methods for importing CSV or JSON data into a sheet.

#### Methods on `Workbook`

| Method        | Signature                                                                                         | Description                                                                                            |
| ------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `import_csv`  | `pub fn import_csv<R: Read>(&mut self, sheet: &str, mut reader: R) -> Result<(), WorkbookError>`  | Replaces the sheet’s rows with data read from CSV. The first line must match the column names exactly. |
| `import_json` | `pub fn import_json<R: Read>(&mut self, sheet: &str, mut reader: R) -> Result<(), WorkbookError>` | Replaces the sheet’s rows with data from a JSON array. Each object must have all required fields.      |

---

### `menu::insert`

Methods for inserting or deleting rows and columns.

#### Methods on `Workbook`

| Method          | Signature                                                                                                      | Description                                                                                      |
| --------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `insert_row_at` | `pub fn insert_row_at(&mut self, sheet: &str, index: usize, values: Vec<Value>) -> Result<(), WorkbookError>`  | Inserts a new row at the given index.                                                            |
| `insert_column` | `pub fn insert_column(&mut self, sheet: &str, index: usize, col_def: &ColumnDef) -> Result<(), WorkbookError>` | Inserts a new column at the given index, filling existing rows with the default value or `Null`. |
| `delete_column` | `pub fn delete_column(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError>`                      | Deletes a column at the given index. Cannot delete the last remaining column.                    |

---

### `menu::sheet`

Methods for managing sheets (tables).

#### Methods on `Workbook`

| Method         | Signature                                                                                          | Description                                     |
| -------------- | -------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| `sheet_names`  | `pub fn sheet_names(&self) -> Vec<String>`                                                         | Returns a list of all sheet names.              |
| `create_sheet` | `pub fn create_sheet(&mut self, name: &str, columns: Vec<ColumnDef>) -> Result<(), WorkbookError>` | Creates a new sheet with the specified columns. |
| `drop_sheet`   | `pub fn drop_sheet(&mut self, name: &str) -> Result<(), WorkbookError>`                            | Deletes a sheet.                                |
| `rename_sheet` | `pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), WorkbookError>`      | Renames a sheet.                                |
| `insert_row`   | `pub fn insert_row(&mut self, sheet: &str, values: Vec<Value>) -> Result<(), WorkbookError>`       | Appends a new row at the end.                   |
| `delete_row`   | `pub fn delete_row(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError>`             | Removes the row at the given index.             |
| `clear_sheet`  | `pub fn clear_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError>`                          | Removes all rows from the sheet.                |

---

## Module `query`

The `query` module provides a fluent API for querying and transforming data in a workbook.

### `Query`

```rust
pub struct Query<'a, S: StorageEngine> { /* private fields */ }
```

#### Methods

| Method             | Signature                                                                    | Description                                       |
| ------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------- |
| `new`              | `pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self`    | Creates a new query for a sheet.                  |
| `select`           | `pub fn select(mut self, columns: Vec<usize>) -> Self`                       | Selects specific column indices.                  |
| `select_by_names`  | `pub fn select_by_names(mut self, columns: &[&str]) -> Self`                 | Selects columns by name (case‑insensitive).       |
| `filter`           | `pub fn filter(mut self, predicate: impl Fn(&Row) -> bool + 'a) -> Self`     | Filters rows using a closure.                     |
| `filter_by_column` | `pub fn filter_by_column<F>(mut self, col_name: &str, predicate: F) -> Self` | Filters rows based on a value in a named column.  |
| `order_by`         | `pub const fn order_by(mut self, col: usize, ascending: bool) -> Self`       | Sorts by a column index.                          |
| `order_by_name`    | `pub fn order_by_name(mut self, col_name: &str, ascending: bool) -> Self`    | Sorts by a column name.                           |
| `limit`            | `pub const fn limit(mut self, n: usize) -> Self`                             | Limits the number of results.                     |
| `fetch_all`        | `pub fn fetch_all(self) -> Result<Vec<Row>, WorkbookError>`                  | Executes the query and returns all matching rows. |
| `fetch_one`        | `pub fn fetch_one(self) -> Result<Row, WorkbookError>`                       | Returns the first row or an error if none.        |
| `fetch_optional`   | `pub fn fetch_optional(self) -> Result<Option<Row>, WorkbookError>`          | Returns the first row as `Option`.                |
| `count`            | `pub fn count(self) -> Result<usize, WorkbookError>`                         | Returns the number of rows.                       |
| `sum`              | `pub fn sum(self, col: usize) -> Result<Value, WorkbookError>`               | Computes the sum of a numeric column.             |
| `avg`              | `pub fn avg(self, col: usize) -> Result<Value, WorkbookError>`               | Computes the average of a numeric column.         |
| `min`              | `pub fn min(self, col: usize) -> Result<Value, WorkbookError>`               | Returns the minimum value in a column.            |
| `max`              | `pub fn max(self, col: usize) -> Result<Value, WorkbookError>`               | Returns the maximum value in a column.            |
| `map`              | `pub fn map<G, O>(self, f: G) -> Map<'a, S, ...>`                            | Transforms each row using a closure.              |
| `try_map`          | `pub const fn try_map<G, O>(self, f: G) -> Map<'a, S, G>`                    | Transforms rows with a fallible closure.          |

### `QueryBuilder`

Provides a more ergonomic, builder‑style query construction that can also produce `QueryAs` for typed output.

```rust
pub struct QueryBuilder<'a, S: StorageEngine> { /* private fields */ }
```

#### Methods

| Method            | Signature                                                                 | Description                                      |
| ----------------- | ------------------------------------------------------------------------- | ------------------------------------------------ |
| `new`             | `pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self` | Starts building a query.                         |
| `select`          | `pub fn select(mut self, columns: Vec<usize>) -> Self`                    | Selects column indices.                          |
| `filter`          | `pub fn filter<F>(mut self, predicate: F) -> Self`                        | Adds a filter condition.                         |
| `order_by`        | `pub const fn order_by(mut self, col: usize, ascending: bool) -> Self`    | Sets sort order by column index.                 |
| `limit`           | `pub const fn limit(mut self, n: usize) -> Self`                          | Sets a limit.                                    |
| `build`           | `pub fn build(self) -> Query<'a, S>`                                      | Converts the builder into a `Query`.             |
| `build_query_as`  | `pub fn build_query_as<O: FromRow>(self) -> QueryAs<'a, S, O>`            | Converts into a `QueryAs` for typed row mapping. |
| `select_by_names` | `pub fn select_by_names(mut self, columns: &[&str]) -> Self`              | Selects columns by name.                         |
| `order_by_name`   | `pub fn order_by_name(mut self, col_name: &str, ascending: bool) -> Self` | Sorts by column name.                            |

### `QueryAs`

Typed query that maps each row to a user‑defined type implementing `FromRow`.

```rust
pub struct QueryAs<'a, S: StorageEngine, O> { /* private fields */ }
```

#### Methods

| Method            | Signature                                                                 | Description                                            |
| ----------------- | ------------------------------------------------------------------------- | ------------------------------------------------------ |
| `new`             | `pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self` | Creates a new typed query.                             |
| `select`          | `pub fn select(mut self, columns: Vec<usize>) -> Self`                    | Selects columns (affects the row passed to `FromRow`). |
| `filter`          | `pub fn filter(mut self, predicate: impl Fn(&Row) -> bool + 'a) -> Self`  | Adds a filter.                                         |
| `order_by`        | `pub fn order_by(mut self, col: usize, ascending: bool) -> Self`          | Sorts by column index.                                 |
| `limit`           | `pub fn limit(mut self, n: usize) -> Self`                                | Limits the result count.                               |
| `fetch_all`       | `pub fn fetch_all(self) -> Result<Vec<O>, WorkbookError>`                 | Executes and returns all mapped objects.               |
| `fetch_one`       | `pub fn fetch_one(self) -> Result<O, WorkbookError>`                      | Returns the first mapped object.                       |
| `fetch_optional`  | `pub fn fetch_optional(self) -> Result<Option<O>, WorkbookError>`         | Returns the first mapped object as `Option`.           |
| `select_by_names` | `pub fn select_by_names(mut self, columns: &[&str]) -> Self`              | Selects columns by name.                               |
| `order_by_name`   | `pub fn order_by_name(mut self, col_name: &str, ascending: bool) -> Self` | Sorts by column name.                                  |

### `Map`

Result of calling `map` / `try_map` on a `Query` or another `Map`.

```rust
pub struct Map<'a, S: StorageEngine, F> { /* private fields */ }
```

#### Methods

| Method           | Signature                                                         | Description                                                       |
| ---------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------- |
| `fetch_all`      | `pub fn fetch_all(self) -> Result<Vec<O>, WorkbookError>`         | Executes the underlying query and applies the mapper to each row. |
| `fetch_one`      | `pub fn fetch_one(self) -> Result<O, WorkbookError>`              | Returns the first mapped result.                                  |
| `fetch_optional` | `pub fn fetch_optional(self) -> Result<Option<O>, WorkbookError>` | Returns the first mapped result as `Option`.                      |
| `map`            | `pub fn map<G, P>(self, g: G) -> Map<'a, S, ...>`                 | Chains another mapping.                                           |
| `try_map`        | `pub fn try_map<G, P>(self, g: G) -> Map<'a, S, ...>`             | Chains another fallible mapping.                                  |

### `FromRow` and `FromValue`

Traits for converting a `Row` or `Value` into a custom type.

#### `FromRow`

```rust
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, WorkbookError>;
}
```

Implementations are provided for:

- `Value`
- `String`
- `i64`
- `f64`
- `bool`
- `Vec<Value>`
- Tuples up to 9 elements

#### `FromValue`

```rust
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, WorkbookError>;
}
```

Implementations are provided for:

- `Value`
- `String`
- `i64`
- `f64`
- `bool`

---

## Module `transaction`

The `transaction` module contains the `Transaction` type already described in [Core Types](#core-types). It provides atomicity through snapshots.
