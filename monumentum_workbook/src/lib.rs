#![cfg_attr(test, allow(unused_crate_dependencies))]
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
use monumentum_query::formula::{FunctionImpl, FunctionRegistry};
mod error;
pub mod menu;
pub mod transaction;

pub use error::WorkbookError;

#[derive(Debug)]
pub struct Workbook<S: StorageEngine> {
    pub(crate) catalog: Catalog,
    pub(crate) storage: S,
    pub(crate) functions: FunctionRegistry,
}

impl<S: StorageEngine> Workbook<S> {
    #[must_use]
    pub(crate) fn default_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        monumentum_functions::register_all(&mut registry);
        registry
    }

    #[must_use]
    pub const fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub const fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub fn register_function(&mut self, name: &str, func: FunctionImpl) {
        self.functions.register(name, func);
    }

    #[must_use]
    pub const fn functions(&self) -> &FunctionRegistry {
        &self.functions
    }
}

impl<S: StorageEngine> Workbook<S> {
    pub fn sheet(&self, name: &str) -> Result<&Table, WorkbookError> {
        self.catalog.get_table(name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(name).to_string())
        })
    }

    pub fn sheet_mut(&mut self, name: &str) -> Result<&mut Table, WorkbookError> {
        self.catalog.get_table_mut(name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(name).to_string())
        })
    }

    pub fn persist_catalog(&mut self) -> Result<(), WorkbookError> {
        self.storage.save_catalog(&self.catalog)?;
        Ok(())
    }

    pub fn set_allowed_values(
        &mut self,
        sheet: &str,
        col_name: &str,
        values: Vec<Value>,
    ) -> Result<(), WorkbookError> {
        self.ensure_writable(sheet)?;
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let idx = table.schema().column_index(col_name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::column_not_found(col_name).to_string())
        })?;
        let col_def = table
            .schema_mut()
            .columns_mut()
            .get_mut(idx)
            .ok_or_else(|| {
                WorkbookError::Db(
                    monumentum_db::error::DbError::column_not_found(col_name).to_string(),
                )
            })?;
        col_def.set_allowed_values(Some(values));
        Ok(())
    }

    pub fn validate_cell(
        &self,
        sheet: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        let value = table
            .get(row_idx)
            .and_then(|r| r.get(col_idx))
            .ok_or(WorkbookError::InvalidReference)?;
        let col_def = table
            .schema()
            .columns()
            .get(col_idx)
            .ok_or(WorkbookError::InvalidReference)?;
        col_def.validate_value(value)?;
        Ok(())
    }

    pub fn protect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table.set_read_only(true);
        Ok(())
    }

    pub fn unprotect_sheet(&mut self, sheet: &str) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        table.set_read_only(false);
        Ok(())
    }

    pub fn is_sheet_protected(&self, sheet: &str) -> Result<bool, WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        Ok(table.is_read_only())
    }

    pub(crate) fn ensure_writable(&self, sheet: &str) -> Result<(), WorkbookError> {
        let table = self.catalog.get_table(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet).to_string())
        })?;
        if table.is_read_only() {
            return Err(WorkbookError::Db(
                monumentum_db::error::DbError::invalid_operation("sheet is protected").to_string(),
            ));
        }
        Ok(())
    }
}
