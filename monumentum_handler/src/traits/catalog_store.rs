use crate::core::schema::table_schema::TableSchema;
use crate::error::DbError;

pub trait CatalogStore {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError>;
    fn drop_table(&mut self, name: &str) -> Result<(), DbError>;
    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError>;
}
