use crate::{Workbook, WorkbookError};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;

impl<S: StorageEngine> Workbook<S> {
    pub fn row_count(&self, sheet: &str) -> Result<usize, WorkbookError> {
        let table = self.sheet(sheet)?;
        Ok(table.len())
    }

    pub fn column_count(&self, sheet: &str) -> Result<usize, WorkbookError> {
        let table = self.sheet(sheet)?;
        Ok(table.schema().columns().len())
    }

    pub fn get_cell(&self, sheet: &str, row_idx: usize, col_idx: usize) -> Option<&Value> {
        let table = self.sheet(sheet).ok()?;
        let row = table.get(row_idx)?;
        row.get(col_idx)
    }

    pub fn set_cell(
        &mut self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
        value: Value,
    ) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let row = table
            .get_mut(row_idx)
            .ok_or(WorkbookError::InvalidReference)?;
        let cell = row
            .values_mut()
            .get_mut(col_idx)
            .ok_or(WorkbookError::InvalidReference)?;
        *cell = value;
        Ok(())
    }

    pub fn find_in_sheet(
        &self,
        sheet: &str,
        value: &Value,
    ) -> Result<Vec<(usize, usize)>, WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let mut matches = Vec::new();
        for (row_idx, row) in table.rows().iter().enumerate() {
            for (col_idx, cell) in row.values().iter().enumerate() {
                if cell == value {
                    matches.push((row_idx, col_idx));
                }
            }
        }
        Ok(matches)
    }

    pub fn replace_in_sheet(
        &mut self,
        sheet: &str,
        old_value: &Value,
        new_value: &Value,
    ) -> Result<usize, WorkbookError> {
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let mut count: usize = 0;
        for row in table.rows_mut() {
            for cell in row.values_mut() {
                if cell == old_value {
                    *cell = new_value.clone();
                    count = count.saturating_add(1);
                }
            }
        }
        Ok(count)
    }
}
