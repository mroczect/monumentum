use crate::coordinates::CellRef;
use crate::formula::error::FormulaError;
use monumentum_db::core::value::Value;

pub trait FormulaContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError>;
}
