# Monumentum

**A modular, safe, and embeddable spreadsheet & database engine written in pure Rust.**

[![CI](https://github.com/mroczect/monumentum/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/mroczect/monumentum/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.96%2B-blue.svg)](https://blog.rust-lang.org/2024/06/13/Rust-1.96.0.html)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/mroczect/monumentum/graphs/commit-activity)
[![GitHub stars](https://img.shields.io/github/stars/mroczect/monumentum.svg?style=social&label=Star)](https://github.com/mroczect/monumentum/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/mroczect/monumentum.svg?style=social&label=Fork)](https://github.com/mroczect/monumentum/network/members)
[![GitHub issues](https://img.shields.io/github/issues/mroczect/monumentum.svg)](https://github.com/mroczect/monumentum/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/mroczect/monumentum.svg)](https://github.com/mroczect/monumentum/pulls)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

---

## Overview

**Monumentum** is a Rust workspace that combines a lightweight embedded database, a spreadsheet formula engine, high‑level workbook operations, and a collection of preset spreadsheet functions. It is designed for building applications that need safe, offline, embedded data storage with spreadsheet‑like functionality — such as desktop editors, backend services, calculators, or data analysis tools.

The project is split into focused crates so that you can use only the pieces you need.

---

## Workspace Crates

| Crate                                           | Description                                                                               | Type    |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------- | ------- |
| [`monumentum_db`](monumentum_db/)               | Core storage engine: typed tables, constraints, WAL, file persistence                     | library |
| [`monumentum_query`](monumentum_query/)         | Formula engine: lexer, parser, evaluator, cell/range references, custom function registry | library |
| [`monumentum_workbook`](monumentum_workbook/)   | High‑level spreadsheet API: sheets, cells, formulas, sorting, filtering, transactions     | library |
| [`monumentum_functions`](monumentum_functions/) | Preset spreadsheet functions (`SUM`, `IF`, `TRIM`, etc.) for the formula engine           | library |

All crates share **Rust edition 2024** and are tested with **Rust 1.96.0**.

---

## Features

- **Embedded database**: schema-validated tables, rows, columns, primary/unique keys, check constraints, default values, and allowed values.
- **Durable storage**: snapshot + write‑ahead log (WAL) with CRC32 checksums, atomic snapshots, and file locking.
- **Formula engine**: arithmetic, logic, comparison, cell references (`A1`, `$B$2`, `Sheet2!D5`), ranges (`A1:B10`), and custom functions.
- **Workbook layer**: create/rename/drop sheets, edit cells, sort/filter data, protect sheets, evaluate formulas, and manage transactions.
- **Preset functions**: `SUM`, `AVERAGE`, `MIN`, `MAX`, `IF`, `AND`, `OR`, `NOT`, `CONCAT`, `TRIM`, `UPPER`, `LOWER`, `LEN`.
- **Safety**: no `unsafe` code, strict input validation, resource limits, no SQL injection surface, and explicit error handling.
- **Portable**: standard library only for the core crates; platform‑specific features degrade gracefully.

---

## Quick Start

### Add the workspace crates

Add the pieces you need to your `Cargo.toml`:

```toml
[dependencies]
monumentum_db = { git = "https://github.com/mroczect/monumentum" }
monumentum_query = { git = "https://github.com/mroczect/monumentum" }
monumentum_workbook = { git = "https://github.com/mroczect/monumentum" }
monumentum_functions = { git = "https://github.com/mroczect/monumentum" }
```

### Example: In‑memory workbook with formula

```rust
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::InMemoryStorage;
use monumentum_workbook::Workbook;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();

    wb.create_sheet(
        "Data",
        vec![
            ColumnDef::new("Name", DataType::Text),
            ColumnDef::new("Score", DataType::Integer),
        ],
    )?;

    wb.insert_row("Data", vec![Value::from("Alice"), Value::from(90_i64)])?;
    wb.insert_row("Data", vec![Value::from("Bob"), Value::from(80_i64)])?;

    // Add formula row
    wb.insert_row("Data", vec![Value::from("Average"), Value::Null])?;
    let row_idx = wb.row_count("Data")? - 1;
    wb.set_formula("Data", row_idx, 1, "AVERAGE(B1:B2)")?;

    let avg = wb.get_cell_value("Data", row_idx, 1)?;
    println!("Average: {:?}", avg); // Float(85.0)

    Ok(())
}
```

### Example: File‑backed workbook

```rust
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::FileStorage;
use monumentum_workbook::Workbook;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("example.monumentum");

    {
        let mut wb = Workbook::<FileStorage>::create_new(path)?;
        wb.create_sheet(
            "Users",
            vec![
                ColumnDef::new("ID", DataType::Integer),
                ColumnDef::new("Name", DataType::Text),
            ],
        )?;
        wb.insert_row("Users", vec![Value::from(1_i64), Value::from("Alice")])?;
        wb.save()?;
    }

    // Reopen
    let wb = Workbook::<FileStorage>::open(path)?;
    println!("Sheets: {:?}", wb.sheet_names());
    Ok(())
}
```

---

## Architecture

The workspace follows a layered design:

```
┌─────────────────────────────┐
│   monumentum_workbook       │  High‑level spreadsheet API
├─────────────────────────────┤
│   monumentum_functions      │  Preset formula functions
├─────────────────────────────┤
│   monumentum_query          │  Formula lexer/parser/evaluator
├─────────────────────────────┤
│   monumentum_db             │  Core storage & data model
└─────────────────────────────┘
```

Dependency flow is one‑way:

- `monumentum_workbook` → `monumentum_query`, `monumentum_functions`, `monumentum_db`
- `monumentum_functions` → `monumentum_query`, `monumentum_db`
- `monumentum_query` → `monumentum_db`

---

## Security

- **Memory safety**: no `unsafe` code anywhere in the workspace.
- **Data integrity**: CRC32 WAL checksums, atomic file writes, and strict deserialization limits.
- **Resource limits**: maximum formula length (64 KiB), parser depth (128), range cells (100k), row count (10M), and snapshot size (256 MiB).
- **File safety**: `O_NOFOLLOW`, mode `0600`, and atomic rename prevent symlink attacks and partial writes.
- **Input validation**: identifiers and values are validated; no raw string queries or injection surfaces.

---

## Testing

Run the entire workspace test suite:

```bash
cargo test --workspace --all-targets --all-features
```

Run Clippy with warnings denied:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Check formatting:

```bash
cargo fmt --all -- --check
```

---

## Roadmap

- [ ] SQL‑like query engine (`monumentum_sql`)
- [ ] Encryption at rest (age‑auth integration)
- [ ] Versioned workbooks (libvctrl integration)
- [ ] Asynchronous storage APIs
- [ ] WebAssembly bindings

---

## Contributing

Contributions are welcome! Please:

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run `cargo test --workspace --all-targets --all-features`.
4. Ensure all public items are documented.
5. Follow Rust idiomatic style.
6. Open an issue or PR.

---

## License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
