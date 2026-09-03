use crate::core::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum IndexKey {
    Null,
    Integer(i64),
    Float(u64),
    Text(String),
    Blob(Vec<u8>),
    Formula(String),
}

impl IndexKey {
    pub(crate) fn from_value(v: &Value) -> Option<Self> {
        match v {
            Value::Null => Some(Self::Null),
            Value::Integer(i) => Some(Self::Integer(i.as_i64())),
            Value::Float(f) => {
                let bits = f.as_f64().to_bits();
                let bits = if f.as_f64() == 0.0 {
                    0.0f64.to_bits()
                } else {
                    bits
                };
                Some(Self::Float(bits))
            }
            Value::Text(t) => Some(Self::Text(t.as_str().to_string())),
            Value::Blob(b) => Some(Self::Blob(b.as_slice().to_vec())),
            Value::Boolean(_) => None,
            Value::Formula(s) => Some(Self::Formula(s.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct HashIndex {
    map: HashMap<IndexKey, Vec<usize>>,
}

impl HashIndex {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, key: IndexKey, row_idx: usize) {
        self.map.entry(key).or_default().push(row_idx);
    }

    pub(crate) fn contains(&self, key: &IndexKey) -> bool {
        self.map.contains_key(key)
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }

    pub(crate) fn get_indices(&self, key: &IndexKey) -> Option<&[usize]> {
        self.map.get(key).map(Vec::as_slice)
    }

    pub(crate) fn remove(&mut self, key: &IndexKey, row_idx: usize) {
        if let Some(indices) = self.map.get_mut(key) {
            indices.retain(|&x| x != row_idx);
            if indices.is_empty() {
                self.map.remove(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HashIndex, IndexKey};
    use crate::core::value::Value;
    use std::collections::HashMap;

    #[test]
    fn index_key_from_null_value() {
        let v = Value::Null;
        assert_eq!(IndexKey::from_value(&v), Some(IndexKey::Null));
    }

    #[test]
    fn index_key_from_integer_value() {
        let vals = [
            (0_i64, IndexKey::Integer(0)),
            (1_i64, IndexKey::Integer(1)),
            (-1_i64, IndexKey::Integer(-1)),
            (i64::MAX, IndexKey::Integer(i64::MAX)),
            (i64::MIN, IndexKey::Integer(i64::MIN)),
        ];
        for (i, expected) in vals {
            let v = Value::from(i);
            assert_eq!(IndexKey::from_value(&v), Some(expected));
        }
    }

    #[test]
    fn index_key_from_float_value_normalizes_zero() {
        let v = Value::try_from(0.0_f64).unwrap();
        assert_eq!(
            IndexKey::from_value(&v),
            Some(IndexKey::Float(0.0_f64.to_bits()))
        );

        let neg_zero = Value::try_from(-0.0_f64).unwrap();
        assert_eq!(
            IndexKey::from_value(&neg_zero),
            Some(IndexKey::Float(0.0_f64.to_bits()))
        );
    }

    #[test]
    fn index_key_from_positive_float() {
        let v = Value::try_from(2.5_f64).unwrap();
        let expected_bits = 2.5_f64.to_bits();
        assert_eq!(
            IndexKey::from_value(&v),
            Some(IndexKey::Float(expected_bits))
        );
    }

    #[test]
    fn index_key_from_text_value() {
        let cases = [
            "",
            "hello",
            "Hello",
            "héllo",
            "日本語",
            "long string with spaces",
        ];
        for s in cases {
            let v = Value::from(s);
            assert_eq!(
                IndexKey::from_value(&v),
                Some(IndexKey::Text(s.to_string()))
            );
        }
    }

    #[test]
    fn index_key_from_blob_value() {
        let cases: Vec<Vec<u8>> = vec![
            vec![],
            vec![0],
            vec![1, 2, 3],
            vec![b'a', b'b', b'c', 0, 255],
        ];
        for bytes in cases {
            let v = Value::from(bytes.clone());
            assert_eq!(IndexKey::from_value(&v), Some(IndexKey::Blob(bytes)));
        }
    }

    #[test]
    fn index_key_from_boolean_returns_none() {
        let v = Value::Boolean(true);
        assert_eq!(IndexKey::from_value(&v), None);
        let v2 = Value::Boolean(false);
        assert_eq!(IndexKey::from_value(&v2), None);
    }

    #[test]
    fn index_key_from_formula_value() {
        let v = Value::Formula("SUM(A1:A10)".to_string());
        assert_eq!(
            IndexKey::from_value(&v),
            Some(IndexKey::Formula("SUM(A1:A10)".to_string()))
        );
        let empty = Value::Formula(String::new());
        assert_eq!(
            IndexKey::from_value(&empty),
            Some(IndexKey::Formula(String::new()))
        );
    }

    #[test]
    fn hash_index_insert_and_contains() {
        let mut idx = HashIndex::new();
        let key = IndexKey::Integer(10);
        idx.insert(key.clone(), 3);
        assert!(idx.contains(&key));
        assert_eq!(idx.get_indices(&key), Some(&[3][..]));
    }

    #[test]
    fn hash_index_multiple_rows_same_key() {
        let mut idx = HashIndex::new();
        let key = IndexKey::Text("dup".to_string());
        idx.insert(key.clone(), 0);
        idx.insert(key.clone(), 1);
        idx.insert(key.clone(), 2);
        assert_eq!(idx.get_indices(&key), Some(&[0, 1, 2][..]));
    }

    #[test]
    fn hash_index_clear() {
        let mut idx = HashIndex::new();
        idx.insert(IndexKey::Null, 0);
        idx.insert(IndexKey::Integer(1), 1);
        idx.clear();
        assert!(!idx.contains(&IndexKey::Null));
        assert!(!idx.contains(&IndexKey::Integer(1)));
        assert!(idx.get_indices(&IndexKey::Null).is_none());
    }

    #[test]
    fn hash_index_remove_existing_row() {
        let mut idx = HashIndex::new();
        let key = IndexKey::Integer(5);
        idx.insert(key.clone(), 10);
        idx.insert(key.clone(), 20);
        idx.insert(key.clone(), 30);
        idx.remove(&key, 20);
        assert_eq!(idx.get_indices(&key), Some(&[10, 30][..]));
    }

    #[test]
    fn hash_index_remove_last_row_deletes_key() {
        let mut idx = HashIndex::new();
        let key = IndexKey::Integer(5);
        idx.insert(key.clone(), 10);
        idx.remove(&key, 10);
        assert!(!idx.contains(&key));
        assert!(idx.get_indices(&key).is_none());
    }

    #[test]
    fn hash_index_remove_non_existent_row_keeps_others() {
        let mut idx = HashIndex::new();
        let key = IndexKey::Integer(5);
        idx.insert(key.clone(), 10);
        idx.remove(&key, 999);
        assert_eq!(idx.get_indices(&key), Some(&[10][..]));
    }

    #[test]
    fn hash_index_with_different_keys() {
        let mut idx = HashIndex::new();
        idx.insert(IndexKey::Integer(1), 0);
        idx.insert(IndexKey::Text("a".to_string()), 1);
        idx.insert(IndexKey::Float(1.0_f64.to_bits()), 2);
        idx.insert(IndexKey::Blob(vec![1, 2]), 3);
        idx.insert(IndexKey::Formula("=A1".to_string()), 4);
        idx.insert(IndexKey::Null, 5);

        assert_eq!(idx.get_indices(&IndexKey::Integer(1)), Some(&[0][..]));
        assert_eq!(
            idx.get_indices(&IndexKey::Text("a".to_string())),
            Some(&[1][..])
        );
        assert_eq!(
            idx.get_indices(&IndexKey::Float(1.0_f64.to_bits())),
            Some(&[2][..])
        );
        assert_eq!(idx.get_indices(&IndexKey::Blob(vec![1, 2])), Some(&[3][..]));
        assert_eq!(
            idx.get_indices(&IndexKey::Formula("=A1".to_string())),
            Some(&[4][..])
        );
        assert_eq!(idx.get_indices(&IndexKey::Null), Some(&[5][..]));
    }

    #[test]
    fn hash_index_insert_remove_random_order_maintains_consistency() {
        let mut idx = HashIndex::new();
        let mut reference: HashMap<IndexKey, Vec<usize>> = HashMap::new();

        for i in 0usize..100 {
            let key = IndexKey::Integer((i % 10) as i64);
            idx.insert(key.clone(), i);
            reference.entry(key).or_default().push(i);
        }

        for (key, rows) in &reference {
            assert_eq!(idx.get_indices(key), Some(rows.as_slice()));
        }

        for i in (0usize..100).step_by(7) {
            let key = IndexKey::Integer((i % 10) as i64);
            idx.remove(&key, i);
            if let Some(rows) = reference.get_mut(&key) {
                rows.retain(|&x| x != i);
                if rows.is_empty() {
                    reference.remove(&key);
                }
            }
        }

        for (key, rows) in &reference {
            assert_eq!(idx.get_indices(key), Some(rows.as_slice()));
        }
    }
}
