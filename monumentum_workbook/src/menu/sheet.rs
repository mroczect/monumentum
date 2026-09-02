use crate::{Workbook, WorkbookError};
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::ColumnDef;
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;

impl<S: StorageEngine> Workbook<S> {
    pub fn sheet_names(&self) -> Vec<String> {
        self.catalog
            .tables()
            .map(|(name, _)| name.to_string())
            .collect()
    }

    pub fn create_sheet(
        &mut self,
        name: &str,
        columns: Vec<ColumnDef>,
    ) -> Result<(), WorkbookError> {
        let schema = TableSchema::try_new(name, columns)?;
        self.catalog.create_table(schema)?;
        Ok(())
    }

    pub fn drop_sheet(&mut self, name: &str) -> Result<(), WorkbookError> {
        self.ensure_writable(name)?;
        self.catalog.drop_table(name)?;
        Ok(())
    }

    pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), WorkbookError> {
        self.ensure_writable(old_name)?;
        if self.catalog.get_table(new_name).is_some() {
            return Err(WorkbookError::FileExists);
        }

        let table = self.catalog.get_table(old_name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(old_name).to_string())
        })?;

        let schema = table.schema().clone();
        let new_schema = TableSchema::try_new(new_name, schema.columns().to_vec())?;
        let rows: Vec<Row> = table.rows().to_vec();

        let mut new_table = Table::new(new_schema);
        for row in rows {
            new_table.insert(row)?;
        }

        self.catalog.drop_table(old_name)?;
        self.catalog.create_table(new_table.schema().clone())?;
        self.catalog.replace_table(new_name, new_table)?;
        Ok(())
    }

    pub fn insert_row(&mut self, sheet: &str, values: Vec<Value>) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let row = Row::new(values);
        table.insert(row)?;
        Ok(())
    }

    pub fn delete_row(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let mut rows = table.rows().to_vec();
        if index >= rows.len() {
            return Err(WorkbookError::InvalidReference);
        }
        let _ = rows.remove(index);
        table.replace_rows(rows)?;
        Ok(())
    }

    pub fn clear_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table.replace_rows(Vec::new())?;
        Ok(())
    }
}
