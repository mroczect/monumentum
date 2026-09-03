use crate::{Workbook, WorkbookError};
use core::cell::RefCell;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
use monumentum_query::coordinates::CellRef;
use monumentum_query::formula::{FormulaContext, FormulaError, evaluate, parse, tokenize};
use std::collections::HashSet;

const MAX_EVAL_DEPTH: usize = 128;

impl<S: StorageEngine> Workbook<S> {
    pub fn set_formula(
        &mut self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
        formula: &str,
    ) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;

        table.set_cell(row_idx, col_idx, Value::Formula(formula.to_string()))?;
        Ok(())
    }

    pub fn get_cell_value(
        &self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<Value, WorkbookError> {
        let stack = RefCell::new(HashSet::new());
        let depth = RefCell::new(0);
        self.evaluate_cell(sheet, row_idx, col_idx, &stack, &depth)
    }

    fn evaluate_cell(
        &self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
        stack: &RefCell<HashSet<String>>,
        depth: &RefCell<usize>,
    ) -> Result<Value, WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let row = table.get(row_idx).ok_or(WorkbookError::InvalidReference)?;
        let value = row.get(col_idx).ok_or(WorkbookError::InvalidReference)?;

        #[allow(clippy::wildcard_enum_match_arm)]
        match value {
            Value::Formula(formula_str) => {
                let cell_key = format!("{}!R{}C{}", sheet, row_idx, col_idx);
                if stack.borrow().contains(&cell_key) {
                    return Err(WorkbookError::CircularReference);
                }
                if *depth.borrow() >= MAX_EVAL_DEPTH {
                    return Err(WorkbookError::Formula(
                        "maximum evaluation depth exceeded".to_string(),
                    ));
                }
                let _ = stack.borrow_mut().insert(cell_key.clone());
                {
                    let mut d = depth.borrow_mut();
                    *d = d.saturating_add(1);
                }
                let result = self.evaluate_formula_str(formula_str, sheet, stack, depth);
                {
                    let mut d = depth.borrow_mut();
                    *d = d.saturating_sub(1);
                }
                let _ = stack.borrow_mut().remove(&cell_key);
                result
            }
            _ => Ok(value.clone()),
        }
    }

    fn evaluate_formula_str(
        &self,
        formula: &str,
        current_sheet: &str,
        stack: &RefCell<HashSet<String>>,
        depth: &RefCell<usize>,
    ) -> Result<Value, WorkbookError> {
        let tokens = tokenize(formula)?;
        let expr = parse(&tokens)?;
        let ctx = WorkbookFormulaContext {
            workbook: self,
            current_sheet,
            stack,
            depth,
        };
        evaluate(&expr, &ctx, &self.functions).map_err(WorkbookError::from)
    }
}

struct WorkbookFormulaContext<'a, S: StorageEngine> {
    workbook: &'a Workbook<S>,
    current_sheet: &'a str,
    stack: &'a RefCell<HashSet<String>>,
    depth: &'a RefCell<usize>,
}

impl<S: StorageEngine> FormulaContext for WorkbookFormulaContext<'_, S> {
    #[allow(clippy::wildcard_enum_match_arm)]
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        let sheet = cell.sheet.as_deref().unwrap_or(self.current_sheet);
        let row_idx = cell.row as usize;
        let col_idx = cell.col as usize;
        self.workbook
            .evaluate_cell(sheet, row_idx, col_idx, self.stack, self.depth)
            .map_err(|e| match e {
                WorkbookError::Formula(msg) => FormulaError::Eval(msg),
                WorkbookError::CircularReference => {
                    FormulaError::CircularReference(format!("{}", cell))
                }
                WorkbookError::InvalidReference => {
                    FormulaError::InvalidReference(format!("{}", cell))
                }
                WorkbookError::Db(msg) => FormulaError::Eval(msg),
                other => FormulaError::Eval(other.to_string()),
            })
    }
}
