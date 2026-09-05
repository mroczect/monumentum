#![allow(unused_crate_dependencies)]

use monumentum_core::page::{PAGE_SIZE, Page, PageType};
use monumentum_core::pager::Pager;
use monumentum_handler::error::DbError;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_pager_test_{}_{}.db",
        std::process::id(),
        nanos
    ))
}

#[test]
fn test_pager_open_new_file() {
    let path = temp_db_path();
    let pager = Pager::open(&path);
    assert!(pager.is_ok());
    if let Ok(pager) = pager {
        assert_eq!(pager.page_count(), 0);
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_pager_allocate_read_write_page() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(mut pager) = pager_result {
        let page_id_result = pager.allocate_page(PageType::Data);
        assert!(page_id_result.is_ok());
        if let Ok(page_id) = page_id_result {
            assert_eq!(page_id, 0);
            assert_eq!(pager.page_count(), 1);

            let read_result = pager.read_page(page_id);
            assert!(read_result.is_ok());
            if let Ok(mut page) = read_result {
                page.header.cell_count = 7;
                page.data[0] = 0xAA;
                let write_result = pager.write_page(&page);
                assert!(write_result.is_ok());
            }

            let read_again = pager.read_page(page_id);
            assert!(read_again.is_ok());
            if let Ok(page) = read_again {
                assert_eq!(page.header.cell_count, 7);
                assert_eq!(page.data[0], 0xAA);
            }
        }
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_pager_free_page() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(mut pager) = pager_result {
        let page_id_result = pager.allocate_page(PageType::Data);
        assert!(page_id_result.is_ok());
        if let Ok(page_id) = page_id_result {
            let free_result = pager.free_page(page_id);
            assert!(free_result.is_ok());
            let read = pager.read_page(page_id);
            assert!(read.is_ok());
            if let Ok(page) = read {
                assert_eq!(page.header.page_type, PageType::Freelist);
            }
        }
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_pager_rejects_invalid_page_id() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(mut pager) = pager_result {
        let read = pager.read_page(0);
        assert!(read.is_err());
        let write = pager.write_page(&Page::new(0, PageType::Data));
        assert!(write.is_err());
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_pager_detects_corrupt_size() {
    let path = temp_db_path();
    {
        let mut file = fs::File::create(&path).unwrap_or_else(|_| unreachable!());
        file.write_all(&[0u8; PAGE_SIZE - 1])
            .unwrap_or_else(|_| unreachable!());
    }
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_err());
    if let Err(e) = pager_result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_pager_detects_checksum_mismatch() {
    use std::io::{Read, Seek, Write};

    let path = temp_db_path();
    {
        let mut pager = Pager::open(&path).unwrap_or_else(|_| unreachable!());
        let _ = pager
            .allocate_page(PageType::Data)
            .unwrap_or_else(|_| unreachable!());
        let mut page = pager.read_page(0).unwrap_or_else(|_| unreachable!());
        page.data[0] = 0xAB;
        pager.write_page(&page).unwrap_or_else(|_| unreachable!());
    }

    {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap_or_else(|_| unreachable!());
        let mut buf = [0u8; 1];
        let _ = file
            .seek(std::io::SeekFrom::Start(100))
            .unwrap_or_else(|_| unreachable!());
        file.read_exact(&mut buf).unwrap_or_else(|_| unreachable!());
        buf[0] ^= 0xFF;
        let _ = file
            .seek(std::io::SeekFrom::Start(100))
            .unwrap_or_else(|_| unreachable!());
        file.write_all(&buf).unwrap_or_else(|_| unreachable!());
    }

    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(mut pager) = pager_result {
        let read_result = pager.read_page(0);
        assert!(read_result.is_err());
        if let Err(e) = read_result {
            assert!(matches!(e, DbError::Corruption(_)));
        }
    }

    let _ = fs::remove_file(&path);
}
