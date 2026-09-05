#![allow(unused_crate_dependencies)]

use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::page::PageType;
use monumentum_core::pager::Pager;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_buff_test_{}_{}.db",
        std::process::id(),
        nanos
    ))
}

#[test]
fn test_buffer_pool_new_rejects_zero_capacity() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(pager) = pager_result {
        let result = BufferPool::new(pager, 0);
        assert!(result.is_err());
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_buffer_pool_allocate_and_get_page() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(pager) = pager_result {
        let pool_result = BufferPool::new(pager, 2);
        assert!(pool_result.is_ok());
        if let Ok(mut pool) = pool_result {
            let page_id_result = pool.allocate_page(PageType::Data);
            assert!(page_id_result.is_ok());
            if let Ok(page_id) = page_id_result {
                let page_ref = pool.get_page(page_id);
                assert!(page_ref.is_ok());
                if let Ok(page) = page_ref {
                    page.header.cell_count = 42;
                }
                let unpin_result = pool.unpin_page(page_id, true);
                assert!(unpin_result.is_ok());
                let flush_result = pool.flush_page(page_id);
                assert!(flush_result.is_ok());

                let page_again_ref = pool.get_page(page_id);
                assert!(page_again_ref.is_ok());
                if let Ok(page) = page_again_ref {
                    assert_eq!(page.header.cell_count, 42);
                }
                let unpin2 = pool.unpin_page(page_id, false);
                assert!(unpin2.is_ok());
            }
        }
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_buffer_pool_eviction() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(pager) = pager_result {
        let pool_result = BufferPool::new(pager, 1);
        assert!(pool_result.is_ok());
        if let Ok(mut pool) = pool_result {
            let first = pool.allocate_page(PageType::Data);
            let second = pool.allocate_page(PageType::Data);
            assert!(first.is_ok());
            assert!(second.is_ok());
            if let (Ok(first_id), Ok(second_id)) = (first, second) {
                let first_page = pool.get_page(first_id);
                assert!(first_page.is_ok());
                let unpin = pool.unpin_page(first_id, false);
                assert!(unpin.is_ok());

                let second_page = pool.get_page(second_id);
                assert!(second_page.is_ok());
                let unpin2 = pool.unpin_page(second_id, false);
                assert!(unpin2.is_ok());
            }
        }
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn test_buffer_pool_flush_all() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(pager) = pager_result {
        let pool_result = BufferPool::new(pager, 4);
        assert!(pool_result.is_ok());
        if let Ok(mut pool) = pool_result {
            let page_id_result = pool.allocate_page(PageType::Data);
            assert!(page_id_result.is_ok());
            if let Ok(page_id) = page_id_result {
                let page_ref = pool.get_page(page_id);
                if let Ok(page) = page_ref {
                    page.data[0] = 0xAB;
                }
                let _ = pool.unpin_page(page_id, true);
                let flush_all = pool.flush_all();
                assert!(flush_all.is_ok());
            }
        }
    }
    let _ = fs::remove_file(&path);
}
