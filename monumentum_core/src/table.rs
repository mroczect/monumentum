use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::error::DbError;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    schema: TableSchema,
    read_only: bool,
    data_page_id: Option<u32>,
}

impl Table {
    #[must_use]
    pub const fn new(schema: TableSchema) -> Self {
        Self {
            schema,
            read_only: false,
            data_page_id: None,
        }
    }

    pub fn rename_schema(&mut self, new_name: &str) -> Result<(), DbError> {
        let new_schema = TableSchema::try_new(new_name, self.schema.columns().to_vec())?;
        self.schema = new_schema;
        Ok(())
    }

    #[must_use]
    pub const fn schema(&self) -> &TableSchema {
        &self.schema
    }

    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub const fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
    }

    #[must_use]
    pub const fn data_page_id(&self) -> Option<u32> {
        self.data_page_id
    }

    pub const fn set_data_page_id(&mut self, id: u32) {
        self.data_page_id = Some(id);
    }
}
