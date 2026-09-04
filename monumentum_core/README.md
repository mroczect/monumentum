# monumentum_core

Reference implementation of the `monumentum_handler` contracts.  
Provides concrete storage backends, serialization, indexing, and table/catalog management.

## Features

- **Catalog** – B-tree map of named tables with CRUD and rename operations.
- **Table** – Row storage with constraint validation, unique indexes, and read-only mode.
- **Index** – `IndexKey` and `HashIndex` for efficient lookups.
- **Serialization** – Binary encoding/decoding for all core types.
- **Storage** – In-memory and file-backed engines with WAL and snapshot support.
- **WAL** – Write-ahead log for durability.
- **File utilities** – Atomic writes, file locking, and safe file operations.
- **Recovery** – Read records from WAL.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
monumentum_core = "0.1"
monumentum_handler = "0.1"
```

## API Reference

### Catalog (`catalog.rs`)

```rust
pub struct Catalog {
    tables: BTreeMap<String, Table>,
}
```

Methods:

- `new() -> Self`
- `create_table(&mut self, schema: TableSchema) -> Result<(), DbError>`
- `drop_table(&mut self, name: &str) -> Result<(), DbError>`
- `replace_table(&mut self, name: &str, table: Table) -> Result<(), DbError>`
- `get_table(&self, name: &str) -> Option<&Table>`
- `get_table_mut(&mut self, name: &str) -> Option<&mut Table>`
- `tables(&self) -> impl Iterator<Item = (&str, &Table)>`
- `len(&self) -> usize`
- `is_empty(&self) -> bool`
- `rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>`

Implements: `Debug`, `Default`, `Clone`, `PartialEq`.

### Table (`table.rs`)

```rust
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
    unique_indexes: Vec<Option<HashIndex>>,
    read_only: bool,
}
```

Methods:

- `new(schema: TableSchema) -> Self`
- `rename_schema(&mut self, new_name: &str) -> Result<(), DbError>`
- `schema(&self) -> &TableSchema`
- `rows(&self) -> &[Row]`
- `insert(&mut self, row: &Row) -> Result<(), DbError>`
- `len(&self) -> usize`
- `is_empty(&self) -> bool`
- `get(&self, index: usize) -> Option<&Row>`
- `replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>`
- `set_cell(&mut self, row_idx, col_idx, value: Value) -> Result<(), DbError>`
- `set_column_allowed_values(&mut self, col_idx, values) -> Result<(), DbError>`
- `lookup_by_unique(&self, col_idx, value: &Value) -> Option<&Row>`
- `is_read_only(&self) -> bool`
- `set_read_only(&mut self, value: bool)`
- `get_column_by_name(&self, name: &str) -> Option<&ColumnDef>`

### Index

#### `IndexKey` (`index/key.rs`)

```rust
pub enum IndexKey {
    Integer(i64),
    Float(u64),   // normalized bit representation
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
}
```

Methods:

- `from_value(v: &Value) -> Option<Self>`

#### `HashIndex` (`index/hash_index.rs`)

```rust
pub struct HashIndex {
    map: HashMap<IndexKey, Vec<usize>>,
}
```

Methods:

- `new() -> Self`
- `insert(&mut self, key: IndexKey, row_idx: usize)`
- `contains(&self, key: &IndexKey) -> bool`
- `clear(&mut self)`
- `get_indices(&self, key: &IndexKey) -> Option<&[usize]>`
- `remove(&mut self, key: &IndexKey, row_idx: usize)`

### Serialization (`serde/`)

```rust
pub fn encode_catalog(catalog: &Catalog) -> Result<Vec<u8>, DbError>;
pub fn decode_catalog(data: &[u8]) -> Result<Catalog, DbError>;
```

Internal traits `Encode` and `Decode` are implemented for all supported types.

Format details:

- Little-endian encoding
- Tags for each value type: Null=0, Integer=1, Float=2, Text=3, Blob=4, Boolean=5
- Size limits enforced during decoding

### Storage

#### `StorageEngine` trait (`store/storage.rs`)

```rust
pub trait StorageEngine {
    fn get_catalog(&self) -> &Catalog;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
}
```

#### `InMemoryStorage`

```rust
pub struct InMemoryStorage {
    catalog: Catalog,
}
```

Methods:

- `new() -> Self`

#### `FileStorage`

```rust
pub struct FileStorage {
    data_path: PathBuf,
    wal: Wal,
    catalog: Catalog,
    current_seq: u64,
}
```

Methods:

- `open(path: &Path) -> Result<Self, DbError>`
- `sync(&mut self) -> Result<(), DbError>`
- `checkpoint(&mut self) -> Result<(), DbError>`
- `reload_from_disk(&mut self) -> Result<Catalog, DbError>`
- `close(self) -> Result<(), DbError>`

### WAL (`store/wal.rs`)

```rust
pub struct Wal {
    file: Option<File>,
}
```

Methods:

- `open(path: &Path) -> Result<Self, DbError>`
- `append(&mut self, payload: &[u8]) -> Result<(), DbError>`
- `sync(&self) -> Result<(), DbError>`
- `read_all(&mut self) -> Result<Vec<Vec<u8>>, DbError>`
- `truncate(&mut self) -> Result<(), DbError>`
- `unlock(&mut self) -> Result<(), DbError>`

### Append Log (`store/append_log.rs`)

```rust
pub fn append_record(file: &mut File, payload: &[u8]) -> Result<(), DbError>
pub fn read_records(file: &mut File) -> Result<Vec<Vec<u8>>, DbError>
```

Record format:

- Magic (4 bytes)
- Version (4 bytes)
- Length (8 bytes)
- CRC32 checksum (4 bytes)
- Payload (variable)

### File Utilities (`store/file.rs`)

```rust
pub fn open_or_create(path: &Path) -> Result<File, DbError>
pub fn read_file(path: &Path) -> Result<Vec<u8>, DbError>
pub fn write_all_atomic(path: &Path, data: &[u8]) -> Result<(), DbError>
pub fn append_to_file(file: &mut File, data: &[u8]) -> Result<(), DbError>
pub fn sync_file(file: &File) -> Result<(), DbError>
```

### Recovery (`store/recovery.rs`)

```rust
pub struct RecoveryResult {
    pub records: Vec<Vec<u8>>,
}

pub fn recover_wal(path: &Path) -> Result<RecoveryResult, DbError>
```

## Usage Example

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

## Testing

Run unit and integration tests:

```bash
cargo test
```

## License

MIT
