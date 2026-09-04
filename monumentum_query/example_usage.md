# Monumentum Query: Example Usage Guide

This document provides practical examples for using the `monumentum_query` crate. It covers coordinate parsing, cell references, ranges, formula tokenization, parsing, evaluation, and custom functions. Each example is adapted from the crate’s test suite and is ready to run in your own project.

---

## 1. Setting Up

Add the dependency to `Cargo.toml`:

```toml
[dependencies]
monumentum_query = "0.1.0"
monumentum_db = "0.1.0"   # for Value type
```

Import the necessary items:

```rust
use monumentum_query::coordinates::{CellRef, CellRange, CoordinateError,
    col_index_to_letter, col_letter_to_index, parse_cell_ref, parse_range};
use monumentum_query::formula::{
    FormulaContext, FormulaError, FunctionRegistry, evaluate, parse, tokenize,
};
use monumentum_db::core::value::Value;
```

---

## 2. Coordinates Module

The `coordinates` module provides types and functions for working with cell references and ranges.

### 2.1 Creating and Displaying `CellRef`

```rust
let cell = CellRef::new(0, 0);           // A1
println!("{}", cell);                    // prints "A1"

let cell_with_sheet = CellRef::new(2, 5).with_sheet("Data");
println!("{}", cell_with_sheet);         // prints "Data!C6"

let mut abs_cell = CellRef::new(1, 1);
abs_cell.abs_col = true;
abs_cell.abs_row = true;
println!("{}", abs_cell);                // prints "$B$2"
```

### 2.2 Cell Validation

```rust
let valid = CellRef::new(0, 0);
assert!(valid.is_valid());

let invalid_col = CellRef::new(16384, 0); // max columns = 16384 (0..16383)
assert!(!invalid_col.is_valid());

let invalid_row = CellRef::new(0, 1048576); // max rows = 1048576 (0..1048575)
assert!(!invalid_row.is_valid());
```

### 2.3 Column Index Conversion

```rust
// Letter to index
assert_eq!(col_letter_to_index("A").unwrap(), 0);
assert_eq!(col_letter_to_index("Z").unwrap(), 25);
assert_eq!(col_letter_to_index("AA").unwrap(), 26);
assert_eq!(col_letter_to_index("XFD").unwrap(), 16383);
assert!(col_letter_to_index("XFE").is_err()); // beyond max

// Index to letter
assert_eq!(col_index_to_letter(0), "A");
assert_eq!(col_index_to_letter(25), "Z");
assert_eq!(col_index_to_letter(26), "AA");
assert_eq!(col_index_to_letter(16383), "XFD");
assert_eq!(col_index_to_letter(16384), "#REF!");
```

### 2.4 Parsing Cell References

```rust
// Basic
let cell = parse_cell_ref("A1").unwrap();
assert_eq!(cell.col, 0);
assert_eq!(cell.row, 0);

// Absolute
let abs = parse_cell_ref("$B$2").unwrap();
assert!(abs.abs_col);
assert!(abs.abs_row);

// With sheet
let sheet_ref = parse_cell_ref("Sheet2!C3").unwrap();
assert_eq!(sheet_ref.sheet.as_deref(), Some("Sheet2"));
```

### 2.5 Parsing and Using Ranges

```rust
// Parse a range
let range = parse_range("A1:C3").unwrap();
assert_eq!(range.start.to_string(), "A1");
assert_eq!(range.end.to_string(), "C3");

// Iterate over cells in row-major order
let cells: Vec<String> = range.iter().map(|c| c.to_string()).collect();
assert_eq!(cells, vec!["A1", "B1", "C1", "A2", "B2", "C2", "A3", "B3", "C3"]);

// Check containment
let inside = CellRef::new(1, 1); // B2
assert!(range.contains(&inside));
let outside = CellRef::new(3, 3); // D4
assert!(!range.contains(&outside));
```

### 2.6 Building Ranges Manually

```rust
let start = CellRef::new(0, 0);
let end = CellRef::new(2, 2);
let range = CellRange::try_new(start, end).unwrap();
// If endpoints are reversed, try_new normalizes them
```

---

## 3. Formula Module

The `formula` module provides lexing (`tokenize`), parsing (`parse`), evaluation (`evaluate`), and a function registry.

### 3.1 Tokenizing a Formula String

```rust
let tokens = tokenize("1 + 2 * 3").unwrap();
// tokens: [Integer(1), Plus, Integer(2), Star, Integer(3)]
```

Available token types are defined in `Token` enum; e.g., `Token::Integer`, `Token::Float`, `Token::String`, `Token::Identifier`, `Token::Plus`, `Token::LParen`, etc.

### 3.2 Parsing Tokens into an AST

```rust
let tokens = tokenize("(1 + 2) * 3").unwrap();
let expr = parse(&tokens).unwrap();
// expr is an Expr::BinaryOp(Mul, BinaryOp(Add, Literal(1), Literal(2)), Literal(3))
```

The AST is defined by `Expr`:

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

### 3.3 Evaluating Expressions

To evaluate, you need:

- a `FormulaContext` implementation that resolves cell references,
- a `FunctionRegistry` containing any functions you wish to use.

```rust
struct SimpleContext;
impl FormulaContext for SimpleContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        // For simplicity, return 0 for any cell
        Ok(Value::from(0_i64))
    }
}

let ctx = SimpleContext;
let registry = FunctionRegistry::new(); // no functions registered

let tokens = tokenize("2 + 3 * 4").unwrap();
let expr = parse(&tokens).unwrap();
let result = evaluate(&expr, &ctx, &registry).unwrap();
assert_eq!(result, Value::from(14_i64));
```

### 3.4 Resolving Cell References with a Custom Context

You can create a context that stores values in a `HashMap`.

```rust
use std::collections::HashMap;

struct MapContext {
    cells: HashMap<String, Value>,
}

impl MapContext {
    fn new() -> Self { Self { cells: HashMap::new() } }
    fn set(&mut self, cell: &str, value: Value) {
        self.cells.insert(cell.to_string(), value);
    }
}

impl FormulaContext for MapContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        let key = cell.to_string();
        self.cells.get(&key).cloned()
            .ok_or_else(|| FormulaError::InvalidReference(format!("cell {key} not found")))
    }
}

// Usage
let mut ctx = MapContext::new();
ctx.set("A1", Value::from(10_i64));
let registry = FunctionRegistry::new();
let tokens = tokenize("A1 + 5").unwrap();
let expr = parse(&tokens).unwrap();
let result = evaluate(&expr, &ctx, &registry).unwrap();
assert_eq!(result, Value::from(15_i64));
```

### 3.5 Using Functions

The `FunctionRegistry` stores functions by name (case‑insensitive). Functions are plain Rust functions with signature `fn(&[Value]) -> Result<Value, FormulaError>`.

#### Registering and Calling a Custom Function

```rust
fn add_one(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity("ADDONE expects 1 argument".into()));
    }
    match &args[0] {
        Value::Integer(i) => Ok(Value::from(i.as_i64() + 1)),
        _ => Err(FormulaError::TypeMismatch("ADDONE expects an integer".into())),
    }
}

let mut registry = FunctionRegistry::new();
registry.register("ADDONE", add_one);

let ctx = MapContext::new();
let tokens = tokenize("ADDONE(41)").unwrap();
let expr = parse(&tokens).unwrap();
let result = evaluate(&expr, &ctx, &registry).unwrap();
assert_eq!(result, Value::from(42_i64));
```

#### Using Built‑in Functions

The crate itself does not include built‑in functions; you must register them. However, `monumentum_functions` (a separate crate) provides common ones. The `monumentum_workbook` crate automatically registers them.

---

## 4. Error Handling

Both coordinate parsing and formula processing return detailed error types.

### 4.1 Coordinate Errors

```rust
let result = parse_cell_ref("XFE1"); // column out of bounds
assert!(result.is_err());
match result {
    Err(CoordinateError::InvalidColumn) => println!("Invalid column"),
    Err(CoordinateError::InvalidRow) => println!("Invalid row"),
    Err(CoordinateError::InvalidReference(_)) => println!("Invalid reference"),
    Err(CoordinateError::InvalidRange(_)) => println!("Invalid range"),
    Ok(_) => unreachable!(),
}
```

### 4.2 Formula Errors

```rust
let result = tokenize("1 +");
assert!(result.is_err());
if let Err(e) = result {
    match e {
        FormulaError::Parse(msg) => println!("Parse error: {}", msg),
        FormulaError::Eval(msg) => println!("Eval error: {}", msg),
        FormulaError::DivisionByZero => println!("Division by zero"),
        FormulaError::UnknownFunction(name) => println!("Unknown function: {}", name),
        _ => println!("Other error: {}", e),
    }
}
```

---

## 5. Complete Example: A Mini Formula Engine

This example ties together all the pieces: a context that holds cell values, a registry with a couple of functions, and a function to evaluate a formula string.

```rust
use std::collections::HashMap;
use monumentum_query::coordinates::CellRef;
use monumentum_query::formula::{
    FormulaContext, FormulaError, FunctionRegistry, evaluate, parse, tokenize,
};
use monumentum_db::core::value::Value;

struct Sheet {
    cells: HashMap<String, Value>,
}

impl Sheet {
    fn new() -> Self { Self { cells: HashMap::new() } }
    fn set_cell(&mut self, ref_str: &str, value: Value) {
        self.cells.insert(ref_str.to_string(), value);
    }
}

impl FormulaContext for Sheet {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        let key = cell.to_string();
        self.cells.get(&key).cloned()
            .ok_or_else(|| FormulaError::InvalidReference(format!("Cell {} not found", key)))
    }
}

fn sum(args: &[Value]) -> Result<Value, FormulaError> {
    let mut total = 0_i64;
    for arg in args {
        if let Value::Integer(i) = arg {
            total += i.as_i64();
        } else {
            return Err(FormulaError::TypeMismatch("SUM only supports integers".into()));
        }
    }
    Ok(Value::from(total))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup sheet
    let mut sheet = Sheet::new();
    sheet.set_cell("A1", Value::from(10_i64));
    sheet.set_cell("A2", Value::from(20_i64));

    // Setup function registry
    let mut registry = FunctionRegistry::new();
    registry.register("SUM", sum);

    // Evaluate a formula
    let formula = "SUM(A1:A2) + 5";
    let tokens = tokenize(formula)?;
    let expr = parse(&tokens)?;
    let result = evaluate(&expr, &sheet, &registry)?;

    println!("Result: {:?}", result); // Integer(35)
    Ok(())
}
```

---

## 6. Summary

This guide covered:

- **Coordinates**: `CellRef`, `CellRange`, parsing, conversion.
- **Formula**: tokenizing, parsing to AST, evaluating with a custom context and function registry.
- **Errors**: how to handle coordinate and formula errors.
