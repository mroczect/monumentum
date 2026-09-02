use core::error::Error;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::{FileStorage, InMemoryStorage};
use monumentum_workbook::{Workbook, WorkbookError};
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::path::PathBuf;
use tempfile::TempDir;

const FILE_EXTENSION: &str = "monumentum";

struct TestFile {
    _dir: TempDir,
    path: PathBuf,
}

impl TestFile {
    fn new(extension: &str) -> Result<Self, std::io::Error> {
        let dir = TempDir::new()?;
        let file_name = format!("workbook_{}.{}", uuid_simple(), extension);
        let path = dir.path().join(file_name);
        Ok(Self { _dir: dir, path })
    }

    const fn path(&self) -> &PathBuf {
        &self.path
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{}_{}", std::process::id(), nanos)
}

fn create_schema() -> Result<TableSchema, monumentum_db::error::DbError> {
    let col = ColumnDef::new("id", DataType::Integer);
    TableSchema::try_new("test_table", vec![col])
}

fn create_catalog_with_table() -> Result<Catalog, monumentum_db::error::DbError> {
    let mut catalog = Catalog::new();
    let schema = create_schema()?;
    catalog.create_table(schema)?;
    Ok(catalog)
}

#[test]
fn extension_case_insensitive_uppercase() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new("MONUMENTUM")?;
    let wb = Workbook::<FileStorage>::create_new(temp.path())?;
    assert!(wb.catalog().is_empty());
    wb.close()?;
    Ok(())
}

#[test]
fn extension_invalid_returns_error() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new("txt")?;
    let result = Workbook::<FileStorage>::create_new(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidExtension);
    }
    Ok(())
}

#[test]
fn extension_multiple_dots_last_valid() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new("file.backup.monumentum")?;
    let wb = Workbook::<FileStorage>::create_new(temp.path())?;
    wb.close()?;
    Ok(())
}

#[test]
fn open_existing_valid_file_loads_catalog() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let catalog = create_catalog_with_table()?;

    {
        let mut wb = Workbook::<FileStorage>::create_new(temp.path())?;
        *wb.catalog_mut() = catalog.clone();
        wb.save()?;
        wb.close()?;
    }

    let opened = Workbook::<FileStorage>::open(temp.path())?;
    assert_eq!(opened.catalog(), &catalog);
    opened.close()?;
    Ok(())
}

#[test]
fn open_non_existent_creates_wal_and_empty_workbook() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let wb = Workbook::<FileStorage>::open(temp.path())?;
    assert!(wb.catalog().is_empty());
    let wal_path = temp.path().with_extension("wal");
    assert!(wal_path.exists());
    wb.close()?;
    Ok(())
}

#[test]
fn open_directory_returns_db_error() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("dir.monumentum");
    std::fs::create_dir(&dir_path)?;

    let result = Workbook::<FileStorage>::open(&dir_path);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(_)));
    }
    Ok(())
}

#[test]
fn open_invalid_extension_returns_error() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new("txt")?;
    let result = Workbook::<FileStorage>::open(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidExtension);
    }
    Ok(())
}

#[test]
fn create_new_creates_empty_workbook() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let wb = Workbook::<FileStorage>::create_new(temp.path())?;
    assert!(wb.catalog().is_empty());
    wb.close()?;
    Ok(())
}

#[test]
fn create_new_existing_file_returns_error() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    std::fs::write(temp.path(), b"dummy")?;

    let result = Workbook::<FileStorage>::create_new(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::FileExists);
    }
    Ok(())
}

#[test]
fn create_new_existing_directory_returns_file_exists() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    std::fs::create_dir(temp.path())?;

    let result = Workbook::<FileStorage>::create_new(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::FileExists);
    }
    Ok(())
}

#[test]
fn create_new_invalid_extension_returns_error_before_exists_check() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new("txt")?;
    std::fs::write(temp.path(), b"dummy")?;

    let result = Workbook::<FileStorage>::create_new(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidExtension);
    }
    Ok(())
}

#[test]
fn save_persists_catalog() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let catalog = create_catalog_with_table()?;

    {
        let mut wb = Workbook::<FileStorage>::create_new(temp.path())?;
        *wb.catalog_mut() = catalog.clone();
        wb.save()?;
        wb.close()?;
    }

    let opened = Workbook::<FileStorage>::open(temp.path())?;
    assert_eq!(opened.catalog(), &catalog);
    opened.close()?;
    Ok(())
}

#[test]
fn save_multiple_times_no_error() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let catalog = create_catalog_with_table()?;

    {
        let mut wb = Workbook::<FileStorage>::create_new(temp.path())?;
        *wb.catalog_mut() = catalog.clone();
        wb.save()?;
        wb.save()?;
        wb.save()?;
        wb.close()?;
    }

    let opened = Workbook::<FileStorage>::open(temp.path())?;
    assert_eq!(opened.catalog(), &catalog);
    opened.close()?;
    Ok(())
}

#[test]
fn save_as_creates_new_file_and_keeps_original() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new(FILE_EXTENSION)?;
    let catalog = create_catalog_with_table()?;

    let mut wb = Workbook::<FileStorage>::create_new(original.path())?;
    *wb.catalog_mut() = catalog.clone();
    wb.save()?;

    wb.save_as(copy.path())?;
    wb.close()?;

    assert!(copy.path().exists());
    assert!(original.path().exists());

    let opened_orig = Workbook::<FileStorage>::open(original.path())?;
    let opened_copy = Workbook::<FileStorage>::open(copy.path())?;
    assert_eq!(opened_orig.catalog(), &catalog);
    assert_eq!(opened_copy.catalog(), &catalog);
    opened_orig.close()?;
    opened_copy.close()?;
    Ok(())
}

#[test]
fn save_as_existing_file_returns_error() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new(FILE_EXTENSION)?;
    std::fs::write(copy.path(), b"existing")?;

    let mut wb = Workbook::<FileStorage>::create_new(original.path())?;
    let result = wb.save_as(copy.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::FileExists);
    }
    wb.close()?;
    Ok(())
}

#[test]
fn save_as_invalid_extension_returns_error() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new("txt")?;

    let mut wb = Workbook::<FileStorage>::create_new(original.path())?;
    let result = wb.save_as(copy.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidExtension);
    }
    wb.close()?;
    Ok(())
}

#[test]
fn save_as_uppercase_extension_is_accepted() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new("MONUMENTUM")?;

    let mut wb = Workbook::<FileStorage>::create_new(original.path())?;
    wb.save_as(copy.path())?;
    wb.close()?;
    assert!(copy.path().exists());
    Ok(())
}

#[test]
fn save_a_copy_creates_copy_without_changing_original() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new(FILE_EXTENSION)?;

    let mut wb = Workbook::<FileStorage>::create_new(original.path())?;
    wb.save()?;
    wb.save_a_copy(copy.path())?;

    assert!(copy.path().exists());
    assert!(original.path().exists());

    let opened_copy = Workbook::<FileStorage>::open(copy.path())?;
    assert_eq!(opened_copy.catalog(), wb.catalog());
    opened_copy.close()?;

    wb.close()?;
    Ok(())
}

#[test]
fn save_a_copy_existing_file_returns_error() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new(FILE_EXTENSION)?;
    std::fs::write(copy.path(), b"existing")?;

    let wb = Workbook::<FileStorage>::create_new(original.path())?;
    let result = wb.save_a_copy(copy.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::FileExists);
    }
    wb.close()?;
    Ok(())
}

#[test]
fn save_a_copy_invalid_extension_returns_error() -> Result<(), Box<dyn Error>> {
    let original = TestFile::new(FILE_EXTENSION)?;
    let copy = TestFile::new("txt")?;

    let wb = Workbook::<FileStorage>::create_new(original.path())?;
    let result = wb.save_a_copy(copy.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e, WorkbookError::InvalidExtension);
    }
    wb.close()?;
    Ok(())
}

#[test]
fn reload_discards_unsaved_changes() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;
    let catalog = create_catalog_with_table()?;

    let mut wb = Workbook::<FileStorage>::create_new(temp.path())?;
    *wb.catalog_mut() = catalog.clone();
    wb.save()?;

    wb.catalog_mut().drop_table("test_table")?;

    wb.reload()?;
    assert_eq!(wb.catalog(), &catalog);

    wb.close()?;
    Ok(())
}

#[test]
fn close_releases_file_lock() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;

    let wb = Workbook::<FileStorage>::create_new(temp.path())?;
    wb.close()?;

    let wb2 = Workbook::<FileStorage>::open(temp.path())?;
    wb2.close()?;
    Ok(())
}

#[test]
fn in_memory_new_creates_empty_workbook() -> Result<(), Box<dyn Error>> {
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
    assert!(wb.catalog().is_empty());

    wb.save()?;
    wb.reload()?;
    assert!(wb.catalog().is_empty());

    wb.close()?;
    Ok(())
}

#[test]
fn in_memory_load_catalog_preserves_data() -> Result<(), Box<dyn Error>> {
    let catalog = create_catalog_with_table()?;
    let mut wb = Workbook::<InMemoryStorage>::load_in_memory(catalog.clone());
    assert_eq!(wb.catalog(), &catalog);

    wb.save()?;
    wb.reload()?;
    assert_eq!(wb.catalog(), &catalog);

    wb.close()?;
    Ok(())
}

#[test]
fn in_memory_save_and_reload_cycles() -> Result<(), Box<dyn Error>> {
    let mut wb = Workbook::<InMemoryStorage>::new_in_memory();
    let catalog = create_catalog_with_table()?;
    *wb.catalog_mut() = catalog.clone();
    wb.save()?;

    wb.catalog_mut().drop_table("test_table")?;

    wb.reload()?;
    assert_eq!(wb.catalog(), &catalog);

    wb.close()?;
    Ok(())
}

#[test]
fn full_workflow_create_save_open_modify_save() -> Result<(), Box<dyn Error>> {
    let temp = TestFile::new(FILE_EXTENSION)?;

    let mut wb = Workbook::<FileStorage>::create_new(temp.path())?;
    let catalog = create_catalog_with_table()?;
    *wb.catalog_mut() = catalog;
    wb.save()?;
    wb.close()?;

    let mut wb2 = Workbook::<FileStorage>::open(temp.path())?;
    let table = wb2
        .catalog_mut()
        .get_table_mut("test_table")
        .ok_or("table not found")?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    wb2.save()?;
    wb2.close()?;

    let wb3 = Workbook::<FileStorage>::open(temp.path())?;
    assert_eq!(
        wb3.catalog()
            .get_table("test_table")
            .ok_or("table not found")?
            .len(),
        1
    );
    wb3.close()?;

    Ok(())
}

proptest! {
    #[test]
    fn create_new_accepts_any_valid_filename(
        name in "[a-zA-Z0-9_-]{1,20}"
    ) {
        let dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        let path = dir.path().join(format!("{}.{}", name, FILE_EXTENSION));

        let wb = Workbook::<FileStorage>::create_new(&path)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        wb.close().map_err(|e| TestCaseError::fail(e.to_string()))?;

        let wb2 = Workbook::<FileStorage>::open(&path)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        wb2.close().map_err(|e| TestCaseError::fail(e.to_string()))?;
    }

    #[test]
    fn create_new_rejects_invalid_extension(
        name in "[a-zA-Z0-9_-]{1,20}",
        ext in "[a-zA-Z0-9]{1,5}"
    ) {
        prop_assume!(!ext.eq_ignore_ascii_case(FILE_EXTENSION));

        let dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        let path = dir.path().join(format!("{}.{}", name, ext));

        let result = Workbook::<FileStorage>::create_new(&path);
        prop_assert!(result.is_err());
        if let Err(e) = result {
            prop_assert_eq!(e, WorkbookError::InvalidExtension);
        }
    }
}
