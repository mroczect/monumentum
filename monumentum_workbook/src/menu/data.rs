use crate::{Workbook, WorkbookError};
use core::cmp::Ordering;
use monumentum_db::core::row::Row;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;

fn compare_values(a: &Value, b: &Value) -> Ordering {
    use Value::{Blob, Boolean, Float, Formula, Integer, Null, Text};
    let rank = |v: &Value| match v {
        Null => 0,
        Integer(_) => 1,
        Float(_) => 2,
        Text(_) => 3,
        Blob(_) => 4,
        Boolean(_) => 5,
        Formula(_) => 6,
        _ => 7,
    };
    let ra = rank(a);
    let rb = rank(b);
    if ra != rb {
        return ra.cmp(&rb);
    }

    match (a, b) {
        (Integer(x), Integer(y)) => x.as_i64().cmp(&y.as_i64()),
        (Float(x), Float(y)) => x.total_cmp(y),
        (Text(x), Text(y)) => x.as_str().cmp(y.as_str()),
        (Blob(x), Blob(y)) => x.as_slice().cmp(y.as_slice()),
        (Boolean(x), Boolean(y)) => u8::from(*x).cmp(&u8::from(*y)),
        (Formula(x), Formula(y)) => x.cmp(y),
        (Null, Null) => Ordering::Equal,
        _ => {
            let sa = format!("{a:?}");
            let sb = format!("{b:?}");
            sa.cmp(&sb)
        }
    }
}

impl<S: StorageEngine> Workbook<S> {
    fn evaluate_column(&self, sheet: &str, col_idx: usize) -> Result<Vec<Value>, WorkbookError> {
        let row_count = self.row_count(sheet)?;
        let mut values = Vec::with_capacity(row_count);
        for row_idx in 0..row_count {
            let value = self.get_cell_value(sheet, row_idx, col_idx)?;
            values.push(value);
        }
        Ok(values)
    }

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

        for row in table.rows() {
            if row.values().iter().any(Value::is_formula) {
                return Err(WorkbookError::Formula(
                    "cannot sort sheet containing formulas".to_string(),
                ));
            }
        }

        let column_count = table.schema().columns().len();
        if col_idx >= column_count {
            return Err(WorkbookError::InvalidReference);
        }

        let mut rows = table.rows().to_vec();
        rows.sort_by(|a, b| {
            let av = a.get(col_idx).unwrap_or(&Value::Null);
            let bv = b.get(col_idx).unwrap_or(&Value::Null);
            let ord = compare_values(av, bv);
            if ascending { ord } else { ord.reverse() }
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

        let row_count = table.len();
        let mut matches = Vec::new();
        for row_idx in 0..row_count {
            let actual_value = self.get_cell_value(sheet, row_idx, col_idx)?;
            if &actual_value == value {
                if let Some(row) = table.get(row_idx) {
                    matches.push(row.clone());
                }
            }
        }
        Ok(matches)
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

        let mut values = self.evaluate_column(sheet, col_idx)?;
        values.sort_by(compare_values);
        values.dedup();
        Ok(values)
    }
}
