#![allow(unused_crate_dependencies)]

//! Deep B-tree audit.
//!   [PASS]     — baseline
//!   [FAIL-C1]  — memicu bug split internal-node

use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::index::btree::BTreeOnDisk;
use monumentum_core::index::key::IndexKey;
use monumentum_core::pager::Pager;
use monumentum_handler::error::DbError;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_audit_btree_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

fn to_u64(i: i64) -> u64 {
    u64::try_from(i).unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_btree_small_sequential_baseline() -> Result<(), DbError> {
    let path = temp_db_path("small");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 32)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    for i in 0..50i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }
    for i in 0..50i64 {
        assert_eq!(
            BTreeOnDisk::lookup_static(&mut pool, root, &IndexKey::Integer(i))?,
            Some(to_u64(i)),
            "key {i} missing"
        );
    }
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_btree_duplicate_key_rejected() -> Result<(), DbError> {
    let path = temp_db_path("dup");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 16)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(1), 100)?;
    let dup = BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(1), 200);
    assert!(dup.is_err(), "duplicate key harus ditolak");

    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_btree_medium_500_no_internal_split() -> Result<(), DbError> {
    let path = temp_db_path("med");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 64)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();
    for i in 0..500i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }
    for i in 0..500i64 {
        assert_eq!(
            BTreeOnDisk::lookup_static(&mut pool, root, &IndexKey::Integer(i))?,
            Some(to_u64(i)),
            "key {i} missing"
        );
    }
    let _ = fs::remove_file(&path);
    Ok(())
}

#[test]
fn test_btree_sequential_15000_internal_split() -> Result<(), DbError> {
    let path = temp_db_path("seq15k");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 1024)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    let n: i64 = 15_000;
    for i in 0..n {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }

    let mut missing = Vec::new();
    for i in 0..n {
        if BTreeOnDisk::lookup_static(&mut pool, root, &IndexKey::Integer(i))?.is_none() {
            missing.push(i);
        }
    }
    let _ = fs::remove_file(&path);
    let sample: Vec<_> = missing.iter().take(20).collect();
    assert!(
        missing.is_empty(),
        "C1 CONFIRMED: {} keys hilang pasca split internal. Sample: {:?}",
        missing.len(),
        sample
    );
    Ok(())
}

#[test]
fn test_btree_reverse_15000_internal_split() -> Result<(), DbError> {
    let path = temp_db_path("rev15k");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 1024)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    let n: i64 = 15_000;
    for i in (0..n).rev() {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }

    let mut missing = Vec::new();
    for i in 0..n {
        if BTreeOnDisk::lookup_static(&mut pool, root, &IndexKey::Integer(i))?.is_none() {
            missing.push(i);
        }
    }
    let _ = fs::remove_file(&path);
    let sample: Vec<_> = missing.iter().take(20).collect();
    assert!(
        missing.is_empty(),
        "C1 CONFIRMED (reverse): {} keys hilang. Sample: {:?}",
        missing.len(),
        sample
    );
    Ok(())
}

#[test]
fn test_btree_range_scan_after_many_inserts() -> Result<(), DbError> {
    let path = temp_db_path("range");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 128)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    for i in 0..1000i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }

    let mut result = Vec::new();
    BTreeOnDisk::range_scan_static(
        &mut pool,
        root,
        &IndexKey::Integer(100),
        &IndexKey::Integer(200),
        &mut result,
    )?;
    let expected: Vec<(IndexKey, u64)> = (100..200)
        .map(|i| (IndexKey::Integer(i), to_u64(i)))
        .collect();
    let _ = fs::remove_file(&path);
    assert_eq!(
        result,
        expected,
        "range scan {} item, harusnya {}",
        result.len(),
        expected.len()
    );
    Ok(())
}

#[test]
fn test_btree_delete_all_sequential() -> Result<(), DbError> {
    let path = temp_db_path("del");
    let pager = Pager::open(&path)?;
    let mut pool = BufferPool::new(pager, 64)?;
    let btree = BTreeOnDisk::create(&mut pool)?;
    let mut root = btree.root_page_id();

    for i in 0..200i64 {
        BTreeOnDisk::insert_static(&mut pool, &mut root, IndexKey::Integer(i), to_u64(i))?;
    }
    for i in 0..200i64 {
        let d = BTreeOnDisk::delete_static(&mut pool, root, &IndexKey::Integer(i))?;
        assert_eq!(d, Some(to_u64(i)));
    }
    for i in 0..200i64 {
        assert!(BTreeOnDisk::lookup_static(&mut pool, root, &IndexKey::Integer(i))?.is_none());
    }
    let _ = fs::remove_file(&path);
    Ok(())
}
