use crate::core::schema::table_schema::TableSchema;
use crate::error::DbError;

pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<(), DbError>;
    fn save_catalog(&mut self) -> Result<(), DbError>;
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
}
