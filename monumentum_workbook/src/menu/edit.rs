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
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let mut rows = table.rows().to_vec();
        let row = rows
            .get_mut(row_idx)
            .ok_or(WorkbookError::InvalidReference)?;
        let cell = row
            .values_mut()
            .get_mut(col_idx)
            .ok_or(WorkbookError::InvalidReference)?;
        *cell = value;
        table.replace_rows(rows)?;
        Ok(())
    }

    pub fn replace_in_sheet(
        &mut self,
        sheet: &str,
        old_value: &Value,
        new_value: &Value,
    ) -> Result<usize, WorkbookError> {
        self.ensure_writable(sheet)?;

        let positions = {
            let table = self.catalog.get_table(sheet).ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
            })?;
            let mut pos = Vec::new();
            let col_count = table.schema().columns().len();

            for (row_idx, row) in table.rows().iter().enumerate() {
                for col_idx in 0..col_count {
                    if row.get(col_idx) == Some(old_value) {
                        pos.push((row_idx, col_idx));
                    }
                }
            }
            pos
        };

        let table_mut = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let mut count: usize = 0;
        for (row_idx, col_idx) in positions {
            table_mut.set_cell(row_idx, col_idx, new_value.clone())?;
            count = count.saturating_add(1);
        }

        Ok(count)
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
}
