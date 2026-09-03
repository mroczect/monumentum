# Monumentum Query

**A pure formula engine for Rust spreadsheet applications.**

`monumentum_query` is a library for parsing and evaluating spreadsheet-style formulas. It works with the `monumentum_db` data types and provides a safe, extensible engine for cell references, ranges, arithmetic, logical operations, and custom functions. The engine itself does **not** include built-in functions; instead, developers register their own functions through a `FunctionRegistry`.

Part of the [Monumentum](https://github.com/mroczect/monumentum) workspace.

---

## Table of Contents

- [Overview](#overview)
- [Design Goals](#design-goals)
- [Installation](#installation)
- [Architecture](#architecture)
- [Coordinates Module](#coordinates-module)
  - [CellRef](#cellref)
  - [CellRange](#cellrange)
  - [Parser Functions](#parser-functions)
- [Formula Module](#formula-module)
  - [AST Types](#ast-types)
  - [Lexer](#lexer)
  - [Parser](#parser)
  - [Evaluator](#evaluator)
  - [FormulaContext](#formulacontext)
  - [Function Registry](#function-registry)
- [Errors](#errors)
- [Examples](#examples)
  - [Basic Evaluation](#basic-evaluation)
  - [Custom Function](#custom-function)
  - [Using Preset Functions](#using-preset-functions)
- [Full API Reference](#full-api-reference)
  - [coordinates::cell_ref](#coordinatescell_ref)
  - [coordinates::range](#coordinatesrange)
  - [coordinates::parser](#coordinatesparser)
  - [formula::ast](#formulaast)
  - [formula::lexer](#formulalexer)
  - [formula::parser](#formulaparser)
  - [formula::evaluator](#formulaevaluator)
  - [formula::context](#formulacontext-1)
  - [formula::functions](#formulafunctions)
  - [formula::error](#formulaerror)
- [Testing](#testing)
- [Security](#security)
- [License](#license)

---

## Overview

`monumentum_query` provides the building blocks to create an Excel-like formula engine in Rust. It handles:

- Parsing cell references like `A1`, `$B$2`, `Sheet2!D5`
- Parsing ranges like `A1:B10`, including cross-sheet ranges
- Lexing and parsing arithmetic, comparison, logical, and unary operators
- Evaluating expressions against a user-supplied context
- Managing custom functions via a registry
- Enforcing resource limits (max formula length, parse depth, range size)
- Producing typed errors (`FormulaError`)

The crate does not include built-in spreadsheet functions (`SUM`, `IF`, `AVERAGE`, etc.). Instead, it offers a `FunctionRegistry` where the developer can add any function they need. A separate crate, `monumentum_functions`, provides a set of commonly used functions that can be registered.

---

## Design Goals

- **Pure engine**: no hard-coded functions; everything is through the registry.
- **Safety**: no `unsafe` code, strict input validation, resource limits to prevent DoS.
- **Stability**: all errors are explicit via `Result`; no panics on malformed input.
- **Flexibility**: supports absolute/relative references, sheet-qualified references, and custom functions.
- **Performance**: checks range size before expansion to avoid memory exhaustion.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
monumentum_query = { git = "https://github.com/mroczect/monumentum" }
monumentum_db = { git = "https://github.com/mroczect/monumentum" }   # required for Value
```

Or, if you want the preset functions:

```toml
monumentum_functions = { git = "https://github.com/mroczect/monumentum" }
```

---

## Architecture

The crate is divided into two main modules:

```
monumentum_query/
├── coordinates/       # cell and range handling
│   ├── cell_ref.rs
│   ├── range.rs
│   ├── parser.rs
│   └── mod.rs
├── formula/           # formula parsing and evaluation
│   ├── ast.rs
│   ├── context.rs
│   ├── error.rs
│   ├── evaluator.rs
│   ├── functions/
│   │   ├── registry.rs
│   │   └── mod.rs
│   ├── lexer.rs
│   ├── parser.rs
│   └── mod.rs
└── lib.rs
```

Dependency flow:

```
monumentum_query  →  monumentum_db (for Value)
```

---

## Coordinates Module

### `CellRef`

Represents a single cell reference. The struct fields are public for easy construction, but the parser validates bounds.

```rust
pub struct CellRef {
    pub col: u32,
    pub row: u32,
    pub abs_col: bool,
    pub abs_row: bool,
    pub sheet: Option<String>,
}
```

- **col**: 0‑based column index (A = 0, B = 1, …)
- **row**: 0‑based row index (row 1 = 0)
- **abs_col / abs_row**: flags for `$` absolute references
- **sheet**: optional sheet name

Methods:

```rust
pub fn new(col: u32, row: u32) -> Self
pub fn with_sheet(mut self, sheet: impl Into<String>) -> Self
pub fn is_valid(&self) -> bool   // checks column < 16384 and row < 1048576
```

`Display` returns a string like `"A1"`, `"$B$2"`, `"Sheet1!C3"`.

### `CellRange`

Represents an inclusive rectangular range. When created via `try_new` or `new_unchecked`, the start and end are normalized so that `start` is the top‑left cell and `end` is the bottom‑right.

```rust
pub struct CellRange {
    pub start: CellRef,
    pub end: CellRef,
}
```

Constructors:

```rust
pub fn try_new(start: CellRef, end: CellRef) -> Result<Self, CoordinateError>
pub fn new_unchecked(start: CellRef, end: CellRef) -> Self
```

Important methods:

```rust
pub fn iter(&self) -> CellRangeIter<'_>
pub fn contains(&self, cell: &CellRef) -> bool   // checks sheet equality
pub fn is_valid(&self) -> bool
```

`CellRangeIter` yields each `CellRef` in row‑major order.

### Parser Functions

```rust
pub fn col_letter_to_index(letters: &str) -> Result<u32, CoordinateError>
pub fn col_index_to_letter(index: u32) -> String
pub fn parse_cell_ref(input: &str) -> Result<CellRef, CoordinateError>
pub fn parse_range(input: &str) -> Result<CellRange, CoordinateError>
```

- `col_letter_to_index("A") == 0`, `"AA" == 26`, `"XFD" == 16383`
- `col_index_to_letter(0) == "A"`, `26 == "AA"`
- `parse_cell_ref` accepts forms like `A1`, `$A$1`, `Sheet1!A1`
- `parse_range` accepts `A1:B2`, `Sheet1!A1:B10`, and single cells (`A1`)

---

## Formula Module

### AST Types

```rust
pub enum Expr {
    Literal(Value),
    CellRef(String),
    Range(CellRange),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, NotEq, Lt, Lte, Gt, Gte,
    And, Or,
}

pub enum UnaryOp {
    Neg,
    Not,
}
```

`Value` is from `monumentum_db::core::value::Value`.

### Lexer

Tokenizes formula strings into `Token`s.

```rust
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Identifier(String),
    Plus, Minus, Star, Slash, Percent, Caret,
    EqEq, NotEq, Lt, LtEq, Gt, GtEq,
    AndAnd, OrOr, Bang,
    LParen, RParen, Comma, Colon,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, FormulaError>
```

Supports:

- Numbers (integer, float, scientific notation)
- Strings (with escape sequences: `\"`, `\\`, `\n`, `\t`, `\r`)
- Booleans (`true`, `TRUE`, `false`, `FALSE`)
- Null (`null`, `NULL`)
- Identifiers (cell refs, function names, sheet names) starting with letter, underscore, or `$`
- All operators and punctuation

**Limits**: input length ≤ 64 KiB; oversized input returns `FormulaError::Parse`.

### Parser

Parses tokens into an `Expr` AST.

```rust
pub fn parse(tokens: &[Token]) -> Result<Expr, FormulaError>
```

Features:

- Precedence: `||` < `&&` < `==/!=` < `<,<=,>,>=` < `+,-` < `*,/,%` < `^` (right‑assoc) < unary `-`, `!`
- Parentheses
- Function calls with zero or more arguments
- Cell references and ranges

**Limits**: maximum parse depth = 128; deeper nesting returns `FormulaError::Parse`.

### Evaluator

Evaluates an AST against a `FormulaContext` and a `FunctionRegistry`.

```rust
pub fn evaluate(
    expr: &Expr,
    ctx: &dyn FormulaContext,
    registry: &FunctionRegistry,
) -> Result<Value, FormulaError>
```

Features:

- Range expansion inside function arguments is limited to `MAX_RANGE_CELLS = 100_000` cells to prevent memory exhaustion.
- Integer arithmetic uses checked operations; overflow returns error.
- Float operations reject non‑finite results.
- Division/modulo by zero returns `FormulaError::DivisionByZero`.
- Text concatenation with `+` is supported for two `Text` values.

### FormulaContext

Trait that the evaluator uses to fetch cell values during evaluation.

```rust
pub trait FormulaContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError>;
}
```

Implement this trait for your own data model (e.g., a workbook).

### Function Registry

Allows registration of custom functions by name.

```rust
pub type FunctionImpl = fn(&[Value]) -> Result<Value, FormulaError>;

#[derive(Debug, Clone, Default)]
pub struct FunctionRegistry {
    // private fields
}

impl FunctionRegistry {
    pub fn new() -> Self
    pub fn register(&mut self, name: &str, func: FunctionImpl)
    pub fn remove(&mut self, name: &str) -> Option<FunctionImpl>
    pub fn contains(&self, name: &str) -> bool
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, FormulaError>
}
```

- Function names are case‑insensitive and stored uppercase.
- `call` returns `FormulaError::UnknownFunction` if the name is not registered.

---

## Errors

### `CoordinateError`

```rust
pub enum CoordinateError {
    InvalidColumn,
    InvalidRow,
    InvalidReference(String),
    InvalidRange(String),
}
```

Implements `Display` and `Error`.

### `FormulaError`

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

Implements `Display` and `Error`.

---

## Examples

### Basic Evaluation

```rust
use monumentum_db::core::value::Value;
use monumentum_query::{
    coordinates::CellRef,
    formula::{evaluate, parse, tokenize, FormulaContext, FormulaError, FunctionRegistry},
};
use std::collections::HashMap;

struct Context {
    cells: HashMap<String, Value>,
}

impl FormulaContext for Context {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        self.cells
            .get(&cell.to_string())
            .cloned()
            .ok_or_else(|| FormulaError::InvalidReference(format!("{}", cell)))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = Context { cells: HashMap::new() };
    ctx.cells.insert("A1".to_string(), Value::from(10_i64));
    ctx.cells.insert("A2".to_string(), Value::from(20_i64));

    let registry = FunctionRegistry::new();
    let tokens = tokenize("A1 + A2")?;
    let expr = parse(&tokens)?;
    let result = evaluate(&expr, &ctx, &registry)?;

    assert_eq!(result, Value::from(30_i64));
    Ok(())
}
```

### Custom Function

```rust
use monumentum_db::core::value::Value;
use monumentum_query::formula::{FunctionRegistry, FormulaError};

fn double(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [Value::Integer(i)] => Ok(Value::from(i.as_i64() * 2)),
        _ => Err(FormulaError::WrongArity("DOUBLE expects exactly 1 integer".to_string())),
    }
}

fn main() {
    let mut registry = FunctionRegistry::new();
    registry.register("DOUBLE", double);

    let result = registry.call("double", &[Value::from(21_i64)]).unwrap();
    assert_eq!(result, Value::from(42_i64));
}
```

### Using Preset Functions

If you have `monumentum_functions` in your dependencies:

```rust
use monumentum_functions::register_all;
use monumentum_query::formula::FunctionRegistry;

fn main() {
    let mut registry = FunctionRegistry::new();
    register_all(&mut registry);

    // Now SUM, AVERAGE, etc. are available
    assert!(registry.contains("SUM"));
}
```

---

## Full API Reference

### `coordinates::cell_ref`

```rust
pub struct CellRef {
    pub col: u32,
    pub row: u32,
    pub abs_col: bool,
    pub abs_row: bool,
    pub sheet: Option<String>,
}

impl CellRef {
    pub fn new(col: u32, row: u32) -> Self;
    pub fn with_sheet(mut self, sheet: impl Into<String>) -> Self;
    pub fn is_valid(&self) -> bool;
}

impl fmt::Display for CellRef;
```

### `coordinates::range`

```rust
pub struct CellRange {
    pub start: CellRef,
    pub end: CellRef,
}

impl CellRange {
    pub fn try_new(start: CellRef, end: CellRef) -> Result<Self, CoordinateError>;
    pub fn new_unchecked(start: CellRef, end: CellRef) -> Self;
    pub fn iter(&self) -> CellRangeIter<'_>;
    pub fn contains(&self, cell: &CellRef) -> bool;
    pub fn is_valid(&self) -> bool;
}

pub struct CellRangeIter<'a> { /* private */ }
impl<'a> Iterator for CellRangeIter<'a> {
    type Item = CellRef;
    fn next(&mut self) -> Option<Self::Item>;
}
```

### `coordinates::parser`

```rust
pub fn col_letter_to_index(letters: &str) -> Result<u32, CoordinateError>;
pub fn col_index_to_letter(index: u32) -> String;
pub fn parse_cell_ref(input: &str) -> Result<CellRef, CoordinateError>;
pub fn parse_range(input: &str) -> Result<CellRange, CoordinateError>;
```

### `formula::ast`

```rust
pub enum Expr {
    Literal(Value),
    CellRef(String),
    Range(CellRange),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, NotEq, Lt, Lte, Gt, Gte,
    And, Or,
}

pub enum UnaryOp {
    Neg,
    Not,
}
```

### `formula::lexer`

```rust
pub enum Token { ... }  // as listed earlier

pub fn tokenize(input: &str) -> Result<Vec<Token>, FormulaError>;
```

### `formula::parser`

```rust
pub fn parse(tokens: &[Token]) -> Result<Expr, FormulaError>;
```

### `formula::evaluator`

```rust
pub fn evaluate(
    expr: &Expr,
    ctx: &dyn FormulaContext,
    registry: &FunctionRegistry,
) -> Result<Value, FormulaError>;
```

### `formula::context`

```rust
pub trait FormulaContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError>;
}
```

### `formula::functions`

```rust
pub type FunctionImpl = fn(&[Value]) -> Result<Value, FormulaError>;

pub struct FunctionRegistry { /* private */ }

impl FunctionRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, name: &str, func: FunctionImpl);
    pub fn remove(&mut self, name: &str) -> Option<FunctionImpl>;
    pub fn contains(&self, name: &str) -> bool;
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, FormulaError>;
}
```

### `formula::error`

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

---

## Testing

The crate includes extensive unit and integration tests. Run the workspace test suite:

```bash
cargo test --workspace --all-targets --all-features
```

Run clippy with warnings denied:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## Security

- **No `unsafe` code** – all memory safety guarantees of Rust are maintained.
- **Resource limits** – maximum formula length (64 KiB), maximum parse depth (128), maximum range cells (100,000) to prevent DoS.
- **Input validation** – all references and expressions are validated before evaluation.
- **No panic on malformed input** – errors are returned via `Result`.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
