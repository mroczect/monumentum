use crate::{Workbook, WorkbookError};
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::ColumnDef;
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;

impl<S: StorageEngine> Workbook<S> {
    pub fn insert_row_at(
        &mut self,
        sheet: &str,
        index: usize,
        values: Vec<Value>,
    ) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let schema_columns_count = table.schema().columns().len();
        if values.len() != schema_columns_count {
            return Err(WorkbookError::Db(
                monumentum_db::error::DbError::invalid_operation("wrong number of values")
                    .to_string(),
            ));
        }

        let row = Row::new(values);
        table.schema().validate_values(row.values())?;

        let mut rows = table.rows().to_vec();
        let insert_idx = index.min(rows.len());
        rows.insert(insert_idx, row);
        table.replace_rows(rows)?;
        Ok(())
    }

    pub fn insert_column(
        &mut self,
        sheet: &str,
        index: usize,
        col_def: &ColumnDef,
    ) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let columns = table.schema().columns().to_vec();
        if index > columns.len() {
            return Err(WorkbookError::InvalidReference);
        }

        let mut new_columns = columns;
        new_columns.insert(index, col_def.to_owned());
        let new_schema = TableSchema::try_new(sheet, new_columns)?;

        let default_value = col_def.default_value().cloned().unwrap_or(Value::Null);
        let mut new_rows = Vec::with_capacity(table.rows().len());
        for old_row in table.rows() {
            let mut values = old_row.values().to_vec();
            values.insert(index, default_value.clone());
            new_rows.push(Row::new(values));
        }

        let mut new_table = Table::new(new_schema);
        for row in new_rows {
            new_table.insert(row)?;
        }

        self.catalog.drop_table(sheet)?;
        self.catalog.create_table(new_table.schema().clone())?;
        let table_mut = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table_mut.replace_rows(new_table.rows().to_vec())?;
        Ok(())
    }

    pub fn delete_column(&mut self, sheet: &str, index: usize) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let columns = table.schema().columns().to_vec();
        if index >= columns.len() {
            return Err(WorkbookError::InvalidReference);
        }
        if columns.len() <= 1 {
            return Err(WorkbookError::Db(
                monumentum_db::error::DbError::invalid_operation("cannot delete the last column")
                    .to_string(),
            ));
        }

        let mut new_columns = columns;
        let _ = new_columns.remove(index);
        let new_schema = TableSchema::try_new(sheet, new_columns)?;

        let mut new_rows = Vec::with_capacity(table.rows().len());
        for old_row in table.rows() {
            let mut values = old_row.values().to_vec();
            let _ = values.remove(index);
            new_rows.push(Row::new(values));
        }

        let mut new_table = Table::new(new_schema);
        for row in new_rows {
            new_table.insert(row)?;
        }

        self.catalog.drop_table(sheet)?;
        self.catalog.create_table(new_table.schema().clone())?;
        let table_mut = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table_mut.replace_rows(new_table.rows().to_vec())?;
        Ok(())
    }
}
