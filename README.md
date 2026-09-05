# Monumentum

[![CI](https://github.com/mroczect/monumentum/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/mroczect/monumentum/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.96%2B-blue.svg)](https://blog.rust-lang.org/2024/06/13/Rust-1.96.0.html)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/mroczect/monumentum/graphs/commit-activity)
[![GitHub stars](https://img.shields.io/github/stars/mroczect/monumentum.svg?style=social&label=Star)](https://github.com/mroczect/monumentum/stargazers)

**Monumentum** is a lightweight, embedded database system written in Rust. It follows a strict **contracts‑first** architecture, separating pure data types and traits from concrete implementations. The project aims to provide a foundation for building reliable, safe, and modular storage backends without external runtime dependencies.

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

Monumentum is not a full‑featured DBMS; instead it offers the essential building blocks for constructing database engines and storage systems:

- **`monumentum_handler`** – a _pure contracts_ crate containing data types, traits, constants, validation, and error definitions. It has **zero dependencies** beyond the Rust standard library.
- **`monumentum_core`** – the reference implementation of those contracts, providing a page‑based storage engine, buffer pool, B‑tree indexes, serialization, and write‑ahead logging (WAL).

The design follows these principles:

- **Safety** – `#![forbid(unsafe_code)]` throughout.
- **Correctness** – fallible constructors, strict validation, and checksums protect against corruption and invalid states.
- **Modularity** – contracts are fully decoupled from implementations, allowing alternative backends to be plugged in.
- **Resource limits** – hard `MAX_*` constants prevent memory exhaustion from malicious input.
- **Durability** – WAL with CRC32 checksums and atomic checkpointing ensure crash recovery.

---

## Crates

| Crate                | Description                                                             | Documentation                  |
| -------------------- | ----------------------------------------------------------------------- | ------------------------------ |
| `monumentum_handler` | Contracts: types, traits, errors, validation, constants                 | `monumentum_handler/README.md` |
| `monumentum_core`    | Reference implementation: storage, WAL, serialization, table management | `monumentum_core/README.md`    |

---

## Features

### Core Contracts (`monumentum_handler`)

- Rich value system: `Null`, `Integer`, `Float`, `Text`, `Blob`, `Boolean`
- Schema definition with data types, nullability, primary keys, uniqueness, check constraints, foreign keys, allowed values
- Trait contracts: `StorageEngine`, `CatalogStore`, `Index`, `TableStore`
- Error hierarchy (`DbError`, `ErrorKind`) with full source tracing
- Type wrappers (`Integer`, `Float`, `Text`, `Blob`) with built‑in size limits
- Name validation utilities

### Storage Engine (`monumentum_core`)

- Page‑based storage (4096‑byte pages with header and body)
- Buffer pool with LRU/clock eviction and pin counts
- File locking to prevent concurrent writers
- CRC32 checksums for page integrity verification
- B‑tree on‑disk index with `insert`, `lookup`, `delete`, and `range_scan`
- Write‑ahead logging (WAL) with LSN tracking and delta records
- Checkpoint mechanism that atomically writes catalog and dirty pages
- Crash recovery by replaying WAL records
- Table storage that manages rows across multiple data pages
- Serialization for all core types with strict size limits

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

### Storage Engine Internals

```
+------------------+       +------------------+
|   FileStorage    |       |  InMemoryStorage |
+--------+---------+       +------------------+
         |
         v
   +------------+         +------------------+
   | BufferPool | <-----> |      Pager       |
   +------------+         +------------------+
         |
         v
   +------------+         +------------------+
   |  Page I/O  | <-----> |  File (locked)   |
   +------------+         +------------------+
         |
         v
   +------------+         +------------------+
   |   Catalog  | <-----> |   WAL (append)   |
   +------------+         +------------------+
         |
         v
   +------------+
   | B‑tree Index|
   +------------+
```

- **Pager** owns the database file and enforces an exclusive lock. It validates page checksums on read and computes them on write.
- **BufferPool** caches a fixed number of pages, tracks dirty pages, and evicts using an LRU‑like clock.
- **WAL** records either full page snapshots (`PageWrite`) or table metadata updates (`TableMetaUpdate`) with LSNs.
- **Checkpoint** writes the in‑memory catalog to pages, flushes all dirty pages, then truncates WAL.

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

### Create an in‑memory catalog

```rust
use monumentum_core::catalog::Catalog;
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
    assert!(catalog.get_table("users").is_some());

    Ok(())
}
```

### Persist rows to a file database

```rust
use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::traits::StorageEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("database.monumentum");

    // Schema with primary key
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let schema = TableSchema::try_new(
        "books",
        vec![
            id_col,
            ColumnDef::new("title", DataType::Text),
        ],
    )?;

    // Open storage (create if absent)
    let mut storage = FileStorage::open(path, 10)?;

    // Create table
    storage.create_table(schema)?;

    // Insert rows
    let row1 = Row::new(vec![
        Value::from(1i64),
        Value::try_from("The Rust Programming Language".to_string())?,
    ]);
    storage.insert_row("books", &row1)?;

    let row2 = Row::new(vec![
        Value::from(2i64),
        Value::try_from("Designing Data‑Intensive Applications".to_string())?,
    ]);
    storage.insert_row("books", &row2)?;

    // Lookup by primary key
    let found = storage.get_row_by_key("books", &Value::from(1i64))?;
    assert_eq!(found, Some(row1));

    // Persist and close
    storage.checkpoint()?;
    storage.close()?;

    Ok(())
}
```

### Reopen and recover from WAL

```rust
use monumentum_core::store::storage::FileStorage;
use monumentum_handler::traits::StorageEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("database.monumentum");

    // Reopen; the engine automatically applies WAL records
    let mut storage = FileStorage::open(path, 10)?;

    // Read back rows
    let row = storage.get_row("books", 0)?;
    println!("First row: {:?}", row);

    Ok(())
}
```

### On‑disk B‑tree index

```rust
use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::pager::Pager;
use monumentum_core::index::btree::BTreeOnDisk;
use monumentum_core::index::key::IndexKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("btree.db");
    let pager = Pager::open(path)?;
    let mut pool = BufferPool::new(pager, 10)?;

    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root_id = btree.root_page_id();

    // Insert keys
    for i in 0..100_i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root_id, IndexKey::Integer(i), i as u64)?;
    }

    // Range scan
    let mut result = Vec::new();
    BTreeOnDisk::range_scan_static(
        &mut pool,
        root_id,
        &IndexKey::Integer(10),
        &IndexKey::Integer(20),
        &mut result,
    )?;

    assert_eq!(result.len(), 10);

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
- File locking and checksum validation tests.
- B‑tree insert, lookup, delete, and range scan tests.
- Property‑based tests for roundtrips and name validation.

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
