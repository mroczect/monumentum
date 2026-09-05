#![allow(unused_crate_dependencies)]

use monumentum_core::buffer_pool::BufferPool;
use monumentum_core::pager::Pager;
use monumentum_core::table_storage::TableStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_tablestorage_{}_{}.db",
        std::process::id(),
        nanos
    ))
}

#[test]
fn test_table_storage_insert_and_get() {
    let path = temp_db_path();
    let pager_result = Pager::open(&path);
    assert!(pager_result.is_ok());
    if let Ok(pager) = pager_result {
        let buffer_pool_result = BufferPool::new(pager, 10);
        assert!(buffer_pool_result.is_ok());
        if let Ok(mut buffer_pool) = buffer_pool_result {
            let table_result = TableStorage::new(&mut buffer_pool);
            assert!(table_result.is_ok());
            if let Ok(mut table) = table_result {
                let row1 = Row::new(vec![Value::from(1i64), Value::Null]);
                let row2 = Row::new(vec![Value::from(2i64), Value::from(true)]);

                let insert1 = table.insert_row(&mut buffer_pool, &row1);
                assert!(insert1.is_ok());
                let insert2 = table.insert_row(&mut buffer_pool, &row2);
                assert!(insert2.is_ok());

                let get0 = table.get_row(&mut buffer_pool, 0);
                assert!(get0.is_ok());
                if let Ok(Some(row)) = get0 {
                    assert_eq!(row, row1);
                } else {
                    unreachable!("row 0 should exist");
                }

                let get1 = table.get_row(&mut buffer_pool, 1);
                assert!(get1.is_ok());
                if let Ok(Some(row)) = get1 {
                    assert_eq!(row, row2);
                } else {
                    unreachable!("row 1 should exist");
                }

                let get2 = table.get_row(&mut buffer_pool, 2);
                assert!(get2.is_ok());
                assert_eq!(get2, Ok(None));
            }
        }
    }
    let _ = fs::remove_file(&path);
}
