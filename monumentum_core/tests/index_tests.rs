use monumentum_core::index::{HashIndex, IndexKey};
use monumentum_handler::Value;

#[test]
fn index_key_from_value_cases() {
    assert_eq!(IndexKey::from_value(&Value::Null), None);
    assert_eq!(
        IndexKey::from_value(&Value::from(42_i64)),
        Some(IndexKey::Integer(42))
    );
    assert_eq!(
        IndexKey::from_value(&Value::from(true)),
        Some(IndexKey::Boolean(true))
    );
    if let Some(v) = Value::try_from(-0.0_f64).ok()
        && let Some(key) = IndexKey::from_value(&v)
    {
        assert!(matches!(key, IndexKey::Float(bits) if bits == 0.0_f64.to_bits()));
    }
}
#[test]
fn hash_index_basic_operations() {
    let mut idx = HashIndex::new();
    let key = IndexKey::Integer(10);
    idx.insert(key.clone(), 5);
    assert!(idx.contains(&key));
    assert_eq!(idx.get_indices(&key), Some(&[5][..]));
    idx.remove(&key, 5);
    assert!(!idx.contains(&key));
}

#[test]
fn hash_index_multiple_rows() {
    let mut idx = HashIndex::new();
    let key = IndexKey::Text("dup".to_string());
    idx.insert(key.clone(), 0);
    idx.insert(key.clone(), 1);
    idx.insert(key.clone(), 2);
    assert_eq!(idx.get_indices(&key), Some(&[0, 1, 2][..]));
    idx.remove(&key, 1);
    assert_eq!(idx.get_indices(&key), Some(&[0, 2][..]));
}

#[test]
fn hash_index_clear() {
    let mut idx = HashIndex::new();
    idx.insert(IndexKey::Integer(1), 0);
    idx.insert(IndexKey::Integer(2), 1);
    idx.clear();
    assert!(!idx.contains(&IndexKey::Integer(1)));
    assert!(!idx.contains(&IndexKey::Integer(2)));
}
