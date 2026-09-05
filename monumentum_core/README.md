# monumentum_core

**Reference implementation of the `monumentum_handler` contracts.**  
Provides concrete storage backends, serialization, indexing, and table/catalog management built on a page-based storage engine with Write-Ahead Logging (WAL), buffer pool caching, and B-tree indexes.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Module: `buffer_pool`](#module-buffer_pool)
- [Module: `pager`](#module-pager)
- [Module: `page`](#module-page)
- [Module: `catalog`](#module-catalog)
- [Module: `index`](#module-index)
  - [`btree`](#submodule-btree)
  - [`btree_index`](#submodule-btree_index)
  - [`hash_index`](#submodule-hash_index)
  - [`key`](#submodule-key)
- [Module: `serde`](#module-serde)
- [Module: `store`](#module-store)
  - [`append_log`](#submodule-append_log)
  - [`file`](#submodule-file)
  - [`recovery`](#submodule-recovery)
  - [`storage`](#submodule-storage)
  - [`wal`](#submodule-wal)
- [Module: `table`](#module-table)
- [Module: `table_storage`](#module-table_storage)
- [Error Handling](#error-handling)
- [File Format & Durability](#file-format--durability)
- [Examples](#examples)
- [Testing](#testing)

---

## Overview

`monumentum_core` is a low-level storage engine library that persists data to disk using a fixed-size page format. It replaces earlier in-memory storage with a robust architecture that includes:

- A **buffer pool** to cache pages and reduce disk I/O.
- A **pager** that performs raw page reads/writes and ensures data integrity via CRC32 checksums.
- A **B‑tree index** for efficient point lookups and range scans.
- A **write‑ahead log (WAL)** that records page changes and table metadata updates to guarantee durability and support crash recovery.
- A **catalog** that stores table schemas and metadata across multiple disk pages.
- **Table storage** that manages rows within data pages.

All components are designed to work together to provide a reliable, single‑file database backend (with a separate WAL file) suitable for embedded applications.

---

## Architecture

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

- **BufferPool** holds a fixed number of pages in memory. Pages are pinned while in use, and unpinned pages are eligible for eviction. Dirty pages are written back to disk during flush or eviction.
- **Pager** owns the database file and enforces an exclusive lock. It validates page checksums on read and computes them on write.
- **WAL** is an append‑only log file that stores either full page snapshots (`PageWrite`) or table metadata updates (`TableMetaUpdate`) with LSNs.
- **Checkpoint** writes the in‑memory catalog to pages, flushes all dirty pages, then truncates the WAL.

---

## Module: `buffer_pool`

### Type: `BufferPool`

A simple page cache that stores up to `capacity` pages. It uses a clock‑based LRU approximation for eviction.

```rust
pub struct BufferPool { /* private */ }
```

#### Public Methods

| Method                                                                    | Description                                                                                     |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `new(pager: Pager, capacity: usize) -> Result<Self, DbError>`             | Creates a new buffer pool. `capacity` must be > 0.                                              |
| `get_page(&mut self, page_id: u32) -> Result<&mut Page, DbError>`         | Returns a mutable reference to a page, loading it from disk if necessary. Increments pin count. |
| `unpin_page(&mut self, page_id: u32, dirty: bool) -> Result<(), DbError>` | Decrements pin count. If `dirty` is `true`, marks the page as dirty.                            |
| `mark_dirty(&mut self, page_id: u32) -> Result<(), DbError>`              | Explicitly marks a page as dirty.                                                               |
| `flush_page(&mut self, page_id: u32) -> Result<(), DbError>`              | Writes the page to disk if it is dirty and clears the dirty flag.                               |
| `flush_all(&mut self) -> Result<(), DbError>`                             | Flushes all dirty pages and syncs the file.                                                     |
| `allocate_page(&mut self, page_type: PageType) -> Result<u32, DbError>`   | Allocates a new page via the pager and adds it to the pool.                                     |
| `evict_one(&mut self) -> Result<(), DbError>`                             | Evicts the least‑recently‑used unpinned page. If dirty, flushes it first.                       |
| `page_count(&self) -> u32`                                                | Returns total number of pages in the database file.                                             |
| `dirty_page_ids(&self) -> Vec<u32>`                                       | Returns a list of page IDs currently marked dirty.                                              |

> **Note:** `get_page` must be paired with `unpin_page` to avoid pin leakage.

---

## Module: `pager`

### Type: `Pager`

Handles low‑level page I/O on a single file. It enforces an exclusive lock and verifies CRC32 checksums.

```rust
pub struct Pager { /* private */ }
```

#### Public Methods

| Method                                                                  | Description                                                                                                     |
| ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `open(path: &Path) -> Result<Self, DbError>`                            | Opens or creates the database file, locks it exclusively, and validates file size is a multiple of `PAGE_SIZE`. |
| `read_page(&mut self, page_id: u32) -> Result<Page, DbError>`           | Reads a page from disk and verifies its checksum. Returns `DbError::Corruption` on mismatch.                    |
| `write_page(&mut self, page: &Page) -> Result<(), DbError>`             | Computes the page checksum and writes it to disk, then calls `sync_data`.                                       |
| `allocate_page(&mut self, page_type: PageType) -> Result<u32, DbError>` | Appends a new zero‑initialized page to the file, updates `page_count`, and syncs.                               |
| `free_page(&mut self, page_id: u32) -> Result<(), DbError>`             | Marks the page as `PageType::Freelist` and resets its header fields.                                            |
| `sync(&mut self) -> Result<(), DbError>`                                | Calls `sync_all` on the underlying file.                                                                        |
| `page_count(&self) -> u32`                                              | Returns the number of pages currently in the file.                                                              |

> **Drop behaviour:** The file lock is automatically released when `Pager` is dropped.

---

## Module: `page`

### Type: `PageType`

```rust
pub enum PageType {
    Meta = 0,
    Freelist = 1,
    TableMeta = 2,
    Data = 3,
    Index = 4,
    Overflow = 5,
}
```

### Type: `PageHeader`

```rust
pub struct PageHeader {
    pub page_id: u32,
    pub page_type: PageType,
    pub free_space_offset: u16,
    pub cell_count: u16,
    pub checksum: u32,
    pub flags: u32,
}
```

| Method                    | Description                                             |
| ------------------------- | ------------------------------------------------------- |
| `new(page_id, page_type)` | Creates a header with default values.                   |
| `to_bytes()`              | Serializes the header to a 16‑byte little‑endian array. |
| `from_bytes(&[u8;16])`    | Deserializes the header.                                |

### Type: `Page`

```rust
pub struct Page {
    pub header: PageHeader,
    pub data: [u8; PAGE_BODY_SIZE],
}
```

| Method                    | Description                                                                    |
| ------------------------- | ------------------------------------------------------------------------------ |
| `new(page_id, page_type)` | Creates a blank page.                                                          |
| `as_bytes()`              | Returns full 4096‑byte representation.                                         |
| `from_bytes(&[u8])`       | Validates size and parses header/body.                                         |
| `compute_checksum()`      | Computes CRC32 (IEEE) over the header (with checksum field set to 0) and body. |

**Constants**

| Constant                          | Value                | Description                         |
| --------------------------------- | -------------------- | ----------------------------------- |
| `PAGE_SIZE`                       | 4096                 | Total page size in bytes.           |
| `PAGE_HEADER_SIZE`                | 16                   | Size of the page header.            |
| `PAGE_BODY_SIZE`                  | 4080                 | Remaining bytes for page content.   |
| `META_PAGE_ID`                    | 0                    | ID of the meta page.                |
| `META_LSN_OFFSET`                 | 0                    | Offset of LSN in meta page.         |
| `META_CATALOG_PAGE_OFFSET`        | 8                    | Offset of catalog root page ID.     |
| `META_LAST_CHECKPOINT_LSN_OFFSET` | 12                   | Offset of last checkpoint LSN.      |
| `CATALOG_PAGE_HEADER_SIZE`        | 8                    | Header size for catalog data pages. |
| `CATALOG_CHUNK_SIZE`              | `PAGE_BODY_SIZE - 8` | Maximum catalog bytes per page.     |
| `BTREE_NODE_HEADER_SIZE`          | 7                    | Header size for B‑tree nodes.       |
| `DATA_PAGE_HEADER_SIZE`           | 8                    | Header size for row data pages.     |

---

## Module: `catalog`

### Type: `Catalog`

A `BTreeMap<String, Table>` that holds table schemas and metadata. Implements `CatalogStore`.

```rust
pub struct Catalog { /* private */ }
```

| Method                                               | Description                                            |
| ---------------------------------------------------- | ------------------------------------------------------ |
| `new()`                                              | Creates an empty catalog.                              |
| `create_table(&mut self, schema: TableSchema)`       | Adds a new table; rejects empty names and duplicates.  |
| `drop_table(&mut self, name: &str)`                  | Removes a table by name.                               |
| `replace_table(&mut self, name: &str, table: Table)` | Replaces an existing table; schema name must match.    |
| `get_table(&self, name: &str)`                       | Returns a reference to a table.                        |
| `get_table_mut(&mut self, name: &str)`               | Returns a mutable reference.                           |
| `tables(&self)`                                      | Iterates over `(name, table)` pairs.                   |
| `len()`                                              | Number of tables.                                      |
| `is_empty()`                                         | Whether catalog is empty.                              |
| `rename_table(&mut self, old, new)`                  | Renames a table; rejects conflicts and missing tables. |

---

## Module: `index`

### Submodule: `btree`

#### Type: `BTreeOnDisk`

```rust
pub struct BTreeOnDisk {
    root_page_id: u32,
}
```

| Method                                                             | Description                                                             |
| ------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| `create(buffer_pool: &mut BufferPool)`                             | Creates a new empty B‑tree by allocating a root index page.             |
| `root_page_id()`                                                   | Returns the current root page ID.                                       |
| `insert_static(buffer_pool, root_page_id: &mut u32, key, value)`   | Inserts a key‑value pair. `root_page_id` may be updated if root splits. |
| `lookup_static(buffer_pool, root_page_id, key)`                    | Looks up a key and returns its associated value (usually a row index).  |
| `delete_static(buffer_pool, root_page_id, key)`                    | Deletes a key and returns the value if it existed.                      |
| `range_scan_static(buffer_pool, root_page_id, start, end, result)` | Collects all key‑value pairs in `[start, end)`.                         |

> All methods are static and require an external `&mut BufferPool`. This design allows multiple B‑trees to share the same buffer pool.

### Submodule: `btree_index`

#### Type: `BTreeIndex`

In‑memory B‑tree index that maps `IndexKey` → `Vec<usize>` (row indices). Implements the `Index` trait.

| Method                 | Description                                  |
| ---------------------- | -------------------------------------------- |
| `new()`                | Creates an empty index.                      |
| `insert(key, row_idx)` | Adds a row index to the key.                 |
| `contains(key)`        | Returns whether the key exists.              |
| `clear()`              | Removes all entries.                         |
| `get_indices(key)`     | Returns slice of row indices for the key.    |
| `remove(key, row_idx)` | Removes a row index; deletes key when empty. |

### Submodule: `hash_index`

#### Type: `HashIndex`

Similar to `BTreeIndex` but uses `HashMap<IndexKey, Vec<usize>>`.

| Method                 | Description                   |
| ---------------------- | ----------------------------- |
| `new()`                | Creates an empty hash index.  |
| `insert(key, row_idx)` | Adds a row index.             |
| `contains(key)`        | Checks existence.             |
| `clear()`              | Clears the index.             |
| `get_indices(key)`     | Returns slice of row indices. |
| `remove(key, row_idx)` | Removes a row index.          |

### Submodule: `key`

#### Enum: `IndexKey`

```rust
pub enum IndexKey {
    Integer(i64),
    Float(u64),   // normalized IEEE 754 bits
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
}
```

| Method               | Description                                                                               |
| -------------------- | ----------------------------------------------------------------------------------------- |
| `from_value(&Value)` | Converts a `Value` to an `IndexKey`; returns `None` for unsupported types (e.g., `Null`). |
| `to_bytes()`         | Serializes key with a 1‑byte type tag followed by payload.                                |
| `from_bytes(&[u8])`  | Deserializes key and validates tag/length.                                                |

---

## Module: `serde`

Provides binary encoding/decoding for all core types.

### Functions

| Function                                               | Description                                                |
| ------------------------------------------------------ | ---------------------------------------------------------- |
| `encode_catalog(&Catalog) -> Result<Vec<u8>, DbError>` | Serializes the entire catalog.                             |
| `decode_catalog(&[u8]) -> Result<Catalog, DbError>`    | Deserializes a catalog, enforcing version and size limits. |
| `encode_row(&Row) -> Result<Vec<u8>, DbError>`         | Serializes a row with a 4‑byte length prefix.              |
| `decode_row(&[u8]) -> Result<Row, DbError>`            | Deserializes a row from the length‑prefixed format.        |

### Internal Traits

`Encode` and `Decode` are implemented for:

- Primitive integers (`u8`, `u32`, `u64`, `i64`, `f64`, `bool`)
- `Option<T>`, `Vec<T>`, `&T`
- `Value`, `Row`, `TableSchema`, `ColumnDef`, `Table`, `Catalog`

**Encoding Details**

- Little‑endian byte order.
- Value tags: `Null=0`, `Integer=1`, `Float=2`, `Text=3`, `Blob=4`, `Boolean=5`.
- Length prefixes are `u64` for byte buffers and `u32` for vector counts.
- Maximum read sizes are enforced to prevent memory exhaustion (`MAX_READ_BYTES = 64 MiB`, `MAX_VEC_ELEMENTS = 1_000_000`).

---

## Module: `store`

### Submodule: `append_log`

#### Enum: `WalRecordType`

```rust
pub enum WalRecordType {
    Snapshot = 0,
    PageWrite = 1,
    TableMetaUpdate = 2,
}
```

#### Struct: `WalRecord`

```rust
pub struct WalRecord {
    pub lsn: u64,
    pub record_type: WalRecordType,
    pub data: Vec<u8>,
}
```

#### Functions

| Function                                          | Description                                                                           |
| ------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `append_record(file, payload)`                    | Appends a single log record with magic, version, length, CRC32 checksum, and payload. |
| `append_wal_record(file, lsn, record_type, data)` | Wraps data with LSN and record type, then calls `append_record`.                      |
| `read_records(file)`                              | Reads all raw records, validating magic, version, length, and checksum.               |
| `read_wal_records(file)`                          | Parses raw records into structured `WalRecord` items.                                 |

**Record format (on disk):**

```
+----------------+----------------+----------------+----------------+----------------+
| Magic (4)      | Version (4)    | Length (8)     | CRC32 (4)      | Payload (...)  |
+----------------+----------------+----------------+----------------+----------------+
```

### Submodule: `file`

Utility functions for file operations.

| Function                       | Description                                                                          |
| ------------------------------ | ------------------------------------------------------------------------------------ |
| `open_or_create(path)`         | Opens or creates a file with read/write permissions and no symlink following (Unix). |
| `read_file(path)`              | Reads entire file into a `Vec<u8>`.                                                  |
| `write_all_atomic(path, data)` | Writes data to a temporary file, fsyncs, then renames to target path.                |
| `append_to_file(file, data)`   | Appends bytes to a file (no automatic sync).                                         |
| `sync_file(file)`              | Calls `sync_all` on the file.                                                        |

### Submodule: `recovery`

#### Struct: `RecoveryResult`

```rust
pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}
```

#### Function

```rust
pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError>
```

Opens the WAL, reads all raw records, and returns them.

### Submodule: `storage`

#### Trait: `StorageEngine`

```rust
pub trait StorageEngine {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
    fn insert_row(&mut self, table: &str, row: &Row) -> Result<(), DbError>;
    fn get_row(&mut self, table: &str, row_idx: usize) -> Result<Option<Row>, DbError>;
    fn set_cell(&mut self, table: &str, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    fn replace_rows(&mut self, table: &str, rows: Vec<Row>) -> Result<(), DbError>;
    fn checkpoint(&mut self) -> Result<(), DbError>;
    fn get_row_by_key(&mut self, table: &str, key: &Value) -> Result<Option<Row>, DbError>;
}
```

#### Type: `FileStorage`

The main disk‑backed storage engine.

```rust
pub struct FileStorage { /* private */ }
```

| Method                                       | Description                                                                               |
| -------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `open(path: &Path, cache_capacity: usize)`   | Opens or creates the database and WAL, loads catalog, applies WAL, and runs a checkpoint. |
| `checkpoint()`                               | Writes catalog to pages, flushes all dirty pages, updates meta page, truncates WAL.       |
| `sync()`                                     | Syncs the WAL file.                                                                       |
| `close(self)`                                | Calls `checkpoint()` and unlocks WAL.                                                     |
| `save_catalog(&mut self, catalog: &Catalog)` | Replaces in‑memory catalog and writes it to pages.                                        |
| `get_catalog()`                              | Returns a reference to the catalog.                                                       |
| `get_table(name)`                            | Returns table metadata from catalog.                                                      |
| `get_row_by_key(table, key)`                 | Looks up a row using primary key B‑tree index.                                            |

> `FileStorage` implements the `StorageEngine` trait fully, including row operations.

#### Type: `InMemoryStorage`

A simple in‑memory implementation that only stores the catalog and does not support row operations.

```rust
pub struct InMemoryStorage { /* private */ }
```

| Method                             | Description                         |
| ---------------------------------- | ----------------------------------- |
| `new()`                            | Creates an empty in‑memory storage. |
| `save_catalog(&mut self, catalog)` | Replaces catalog.                   |
| `get_catalog()`                    | Returns reference to catalog.       |
| `get_table(name)`                  | Returns table metadata.             |

> `InMemoryStorage` implements `StorageEngine` but returns `Unsupported` for row methods.

### Submodule: `wal`

#### Type: `Wal`

```rust
pub struct Wal {
    file: Option<File>,
}
```

| Method                                      | Description                                             |
| ------------------------------------------- | ------------------------------------------------------- |
| `open(path)`                                | Opens or creates the WAL file and locks it exclusively. |
| `append(payload)`                           | Appends a raw record.                                   |
| `append_wal_record(lsn, record_type, data)` | Appends a structured WAL record.                        |
| `sync()`                                    | Syncs the WAL file.                                     |
| `read_all()`                                | Reads all raw records.                                  |
| `read_wal_records()`                        | Reads and parses all structured records.                |
| `truncate()`                                | Truncates the WAL file and syncs.                       |
| `unlock()`                                  | Releases the file lock.                                 |

---

## Module: `table`

### Type: `Table`

A metadata container for a table; no longer holds rows directly.

```rust
pub struct Table {
    schema: TableSchema,
    read_only: bool,
    data_page_id: Option<u32>,
    index_root_page_id: Option<u32>,
    next_row_id: u64,
}
```

| Method                                                 | Description                                                   |
| ------------------------------------------------------ | ------------------------------------------------------------- |
| `new(schema)`                                          | Creates a new table with default values.                      |
| `rename_schema(new_name)`                              | Renames the schema while preserving columns.                  |
| `schema()`                                             | Returns schema reference.                                     |
| `is_read_only()` / `set_read_only(bool)`               | Read‑only flag.                                               |
| `data_page_id()` / `set_data_page_id(u32)`             | First data page ID.                                           |
| `index_root_page_id()` / `set_index_root_page_id(u32)` | B‑tree root page ID.                                          |
| `clear_index_root_page_id()`                           | Sets index root to `None`.                                    |
| `next_row_id()` / `set_next_row_id(u64)`               | Row counter.                                                  |
| `increment_next_row_id()`                              | Returns current value and increments it, with overflow check. |

---

## Module: `table_storage`

### Type: `TableStorage`

Handles row storage on data pages.

```rust
pub struct TableStorage {
    first_data_page_id: u32,
}
```

| Method                                                | Description                                               |
| ----------------------------------------------------- | --------------------------------------------------------- |
| `new(buffer_pool)`                                    | Allocates the first data page and initialises it.         |
| `first_data_page_id()`                                | Returns the first page ID.                                |
| `insert_row_static(buffer_pool, first_page_id, row)`  | Serializes and appends a row to the data page chain.      |
| `get_row_static(buffer_pool, first_page_id, row_idx)` | Reads and deserializes a row by its logical index.        |
| `insert_row(buffer_pool, row)`                        | Convenience wrapper.                                      |
| `get_row(buffer_pool, row_idx)`                       | Convenience wrapper.                                      |
| `clear_static(buffer_pool, first_page_id)`            | Marks all pages in the chain as empty and resets headers. |
| `from_first_page_id(id)`                              | Creates a `TableStorage` from an existing first page ID.  |

> `TableStorage` also implements the `TableStore` trait from `monumentum_handler`, but most methods return `Unsupported` because they require an external buffer pool.

---

## Error Handling

All fallible functions return `Result<T, DbError>`, where `DbError` is defined in `monumentum_handler`. Common error variants include:

- `InvalidOperation` – e.g., duplicate table, invalid page ID.
- `ConstraintViolation` – e.g., duplicate key in B‑tree.
- `Corruption` – e.g., checksum mismatch, invalid magic.
- `TableNotFound` – e.g., table does not exist.
- `Unsupported` – feature not yet implemented.

`DbError` can be converted from `std::io::Error` for file‑related failures.

---

## File Format & Durability

- **Database file** consists of fixed‑size 4096‑byte pages.
- **Meta page** (page 0) stores LSN, catalog root page ID, and last checkpoint LSN.
- **Catalog pages** are chained via a `next_page_id` field; each stores a chunk of the serialized catalog.
- **Data pages** store rows with a simple format: `[4‑byte row length][row bytes]`.
- **WAL file** contains records with LSN, type, and payload, all protected by CRC32.
- **Checkpoint** writes the catalog and all dirty pages, then truncates WAL.

> **Recovery** on startup:
>
> 1. Read meta page (LSN, catalog root).
> 2. Load catalog from pages.
> 3. Apply WAL records with LSN > last checkpoint LSN.
> 4. Reload catalog from pages (after WAL applied).
> 5. Run checkpoint to flush everything.

---

## Examples

### Example 1: Catalog Serialization

```rust
use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{encode_catalog, decode_catalog};
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

    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;

    assert_eq!(catalog, decoded);

    Ok(())
}
```

### Example 2: File Storage with Primary Key

```rust
use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::traits::StorageEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("test.db");

    // Create schema with primary key
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let schema = TableSchema::try_new("users", vec![id_col, ColumnDef::new("name", DataType::Text)])?;

    // Open storage
    let mut storage = FileStorage::open(path, 10)?;

    // Create table
    storage.create_table(schema)?;

    // Insert row
    let row = Row::new(vec![Value::from(1i64), Value::from("Alice")]);
    storage.insert_row("users", &row)?;

    // Lookup by primary key
    let found = storage.get_row_by_key("users", &Value::from(1i64))?;
    assert_eq!(found, Some(row));

    // Checkpoint and close
    storage.checkpoint()?;
    storage.close()?;

    Ok(())
}
```

### Example 3: B‑tree Operations

```rust
use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::pager::Pager;
use monumentum_core::index::btree::BTreeOnDisk;
use monumentum_core::index::key::IndexKey;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("btree.db");
    let pager = Pager::open(path)?;
    let mut pool = BufferPool::new(pager, 10)?;

    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root_id = btree.root_page_id();

    // Insert keys
    for i in 0..100_i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root_id, IndexKey::Integer(i), i as u64)?;
    }

    // Lookup
    let value = BTreeOnDisk::lookup_static(&mut pool, root_id, &IndexKey::Integer(50))?;
    assert_eq!(value, Some(50));

    // Range scan
    let mut result = Vec::new();
    BTreeOnDisk::range_scan_static(&mut pool, root_id, &IndexKey::Integer(10), &IndexKey::Integer(20), &mut result)?;
    assert_eq!(result.len(), 10);

    // Delete
    let removed = BTreeOnDisk::delete_static(&mut pool, root_id, &IndexKey::Integer(50))?;
    assert_eq!(removed, Some(50));

    Ok(())
}
```

---

## Testing

Run all workspace tests:

```bash
cargo test --workspace
```

Run only `monumentum_core` tests:

```bash
cargo test -p monumentum_core
```

Run with strict lints:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The test suite covers:

- Buffer pool eviction and dirty tracking.
- Pager checksum validation and file corruption detection.
- B‑tree insert, lookup, delete, and range scan.
- Catalog CRUD operations.
- Row serialization roundtrips.
- File storage full workflow, crash recovery, and primary key lookup.
- File locking prevents concurrent writers.
- Property‑based tests for serialization and name validation.

---

## License

MIT
