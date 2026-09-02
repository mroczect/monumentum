use super::col_index_to_letter;
use std::fmt;

const MAX_COLUMNS: u32 = 16384;
const MAX_ROWS: u32 = 1048576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRef {
    pub col: u32,
    pub row: u32,
    pub abs_col: bool,
    pub abs_row: bool,
    pub sheet: Option<String>,
}

impl CellRef {
    pub fn new(col: u32, row: u32) -> Self {
        Self {
            col,
            row,
            abs_col: false,
            abs_row: false,
            sheet: None,
        }
    }

    pub fn with_sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        self.col < MAX_COLUMNS && self.row < MAX_ROWS
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sheet) = &self.sheet {
            write!(f, "{}!", sheet)?;
        }
        if self.abs_col {
            write!(f, "$")?;
        }
        write!(f, "{}", col_index_to_letter(self.col))?;
        if self.abs_row {
            write!(f, "$")?;
        }
        write!(f, "{}", self.row + 1)
    }
}
