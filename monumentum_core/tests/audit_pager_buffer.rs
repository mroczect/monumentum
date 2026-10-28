#![allow(unused_crate_dependencies)]

//! Pager & BufferPool audit.

use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::page::{PAGE_SIZE, PageType};
use monumentum_core::pager::Pager;
use monumentum_handler::error::DbError;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_audit_pager_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

#[test]
fn test_pager_allocate_100() -> Result<(), DbError> {
    let path = temp_db_path("alloc");
    let mut p = Pager::open(&path)?;
    for i in 0..100u32 {
        assert_eq!(p.allocate_page(PageType::Data)?, i);
    }
    assert_eq!(p.page_count(), 100);
    drop(p);
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_pager_write_read_roundtrip() -> Result<(), DbError> {
    let path = temp_db_path("rw");
    let mut p = Pager::open(&path)?;
    let id = p.allocate_page(PageType::Data)?;
    let mut page = p.read_page(id)?;
    for b in &mut page.data[..64] {
        *b = 0xAB;
    }
    p.write_page(&page)?;
    let read = p.read_page(id)?;
    for b in &read.data[..64] {
        assert_eq!(*b, 0xAB);
    }
    drop(p);
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_pager_detects_bitflip() -> Result<(), DbError> {
    let path = temp_db_path("flip");
    {
        let mut p = Pager::open(&path)?;
        let id = p.allocate_page(PageType::Data)?;
        let mut page = p.read_page(id)?;
        page.data[0] = 0x42;
        p.write_page(&page)?;
    }
    {
        let mut f = fs::OpenOptions::new().read(true).write(true).open(&path)?;
        let _ = f.seek(SeekFrom::Start(200))?;
        let mut b = [0u8; 1];
        f.read_exact(&mut b)?;
        b[0] ^= 0xFF;
        let _ = f.seek(SeekFrom::Start(200))?;
        f.write_all(&b)?;
    }
    let mut p = Pager::open(&path)?;
    let res = p.read_page(0);
    drop(p);
    let _ = fs::remove_file(&path);
    assert!(res.is_err(), "checksum harus mendeteksi flip");
    Ok(())
}

#[test]
fn test_pager_rejects_partial_file() {
    let path = temp_db_path("partial");
    {
        let mut f = fs::File::create(&path).unwrap_or_else(|_| unreachable!());
        f.write_all(&[0u8; PAGE_SIZE - 1])
            .unwrap_or_else(|_| unreachable!());
    }
    let res = Pager::open(&path);
    let _ = fs::remove_file(&path);
    assert!(res.is_err());
}

#[test]
fn test_buffer_pool_evicts_clean_pages() -> Result<(), DbError> {
    let path = temp_db_path("evict");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 2)?;
    let _a = pool.allocate_page(PageType::Data)?;
    let _b = pool.allocate_page(PageType::Data)?;
    let _c = pool.allocate_page(PageType::Data)?;
    drop(pool);
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_buffer_pool_pinned_page_blocks_eviction() -> Result<(), DbError> {
    let path = temp_db_path("pin");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 1)?;
    let a = pool.allocate_page(PageType::Data)?;
    let _ = pool.get_page(a)?;
    let b = pool.allocate_page(PageType::Data);
    assert!(b.is_err(), "harus gagal evict pinned page");
    pool.unpin_page(a, false)?;
    drop(pool);
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_buffer_pool_dirty_cleared_after_flush() -> Result<(), DbError> {
    let path = temp_db_path("dirty");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 4)?;
    let a = pool.allocate_page(PageType::Data)?;
    let _ = pool.get_page(a)?;
    pool.unpin_page(a, true)?;
    assert!(pool.dirty_page_ids().contains(&a));
    pool.flush_page(a)?;
    assert!(!pool.dirty_page_ids().contains(&a));
    drop(pool);
    let _ = fs::remove_file(&path);
    Ok(())
}
