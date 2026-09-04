# Monumentum Query API Documentation

This document provides a complete reference for the `monumentum_query` crate. It describes all public types, functions, traits, and methods exposed by the crate, organized by module.

---

## Table of Contents

1. [Module `coordinates`](#module-coordinates)
   - [`CellRef`](#cellref)
   - [`CellRange`](#cellrange)
   - [`CellRangeIter`](#cellrangeiter)
   - [`CoordinateError`](#coordinateerror)
   - [Functions](#coordinates-functions)
     - [`col_index_to_letter`](#col_index_to_letter)
     - [`col_letter_to_index`](#col_letter_to_index)
     - [`parse_cell_ref`](#parse_cell_ref)
     - [`parse_range`](#parse_range)
2. [Module `formula`](#module-formula)
   - [`Expr`](#expr)
   - [`BinaryOp`](#binaryop)
   - [`UnaryOp`](#unaryop)
   - [`FormulaContext`](#formulacontext)
   - [`FormulaError`](#formulaerror)
   - [Functions](#formula-functions)
     - [`evaluate`](#evaluate)
     - [`tokenize`](#tokenize)
     - [`parse`](#parse)
   - [`FunctionRegistry`](#functionregistry)
   - [`FunctionImpl`](#functionimpl)
   - [`Token`](#token)

---

## Module `coordinates`

This module contains types and functions for working with spreadsheet cell references and ranges.

### `CellRef`

Represents a single cell reference, optionally with a sheet name and absolute flags.

```rust
pub struct CellRef {
    pub col: u32,
    pub row: u32,
    pub abs_col: bool,
    pub abs_row: bool,
    pub sheet: Option<String>,
}
```

**Fields**

| Field     | Type             | Description                                             |
| --------- | ---------------- | ------------------------------------------------------- |
| `col`     | `u32`            | Zero‑based column index.                                |
| `row`     | `u32`            | Zero‑based row index.                                   |
| `abs_col` | `bool`           | Whether the column reference is absolute (e.g., `$A1`). |
| `abs_row` | `bool`           | Whether the row reference is absolute (e.g., `A$1`).    |
| `sheet`   | `Option<String>` | Optional sheet name (e.g., `Sheet1!A1`).                |

**Methods**

| Method       | Signature                                                       | Description                                                                                       |
| ------------ | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `new`        | `pub fn new(col: u32, row: u32) -> Self`                        | Creates a new `CellRef` with no sheet and both absolute flags set to `false`.                     |
| `with_sheet` | `pub fn with_sheet(mut self, sheet: impl Into<String>) -> Self` | Sets the sheet name and returns the modified `CellRef`.                                           |
| `is_valid`   | `pub fn is_valid(&self) -> bool`                                | Returns `true` if the column index is less than `16384` and the row index is less than `1048576`. |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`
- `Display` – Formats the reference, e.g., `Sheet1!$A$1`.

---

### `CellRange`

Represents a rectangular range of cells, defined by two `CellRef` endpoints.

```rust
pub struct CellRange {
    pub start: CellRef,
    pub end: CellRef,
}
```

**Fields**

| Field   | Type      | Description                         |
| ------- | --------- | ----------------------------------- |
| `start` | `CellRef` | The top‑left cell of the range.     |
| `end`   | `CellRef` | The bottom‑right cell of the range. |

**Methods**

| Method          | Signature                                                                       | Description                                                                                           |
| --------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `try_new`       | `pub fn try_new(start: CellRef, end: CellRef) -> Result<Self, CoordinateError>` | Creates a new `CellRange` if both cells are valid and have the same sheet. Sorts endpoints if needed. |
| `new_unchecked` | `pub fn new_unchecked(start: CellRef, end: CellRef) -> Self`                    | Creates a `CellRange` without validation; uses `debug_assert` that cells are valid.                   |
| `iter`          | `pub fn iter(&self) -> CellRangeIter<'_>`                                       | Returns an iterator over all cells in row‑major order.                                                |
| `contains`      | `pub fn contains(&self, cell: &CellRef) -> bool`                                | Checks whether the given cell lies within the range and has the same sheet.                           |
| `is_valid`      | `pub fn is_valid(&self) -> bool`                                                | Returns `true` if start and end are valid and the range is well‑ordered.                              |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`

---

### `CellRangeIter`

An iterator that yields `CellRef` objects for each cell in a `CellRange`.

```rust
pub struct CellRangeIter<'a> { /* private fields */ }
```

**Trait Implementations**

- `Iterator<Item = CellRef>` – Yields cells in row‑major order (left to right, top to bottom).
- `Debug`

---

### `CoordinateError`

Error type for coordinate parsing and validation.

```rust
pub enum CoordinateError {
    InvalidColumn,
    InvalidRow,
    InvalidReference(String),
    InvalidRange(String),
}
```

**Variants**

| Variant            | Description                                  |
| ------------------ | -------------------------------------------- |
| `InvalidColumn`    | Column letters are invalid or out of bounds. |
| `InvalidRow`       | Row number is zero or exceeds maximum.       |
| `InvalidReference` | The entire reference string is invalid.      |
| `InvalidRange`     | The range string is invalid.                 |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`
- `Display` – Human‑readable error messages.
- `std::error::Error`

---

### Coordinates Functions

#### `col_index_to_letter`

```rust
pub fn col_index_to_letter(index: u32) -> String
```

Converts a zero‑based column index to an Excel‑style column letter sequence (e.g., `0` → `"A"`, `25` → `"Z"`, `26` → `"AA"`). Returns `"#REF!"` if `index >= 16384`.

#### `col_letter_to_index`

```rust
pub fn col_letter_to_index(letters: &str) -> Result<u32, CoordinateError>
```

Converts a column letter string to a zero‑based column index. Returns `CoordinateError::InvalidColumn` if the input is empty, contains non‑uppercase ASCII letters, or the result exceeds the maximum.

#### `parse_cell_ref`

```rust
pub fn parse_cell_ref(input: &str) -> Result<CellRef, CoordinateError>
```

Parses a string like `"A1"`, `"$B$2"`, or `"Sheet1!C3"` into a `CellRef`. The input is trimmed and must not exceed 1024 characters. Returns a `CoordinateError` variant on failure.

#### `parse_range`

```rust
pub fn parse_range(input: &str) -> Result<CellRange, CoordinateError>
```

Parses a string like `"A1:B2"` or `"Sheet1!A1:C3"` into a `CellRange`. A single cell reference is treated as a range containing only that cell. Returns a `CoordinateError` on failure.

---

## Module `formula`

This module provides the formula parser, evaluator, and associated types.

### `Expr`

The abstract syntax tree (AST) node for formulas.

```rust
pub enum Expr {
    Literal(Value),
    CellRef(String),
    Range(CellRange),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}
```

**Variants**

| Variant        | Description                                                     |
| -------------- | --------------------------------------------------------------- |
| `Literal`      | A constant value (number, text, boolean, null).                 |
| `CellRef`      | A reference to a single cell (stored as a string).              |
| `Range`        | A range of cells (`CellRange`).                                 |
| `UnaryOp`      | A unary operation applied to a sub‑expression.                  |
| `BinaryOp`     | A binary operation between two sub‑expressions.                 |
| `FunctionCall` | A function call with a name and a list of argument expressions. |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`

---

### `BinaryOp`

Enumerates the binary operators supported in formulas.

```rust
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, NotEq, Lt, Lte, Gt, Gte,
    And, Or,
}
```

**Variants**

| Variant | Description                  |
| ------- | ---------------------------- |
| `Add`   | Addition (`+`)               |
| `Sub`   | Subtraction (`-`)            |
| `Mul`   | Multiplication (`*`)         |
| `Div`   | Division (`/`)               |
| `Mod`   | Modulo (`%`)                 |
| `Pow`   | Power (`^`)                  |
| `Eq`    | Equality (`==`)              |
| `NotEq` | Inequality (`!=`)            |
| `Lt`    | Less than (`<`)              |
| `Lte`   | Less than or equal (`<=`)    |
| `Gt`    | Greater than (`>`)           |
| `Gte`   | Greater than or equal (`>=`) |
| `And`   | Logical AND (`&&`)           |
| `Or`    | Logical OR (`                |     | `)  |

**Trait Implementations**

- `Debug`
- `Clone`
- `Copy`
- `PartialEq`
- `Eq`

---

### `UnaryOp`

Enumerates the unary operators.

```rust
pub enum UnaryOp {
    Neg,
    Not,
}
```

**Variants**

| Variant | Description          |
| ------- | -------------------- |
| `Neg`   | Unary negation (`-`) |
| `Not`   | Logical NOT (`!`)    |

**Trait Implementations**

- `Debug`
- `Clone`
- `Copy`
- `PartialEq`
- `Eq`

---

### `FormulaContext`

Trait for resolving cell values during formula evaluation. Users implementing a workbook or data source must provide this.

```rust
pub trait FormulaContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError>;
}
```

**Required Method**

| Method           | Signature                                                                 | Description                          |
| ---------------- | ------------------------------------------------------------------------- | ------------------------------------ |
| `get_cell_value` | `fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError>` | Returns the value of the given cell. |

---

### `FormulaError`

Error type for formula parsing and evaluation.

```rust
pub enum FormulaError {
    Parse(String),
    Eval(String),
    CircularReference(String),
    InvalidReference(String),
    DivisionByZero,
    TypeMismatch(String),
    UnknownFunction(String),
    WrongArity(String),
    Unsupported(String),
}
```

**Variants**

| Variant             | Description                                              |
| ------------------- | -------------------------------------------------------- |
| `Parse`             | Error during tokenization or parsing (contains message). |
| `Eval`              | Error during evaluation (e.g., overflow).                |
| `CircularReference` | Formula refers back to itself.                           |
| `InvalidReference`  | Cell reference is invalid or out of bounds.              |
| `DivisionByZero`    | Division or modulo by zero.                              |
| `TypeMismatch`      | Operands have incompatible types.                        |
| `UnknownFunction`   | Function name not found in registry.                     |
| `WrongArity`        | Incorrect number of arguments to a function.             |
| `Unsupported`       | Operation not supported.                                 |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`
- `Display` – Human‑readable error messages.
- `std::error::Error`
- `MonumentumError` (from `monumentum_db`)

---

### Formula Functions

#### `evaluate`

```rust
pub fn evaluate(
    expr: &Expr,
    ctx: &dyn FormulaContext,
    registry: &FunctionRegistry,
) -> Result<Value, FormulaError>
```

Evaluates an `Expr` AST to a `Value`. It resolves cell references via the provided `FormulaContext` and function calls via the `FunctionRegistry`. Ranges are only allowed as arguments to functions; using a range in a scalar context returns an error.

#### `tokenize`

```rust
pub fn tokenize(input: &str) -> Result<Vec<Token>, FormulaError>
```

Converts a formula string into a vector of `Token`s. The input must not exceed 64 KiB. Returns `FormulaError::Parse` on invalid characters or malformed tokens.

#### `parse`

```rust
pub fn parse(tokens: &[Token]) -> Result<Expr, FormulaError>
```

Parses a sequence of tokens into an `Expr` AST. Supports operator precedence and parentheses. Returns `FormulaError::Parse` on syntax errors or excessive nesting.

---

### `FunctionRegistry`

A collection of named functions available for formula evaluation.

```rust
pub struct FunctionRegistry { /* private fields */ }
```

**Methods**

| Method     | Signature                                                                       | Description                                                                      |
| ---------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `new`      | `pub fn new() -> Self`                                                          | Creates an empty registry.                                                       |
| `register` | `pub fn register(&mut self, name: &str, func: FunctionImpl)`                    | Inserts or overwrites a function; names are case‑insensitive (stored uppercase). |
| `remove`   | `pub fn remove(&mut self, name: &str) -> Option<FunctionImpl>`                  | Removes a function; returns the previous function if present.                    |
| `contains` | `pub fn contains(&self, name: &str) -> bool`                                    | Checks whether a function is registered (case‑insensitive).                      |
| `call`     | `pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, FormulaError>` | Invokes a function by name, returning `UnknownFunction` if not found.            |

**Trait Implementations**

- `Debug`
- `Clone`
- `Default`

---

### `FunctionImpl`

Type alias for a function pointer.

```rust
pub type FunctionImpl = fn(&[Value]) -> Result<Value, FormulaError>;
```

Functions of this type can be registered in a `FunctionRegistry`.

---

### `Token`

Tokens produced by the formula lexer.

```rust
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
    Bang,
    LParen,
    RParen,
    Comma,
    Colon,
}
```

**Variants** correspond to the lexical elements of formulas:

| Variant          | Description                              |
| ---------------- | ---------------------------------------- |
| `Integer(i64)`   | Integer literal.                         |
| `Float(f64)`     | Floating‑point literal.                  |
| `String(String)` | String literal (with escapes processed). |
| `Boolean(bool)`  | `true` or `false` literal.               |
| `Null`           | `null` literal.                          |
| `Identifier`     | Cell reference or function name.         |
| `Plus`           | `+`                                      |
| `Minus`          | `-`                                      |
| `Star`           | `*`                                      |
| `Slash`          | `/`                                      |
| `Percent`        | `%`                                      |
| `Caret`          | `^`                                      |
| `EqEq`           | `==`                                     |
| `NotEq`          | `!=`                                     |
| `Lt`             | `<`                                      |
| `LtEq`           | `<=`                                     |
| `Gt`             | `>`                                      |
| `GtEq`           | `>=`                                     |
| `AndAnd`         | `&&`                                     |
| `OrOr`           | `                                        |     | `   |
| `Bang`           | `!`                                      |
| `LParen`         | `(`                                      |
| `RParen`         | `)`                                      |
| `Comma`          | `,`                                      |
| `Colon`          | `:` (range separator)                    |

**Trait Implementations**

- `Debug`
- `Clone`
- `PartialEq`
