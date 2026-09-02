use crate::{Workbook, WorkbookError};
use monumentum_db::core::row::Row;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;

impl<S: StorageEngine> Workbook<S> {
    pub fn sort_sheet(
        &mut self,
        sheet: &str,
        col_idx: usize,
        ascending: bool,
    ) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let column_count = table.schema().columns().len();
        if col_idx >= column_count {
            return Err(WorkbookError::InvalidReference);
        }

        let mut rows = table.rows().to_vec();
        rows.sort_by(|a, b| {
            let av = a.get(col_idx).unwrap_or(&Value::Null);
            let bv = b.get(col_idx).unwrap_or(&Value::Null);
            if ascending {
                av.partial_cmp(bv).unwrap_or(core::cmp::Ordering::Equal)
            } else {
                bv.partial_cmp(av).unwrap_or(core::cmp::Ordering::Equal)
            }
        });

        let table_mut = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table_mut.replace_rows(rows)?;
        Ok(())
    }

    pub fn filter_sheet(
        &self,
        sheet: &str,
        col_idx: usize,
        value: &Value,
    ) -> Result<Vec<Row>, WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let column_count = table.schema().columns().len();
        if col_idx >= column_count {
            return Err(WorkbookError::InvalidReference);
        }

        Ok(table
            .rows()
            .iter()
            .filter(|row| row.get(col_idx) == Some(value))
            .cloned()
            .collect())
    }

    pub fn distinct_values(
        &self,
        sheet: &str,
        col_idx: usize,
    ) -> Result<Vec<Value>, WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        let column_count = table.schema().columns().len();
        if col_idx >= column_count {
            return Err(WorkbookError::InvalidReference);
        }

        let mut values: Vec<Value> = table
            .rows()
            .iter()
            .filter_map(|row| row.get(col_idx).cloned())
            .collect();

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        values.dedup();
        Ok(values)
    }
}
