use monumentum_db::core::{Catalog, ColumnDef, DataType, Row, TableSchema, Value};
use monumentum_db::error::DbError;
use monumentum_db::store::{
    FileStorage, StorageEngine, Wal, append_record, read_records, recover_wal, write_all_atomic,
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn temp_path(name: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{}.{}.{}.test", name, pid, nanos))
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

#[test]
fn open_or_create_creates_file() {
    let path = temp_path("open_test");
    cleanup(&path);
    let file = monumentum_db::store::file::open_or_create(&path).unwrap();
    assert!(path.exists());
    drop(file);
    cleanup(&path);
}

#[test]
fn read_file_returns_bytes() {
    let path = temp_path("read_test");
    cleanup(&path);
    std::fs::write(&path, b"hello world").unwrap();
    let data = monumentum_db::store::file::read_file(&path).unwrap();
    assert_eq!(data, b"hello world");
    cleanup(&path);
}

#[test]
fn write_all_atomic_creates_file_with_content() {
    let path = temp_path("atomic_test");
    cleanup(&path);
    write_all_atomic(&path, b"atomic data").unwrap();
    let data = std::fs::read(&path).unwrap();
    assert_eq!(data, b"atomic data");
    let parent = path.parent().unwrap();
    let tmp_files: Vec<_> = std::fs::read_dir(parent)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("atomic_test"))
        .collect();
    assert_eq!(tmp_files.len(), 1);
    cleanup(&path);
}

#[test]
fn write_all_atomic_overwrites_existing() {
    let path = temp_path("atomic_overwrite");
    cleanup(&path);
    std::fs::write(&path, b"old").unwrap();
    write_all_atomic(&path, b"new data").unwrap();
    let data = std::fs::read(&path).unwrap();
    assert_eq!(data, b"new data");
    cleanup(&path);
}

#[test]
fn append_and_read_records_empty() {
    let path = temp_path("log_empty");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let records = read_records(&mut file).unwrap();
    assert!(records.is_empty());
    cleanup(&path);
}

#[test]
fn append_and_read_records_single() {
    let path = temp_path("log_single");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let payload = b"single payload".to_vec();
    append_record(&mut file, &payload).unwrap();
    let mut reader = File::open(&path).unwrap();
    let records = read_records(&mut reader).unwrap();
    assert_eq!(records, vec![payload]);
    cleanup(&path);
}

#[test]
fn append_and_read_records_multiple() {
    let path = temp_path("log_multiple");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let payloads = vec![
        b"first".to_vec(),
        b"second".to_vec(),
        b"".to_vec(),
        vec![0u8; 1024],
        b"last".to_vec(),
    ];
    for p in &payloads {
        append_record(&mut file, p).unwrap();
    }
    let mut reader = File::open(&path).unwrap();
    let records = read_records(&mut reader).unwrap();
    assert_eq!(records, payloads);
    cleanup(&path);
}

#[test]
fn append_record_rejects_too_large_payload() {
    let path = temp_path("log_too_large");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let large_payload = vec![0u8; 65 * 1024 * 1024];
    let result = append_record(&mut file, &large_payload);
    assert!(matches!(result, Err(DbError::InvalidOperation(_))));
    cleanup(&path);
}

#[test]
fn read_records_detects_bad_magic() {
    let path = temp_path("log_bad_magic");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let mut bad_header = [0u8; 20];
    bad_header[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    file.write_all(&bad_header).unwrap();
    file.sync_all().unwrap();
    let mut reader = File::open(&path).unwrap();
    assert!(matches!(
        read_records(&mut reader),
        Err(DbError::Corruption(_))
    ));
    cleanup(&path);
}

#[test]
fn read_records_detects_bad_version() {
    let path = temp_path("log_bad_version");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let mut header = [0u8; 20];
    header[0..4].copy_from_slice(&0x4D4F4E55u32.to_le_bytes());
    header[4..8].copy_from_slice(&999u32.to_le_bytes());
    header[8..16].copy_from_slice(&0u64.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    file.write_all(&header).unwrap();
    file.sync_all().unwrap();
    let mut reader = File::open(&path).unwrap();
    assert!(matches!(
        read_records(&mut reader),
        Err(DbError::Corruption(_))
    ));
    cleanup(&path);
}

#[test]
fn read_records_detects_truncated_header() {
    let path = temp_path("log_truncated_header");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    file.write_all(&[0u8; 10]).unwrap();
    file.sync_all().unwrap();
    let mut reader = File::open(&path).unwrap();
    assert!(matches!(
        read_records(&mut reader),
        Err(DbError::Corruption(_))
    ));
    cleanup(&path);
}

#[test]
fn read_records_detects_truncated_payload() {
    let path = temp_path("log_truncated_payload");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    append_record(&mut file, b"complete").unwrap();
    let full_len = file.metadata().unwrap().len();
    file.set_len(full_len - 3).unwrap();
    file.sync_all().unwrap();
    let mut reader = File::open(&path).unwrap();
    assert!(matches!(
        read_records(&mut reader),
        Err(DbError::Corruption(_))
    ));
    cleanup(&path);
}

#[test]
fn read_records_detects_checksum_mismatch() {
    let path = temp_path("log_checksum_mismatch");
    cleanup(&path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    append_record(&mut file, b"data1").unwrap();
    append_record(&mut file, b"data2").unwrap();
    file.sync_all().unwrap();

    let mut data = std::fs::read(&path).unwrap();
    let header_size = 20;
    let payload1_len = 5;
    let payload2_start = header_size + payload1_len;
    data[payload2_start + 1] ^= 0xFF;
    std::fs::write(&path, &data).unwrap();

    let mut reader = File::open(&path).unwrap();
    assert!(matches!(
        read_records(&mut reader),
        Err(DbError::Corruption(_))
    ));
    cleanup(&path);
}

#[test]
fn wal_open_creates_file() {
    let path = temp_path("wal_open");
    cleanup(&path);
    let wal = Wal::open(&path).unwrap();
    assert!(path.exists());
    drop(wal);
    cleanup(&path);
}

#[test]
fn wal_append_and_read_all() {
    let path = temp_path("wal_append");
    cleanup(&path);
    let mut wal = Wal::open(&path).unwrap();

    let payloads = vec![
        b"first".to_vec(),
        b"".to_vec(),
        vec![1, 2, 3, 4, 5],
        b"last".to_vec(),
    ];

    for p in &payloads {
        wal.append(p).unwrap();
    }

    let mut wal_reader = Wal::open(&path).unwrap();
    let records = wal_reader.read_all().unwrap();
    assert_eq!(records, payloads);

    cleanup(&path);
}

#[test]
fn wal_sync_works() {
    let path = temp_path("wal_sync");
    cleanup(&path);
    let mut wal = Wal::open(&path).unwrap();
    wal.append(b"sync test").unwrap();
    wal.sync().unwrap();
    drop(wal);

    let mut file = File::open(&path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    assert!(!data.is_empty());

    cleanup(&path);
}

#[test]
fn recover_wal_returns_records() {
    let path = temp_path("recover_wal");
    cleanup(&path);
    let mut wal = Wal::open(&path).unwrap();
    wal.append(b"record1").unwrap();
    wal.append(b"record2").unwrap();
    drop(wal);

    let result = recover_wal(&path).unwrap();
    assert_eq!(
        result.records,
        vec![b"record1".to_vec(), b"record2".to_vec()]
    );

    cleanup(&path);
}

#[test]
fn recover_wal_empty_file() {
    let path = temp_path("recover_wal_empty");
    cleanup(&path);
    {
        let _wal = Wal::open(&path).unwrap();
    }

    let result = recover_wal(&path).unwrap();
    assert!(result.records.is_empty());

    cleanup(&path);
}

#[test]
fn recover_wal_detects_corruption() {
    let path = temp_path("recover_wal_corrupt");
    cleanup(&path);
    {
        let mut wal = Wal::open(&path).unwrap();
        wal.append(b"good").unwrap();
    }

    let mut data = std::fs::read(&path).unwrap();
    data[22] ^= 0xFF;
    std::fs::write(&path, &data).unwrap();

    let result = recover_wal(&path);
    assert!(matches!(result, Err(DbError::Corruption(_))));

    cleanup(&path);
}

#[test]
fn recover_wal_truncated_file_returns_corruption() {
    let path = temp_path("recover_wal_truncated");
    cleanup(&path);
    {
        let mut wal = Wal::open(&path).unwrap();
        wal.append(b"truncate me").unwrap();
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let full_len = file.metadata().unwrap().len();
    file.set_len(full_len - 4).unwrap();
    file.sync_all().unwrap();

    let result = recover_wal(&path);
    assert!(matches!(result, Err(DbError::Corruption(_))));

    cleanup(&path);
}

#[test]
fn file_storage_save_and_load_catalog() {
    let path = temp_path("file_storage");
    cleanup(&path);

    // Buat katalog dengan satu tabel
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )
    .unwrap();
    catalog.create_table(schema).unwrap();

    // Simpan
    {
        let mut storage = FileStorage::open(&path).unwrap();
        storage.save_catalog(&catalog).unwrap();
    }

    // Muat kembali
    {
        let mut storage = FileStorage::open(&path).unwrap();
        let loaded = storage.load_catalog().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.get_table("users").is_some());
        assert_eq!(loaded.get_table("users").unwrap().len(), 0);
    }

    cleanup(&path);
}

#[test]
fn file_storage_persists_rows() {
    let path = temp_path("file_storage_rows");
    cleanup(&path);

    let mut catalog = Catalog::new();
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let schema = TableSchema::try_new("users", vec![id_col]).unwrap();
    catalog.create_table(schema).unwrap();

    {
        let mut storage = FileStorage::open(&path).unwrap();
        storage.save_catalog(&catalog).unwrap();
    }

    // Tambah baris di katalog
    {
        let mut storage = FileStorage::open(&path).unwrap();
        let mut cat = storage.load_catalog().unwrap();
        let table = cat.get_table_mut("users").unwrap();
        table.insert(Row::new(vec![Value::from(1i64)])).unwrap();
        storage.save_catalog(&cat).unwrap();
    }

    // Muat ulang, harus ada 1 baris
    {
        let mut storage = FileStorage::open(&path).unwrap();
        let loaded = storage.load_catalog().unwrap();
        let table = loaded.get_table("users").unwrap();
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(0).unwrap().get(0), Some(&Value::from(1i64)));
    }

    cleanup(&path);
}
