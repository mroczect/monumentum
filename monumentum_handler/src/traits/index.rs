use crate::core::value::Value;

pub trait Index {
    fn insert(&mut self, key: &Value, row_idx: usize);
    fn remove(&mut self, key: &Value, row_idx: usize);
    fn lookup(&self, key: &Value) -> Option<&[usize]>;
}
