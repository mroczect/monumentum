#![allow(unused_crate_dependencies)]

use monumentum_core::index::btree_index::BTreeIndex;
use monumentum_core::index::hash_index::HashIndex;
use monumentum_core::index::key::IndexKey;
use monumentum_handler::core::value::Value;
use monumentum_handler::traits::Index;

const fn int_key(i: i64) -> IndexKey {
    IndexKey::Integer(i)
}

fn int_value(i: i64) -> Value {
    Value::from(i)
}

#[test]
fn test_btree_index_direct_methods() {
    let mut idx = BTreeIndex::new();
    idx.insert(int_key(1), 10);
    idx.insert(int_key(2), 20);
    idx.insert(int_key(1), 30);

    let indices = idx.get_indices(&int_key(1));
    assert!(indices.is_some());
    if let Some(slice) = indices {
        assert_eq!(slice.len(), 2);
        assert!(slice.contains(&10));
        assert!(slice.contains(&30));
    }

    idx.remove(&int_key(1), 10);
    let indices_after = idx.get_indices(&int_key(1));
    assert!(indices_after.is_some());
    if let Some(slice) = indices_after {
        assert_eq!(slice.len(), 1);
        assert_eq!(slice.first().copied(), Some(30));
    }

    idx.remove(&int_key(1), 30);
    assert!(idx.get_indices(&int_key(1)).is_none());
}

#[test]
fn test_btree_index_trait_impl() {
    let mut idx = BTreeIndex::new();
    let key = int_value(5);
    Index::insert(&mut idx, &key, 100);
    let lookup = Index::lookup(&idx, &key);
    assert!(lookup.is_some());
    if let Some(slice) = lookup {
        assert_eq!(slice.len(), 1);
        assert_eq!(slice.first().copied(), Some(100));
    }
    Index::remove(&mut idx, &key, 100);
    assert!(Index::lookup(&idx, &key).is_none());
}

#[test]
fn test_hash_index_basic() {
    let mut idx = HashIndex::new();
    idx.insert(int_key(1), 1);
    idx.insert(int_key(2), 2);
    assert!(idx.contains(&int_key(1)));
    assert!(idx.contains(&int_key(2)));
    assert!(!idx.contains(&int_key(3)));

    let indices = idx.get_indices(&int_key(1));
    assert!(indices.is_some());
    if let Some(slice) = indices {
        assert_eq!(slice, &[1]);
    }

    idx.remove(&int_key(1), 1);
    assert!(!idx.contains(&int_key(1)));
}

#[test]
fn test_index_key_roundtrip() {
    let keys = vec![
        IndexKey::Integer(-42),
        IndexKey::Float(1.5_f64.to_bits()),
        IndexKey::Text("hello".to_string()),
        IndexKey::Blob(vec![1, 2, 3]),
        IndexKey::Boolean(true),
    ];

    for key in keys {
        let bytes = key.to_bytes();
        assert!(bytes.is_ok());
        if let Ok(bytes) = bytes {
            let decoded = IndexKey::from_bytes(&bytes);
            assert!(decoded.is_ok());
            if let Ok(decoded_key) = decoded {
                assert_eq!(key, decoded_key);
            }
        }
    }
}

#[test]
fn test_index_key_from_value() {
    let vals = vec![
        Value::from(42i64),
        Value::try_from(2.5).unwrap_or(Value::Null),
        Value::from(
            monumentum_handler::types::Text::try_new("text".to_string())
                .unwrap_or_else(|_| unreachable!()),
        ),
        Value::from(
            monumentum_handler::types::Blob::try_new(vec![7]).unwrap_or_else(|_| unreachable!()),
        ),
        Value::from(true),
    ];

    for val in vals {
        let key = IndexKey::from_value(&val);
        assert!(key.is_some());
        let key = key.unwrap_or_else(|| unreachable!());
        let bytes = key.to_bytes();
        assert!(bytes.is_ok());
        if let Ok(bytes) = bytes {
            let decoded = IndexKey::from_bytes(&bytes);
            assert!(decoded.is_ok());
            if let Ok(decoded_key) = decoded {
                assert_eq!(key, decoded_key);
            }
        }
    }
}
