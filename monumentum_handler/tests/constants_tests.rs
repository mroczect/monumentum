use monumentum_handler::constants::*;
use proptest as _;

#[test]
fn test_max_name_length() {
    assert_eq!(MAX_NAME_LENGTH, 255);
}

#[test]
fn test_max_columns() {
    assert_eq!(MAX_COLUMNS, 1024);
}

#[test]
fn test_max_text_size() {
    assert_eq!(MAX_TEXT_SIZE, 16 * 1024 * 1024);
}

#[test]
fn test_max_blob_size() {
    assert_eq!(MAX_BLOB_SIZE, 64 * 1024 * 1024);
}

#[test]
fn test_max_rows_per_table() {
    assert_eq!(MAX_ROWS_PER_TABLE, 10_000_000);
}

#[test]
fn test_max_tables() {
    assert_eq!(MAX_TABLES, 1024);
}

#[test]
fn test_max_record_size() {
    assert_eq!(MAX_RECORD_SIZE, 64 * 1024 * 1024);
}

#[test]
fn test_max_snapshot_size() {
    assert_eq!(MAX_SNAPSHOT_SIZE, 256 * 1024 * 1024);
}

#[test]
fn test_max_vec_elements() {
    assert_eq!(MAX_VEC_ELEMENTS, 1_000_000);
}

#[test]
fn test_constants_relations() {
    const {
        assert!(MAX_RECORD_SIZE >= MAX_TEXT_SIZE);
        assert!(MAX_RECORD_SIZE >= MAX_BLOB_SIZE);
        assert!(MAX_BLOB_SIZE >= MAX_TEXT_SIZE);
        assert!(MAX_COLUMNS <= MAX_VEC_ELEMENTS);
        assert!(MAX_NAME_LENGTH > 0);
    }
}
