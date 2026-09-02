use crate::core::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum IndexKey {
    Null,
    Integer(i64),
    Float(u64),
    Text(String),
    Blob(Vec<u8>),
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
            Value::Formula(_) => None,
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
