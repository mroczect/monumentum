# Monumentum

[![CI](https://github.com/mroczect/monumentum/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/mroczect/monumentum/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.96%2B-blue.svg)](https://blog.rust-lang.org/2024/06/13/Rust-1.96.0.html)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/mroczect/monumentum/graphs/commit-activity)
[![GitHub stars](https://img.shields.io/github/stars/mroczect/monumentum.svg?style=social&label=Star)](https://github.com/mroczect/monumentum/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/mroczect/monumentum.svg?style=social&label=Fork)](https://github.com/mroczect/monumentum/network/members)
[![GitHub issues](https://img.shields.io/github/issues/mroczect/monumentum.svg)](https://github.com/mroczect/monumentum/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/mroczect/monumentum.svg)](https://github.com/mroczect/monumentum/pulls)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

**Monumentum** is a lightweight, embedded database system written in Rust. It follows a strict **contracts‑first** architecture, separating pure data types and traits from concrete implementations. The project aims to provide a foundation for building reliable, safe, and modular storage backends without external dependencies.

---

## Table of Contents

- [Overview](#overview)
- [Crates](#crates)
- [Features](#features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
- [Documentation](#documentation)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

Monumentum provides a minimalist but robust core for building database engines. It is not a full‑featured DBMS; instead it offers the building blocks:

- **`monumentum_handler`** – a _pure contracts_ crate containing types, constants, traits, validation, and error definitions. It has **zero dependencies** beyond the Rust standard library.
- **`monumentum_core`** – the reference implementation of those contracts, providing in‑memory and file‑backed storage, serialization, indexing, and write‑ahead logging.

The design enforces **invalid states unrepresentable** by using fallible constructors and immutable accessors where appropriate. All public types are validated at construction time, and the code is `#![forbid(unsafe_code)]`.

---

## Crates

| Crate                | Description                                                             | Documentation                  |
| -------------------- | ----------------------------------------------------------------------- | ------------------------------ |
| `monumentum_handler` | Contracts: types, traits, errors, validation, constants                 | `monumentum_handler/README.md` |
| `monumentum_core`    | Reference implementation: storage, WAL, serialization, table management | `monumentum_core/README.md`    |

---

## Features

- **Strict separation of concerns** – contracts (`handler`) are independent of any implementation.
- **Resource limits** – hard `MAX_*` constants prevent memory exhaustion from malicious input.
- **Rich value system** – `Value` supports null, integer, float, text, blob, and boolean.
- **Schema validation** – column types, nullability, primary keys, uniqueness, check constraints, foreign keys, allowed values.
- **Indexing** – hash‑based unique indexes for fast lookups.
- **Serialization** – binary encoding/decoding with size limits and corruption detection.
- **Write‑ahead logging** – append‑only log with CRC32 checksums for durability.
- **File storage** – atomic snapshots plus WAL replay for crash recovery.
- **In‑memory storage** – simple backend for testing or ephemeral data.
- **No unsafe code** – `#![forbid(unsafe_code)]` throughout.
- **No external runtime dependencies** – only `std` and `alloc`.

---

## Architecture

The workspace is organised as a small, layered dependency graph:

```
monumentum_handler   (contracts)
        ↑
monumentum_core      (reference implementation)
```

- `monumentum_handler` defines **what** operations and data structures exist.
- `monumentum_core` provides **how** those operations are actually performed.

This separation allows alternative implementations (e.g., different storage engines) to be plugged in without altering the core contracts.

---

## Quick Start

Add the crates to your `Cargo.toml`:

```toml
[dependencies]
monumentum_handler = "0.1"
monumentum_core = "0.1"
```

Or, if you only need the contracts:

```toml
[dependencies]
monumentum_handler = "0.1"
```

### Prerequisites

- Rust **1.96.0** or later
- Cargo

No system libraries are required.

---

## Usage Examples

### Create an in‑memory database

```rust
use monumentum_core::catalog::Catalog;
use monumentum_core::store::storage::{InMemoryStorage, StorageEngine};
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;
    catalog.create_table(schema)?;

    let mut storage = InMemoryStorage::new();
    storage.save_catalog(&catalog)?;

    assert!(storage.get_table("users").is_some());
    Ok(())
}
```

### Persist to a file

```rust
use monumentum_core::catalog::Catalog;
use monumentum_core::store::storage::FileStorage;
use monumentum_core::store::storage::StorageEngine;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("database.monumentum");
    let mut storage = FileStorage::open(path)?;

    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "books",
        vec![
            ColumnDef::new("title", DataType::Text),
            ColumnDef::new("year", DataType::Integer),
        ],
    )?;
    catalog.create_table(schema)?;

    storage.save_catalog(&catalog)?;
    storage.checkpoint()?;   // write snapshot
    storage.close()?;
    Ok(())
}
```

### Read a file database

```rust
use monumentum_core::store::storage::FileStorage;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut storage = FileStorage::open(Path::new("database.monumentum"))?;
    let catalog = storage.reload_from_disk()?;
    println!("Tables: {:?}", catalog.tables().map(|(n, _)| n).collect::<Vec<_>>());
    Ok(())
}
```

---

## Documentation

Full API documentation for each crate is available in their respective `README.md` files:

- [`monumentum_handler`](./monumentum_handler/README.md) – contracts, types, traits, validation, errors.
- [`monumentum_core`](./monumentum_core/README.md) – storage backends, serialization, WAL, table management.

To generate rustdoc locally:

```bash
cargo doc --workspace --no-deps
```

---

## Testing

Run all workspace tests:

```bash
cargo test --workspace
```

The test suite includes:

- Unit tests for validation, types, and serialization.
- Integration tests for catalog/table operations and file storage.
- Corruption and recovery tests for the write‑ahead log.

Additional checks (formatting, linting, documentation):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```

A `Makefile` with a `ci` target is provided for convenience:

```bash
make ci
```

---

## Contributing

Contributions are welcome! Please follow the existing style and ensure all checks pass before submitting a pull request.

The project enforces:

- `#![forbid(unsafe_code)]`
- `cargo fmt`
- `cargo clippy -D warnings`
- `cargo doc --no-deps`

For larger changes, consider opening an issue first to discuss the design.

---

## License

Licensed under the MIT License. See [LICENSE](./LICENSE) for details.
