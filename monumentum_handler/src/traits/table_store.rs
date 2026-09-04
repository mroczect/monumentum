use crate::core::row::Row;
use crate::core::value::Value;
use crate::error::DbError;

pub trait TableStore {
    fn insert(&mut self, row: &Row) -> Result<(), DbError>;
    fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError>;
    fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError>;
}
