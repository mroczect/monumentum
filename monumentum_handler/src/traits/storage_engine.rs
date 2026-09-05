use crate::core::row::Row;
use crate::core::schema::table_schema::TableSchema;
use crate::core::value::Value;
use crate::error::DbError;

pub trait StorageEngine {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
    fn insert_row(&mut self, table: &str, row: &Row) -> Result<(), DbError>;
    fn get_row(&self, table: &str, row_idx: usize) -> Result<Option<Row>, DbError>;
    fn set_cell(
        &mut self,
        table: &str,
        row_idx: usize,
        col_idx: usize,
        value: Value,
    ) -> Result<(), DbError>;
    fn replace_rows(&mut self, table: &str, rows: Vec<Row>) -> Result<(), DbError>;
    fn checkpoint(&mut self) -> Result<(), DbError>;
}
