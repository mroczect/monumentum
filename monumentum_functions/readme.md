# Monumentum Functions

**Preset spreadsheet functions for the Monumentum formula engine.**

`monumentum_functions` provides a collection of commonly used spreadsheet functions that can be registered into `monumentum_query::formula::FunctionRegistry`. These functions include mathematical, logical, and text operations and are designed to work directly with `monumentum_db::core::value::Value`.

Part of the [Monumentum](https://github.com/mroczect/monumentum) workspace.

---

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Available Functions](#available-functions)
- [Usage](#usage)
- [Function Reference](#function-reference)
  - [SUM](#sum)
  - [AVERAGE / AVG](#average--avg)
  - [MIN](#min)
  - [MAX](#max)
  - [IF](#if)
  - [AND](#and)
  - [OR](#or)
  - [NOT](#not)
  - [CONCAT / CONCATENATE](#concat--concatenate)
  - [TRIM](#trim)
  - [UPPER](#upper)
  - [LOWER](#lower)
  - [LEN](#len)
- [Testing](#testing)
- [Security](#security)
- [License](#license)

---

## Overview

This crate does not contain any engine logic. Instead, it implements several functions that follow the signature expected by `FunctionRegistry::register`:

```rust
fn(&[Value]) -> Result<Value, FormulaError>
```

Call `register_all(&mut registry)` to add all functions to a given registry. You can also use individual functions by importing the corresponding module if you prefer a more selective registration (though they are currently `pub(super)`; only `register_all` is public).

---

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
monumentum_functions = { git = "https://github.com/mroczect/monumentum" }
```

This will also bring in `monumentum_db` and `monumentum_query` as dependencies.

---

## Available Functions

| Function  | Description                                            | Arity      |
| --------- | ------------------------------------------------------ | ---------- |
| `SUM`     | Sum all numeric arguments                              | at least 1 |
| `AVERAGE` | Average of numeric arguments (alias `AVG`)             | at least 1 |
| `MIN`     | Smallest numeric argument                              | at least 1 |
| `MAX`     | Largest numeric argument                               | at least 1 |
| `IF`      | Conditional selection                                  | 3          |
| `AND`     | Logical AND of boolean arguments                       | at least 1 |
| `OR`      | Logical OR of boolean arguments                        | at least 1 |
| `NOT`     | Logical negation of a boolean                          | 1          |
| `CONCAT`  | Concatenate text representations (alias `CONCATENATE`) | any number |
| `TRIM`    | Remove leading/trailing whitespace                     | 1          |
| `UPPER`   | Convert text to uppercase                              | 1          |
| `LOWER`   | Convert text to lowercase                              | 1          |
| `LEN`     | Length of text (character count)                       | 1          |

All function names are case‑insensitive (the registry stores them uppercase).

---

## Usage

```rust
use monumentum_functions::register_all;
use monumentum_query::formula::FunctionRegistry;

fn main() {
    let mut registry = FunctionRegistry::new();
    register_all(&mut registry);

    assert!(registry.contains("SUM"));
    assert!(registry.contains("AVG"));
}
```

If you already use `monumentum_workbook`, these functions are registered automatically by the workbook’s default registry.

---

## Function Reference

### SUM

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Adds all numeric values. If all arguments are integers and no overflow occurs, returns `Integer`; otherwise returns `Float` if at least one `Float` is present. Non‑numeric values cause `TypeMismatch`. Empty argument list returns `WrongArity`.

### AVERAGE / AVG

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Computes the arithmetic mean of numeric arguments. Always returns `Float`. Empty argument list returns `WrongArity`; non‑numeric values cause `TypeMismatch`.

### MIN

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Returns the smallest numeric argument. If all arguments are integers, returns `Integer`; otherwise returns `Float`. Mixed non‑numeric values cause `TypeMismatch`. Empty argument list returns `WrongArity`.

### MAX

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Returns the largest numeric argument. If all arguments are integers, returns `Integer`; otherwise returns `Float`. Non‑numeric values cause `TypeMismatch`. Empty argument list returns `WrongArity`.

### IF

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Expects exactly 3 arguments: condition (`Boolean`), value if true, value if false. Returns the corresponding branch. Non‑boolean condition returns `TypeMismatch`. Wrong number of arguments returns `WrongArity`.

### AND

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Returns `Boolean(true)` if all arguments are `true`; returns `Boolean(false)` if any argument is `false`. Non‑boolean arguments cause `TypeMismatch`. Empty argument list returns `WrongArity`.

### OR

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Returns `Boolean(true)` if any argument is `true`; returns `Boolean(false)` otherwise. Non‑boolean arguments cause `TypeMismatch`. Empty argument list returns `WrongArity`.

### NOT

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Expects exactly 1 argument of type `Boolean`; returns its logical negation. Wrong number of arguments or non‑boolean input returns an error.

### CONCAT / CONCATENATE

- **Signature:** `evaluate(args: &[Value]) -> Value`
- **Description:** Concatenates the text representation of all arguments. Always returns `Text`. The function does not return an error; invalid types are represented as empty string or lossy conversion (e.g., `Blob` uses `String::from_utf8_lossy`).

### TRIM

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Removes leading and trailing whitespace from a `Text` value. Wrong number of arguments or non‑text input returns an error.

### UPPER

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Converts a `Text` value to uppercase. Wrong number of arguments or non‑text input returns an error.

### LOWER

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Converts a `Text` value to lowercase. Wrong number of arguments or non‑text input returns an error.

### LEN

- **Signature:** `evaluate(args: &[Value]) -> Result<Value, FormulaError>`
- **Description:** Returns the number of characters in a `Text` value as an `Integer`. Wrong number of arguments or non‑text input returns an error.

---

## Testing

The crate includes integration tests covering all functions. Run the full suite:

```bash
cargo test --workspace --all-targets --all-features
```

---

## Security

- **No unsafe code** – all operations are memory safe.
- **Explicit errors** – functions return `Result`; no panics on invalid input.
- **Input validation** – argument types and arity are checked before processing.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
