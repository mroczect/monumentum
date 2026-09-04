use super::key::IndexKey;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HashIndex {
    map: HashMap<IndexKey, Vec<usize>>,
}

impl HashIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: IndexKey, row_idx: usize) {
        let indices = self.map.entry(key).or_default();
        if !indices.contains(&row_idx) {
            indices.push(row_idx);
        }
    }

    #[must_use]
    pub fn contains(&self, key: &IndexKey) -> bool {
        self.map.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn get_indices(&self, key: &IndexKey) -> Option<&[usize]> {
        self.map.get(key).map(Vec::as_slice)
    }

    pub fn remove(&mut self, key: &IndexKey, row_idx: usize) {
        if let Some(indices) = self.map.get_mut(key) {
            indices.retain(|&x| x != row_idx);
            if indices.is_empty() {
                let _ = self.map.remove(key);
            }
        }
    }
}
