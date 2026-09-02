mod cell_ref;
mod parser;
mod range;

pub use cell_ref::CellRef;
pub use parser::{col_index_to_letter, col_letter_to_index, parse_cell_ref, parse_range};
pub use range::{CellRange, CellRangeIter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinateError {
    InvalidColumn,
    InvalidRow,
    InvalidReference(String),
    InvalidRange(String),
}

impl std::fmt::Display for CoordinateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidColumn => write!(f, "invalid column"),
            Self::InvalidRow => write!(f, "invalid row"),
            Self::InvalidReference(s) => write!(f, "invalid reference: {}", s),
            Self::InvalidRange(s) => write!(f, "invalid range: {}", s),
        }
    }
}

impl std::error::Error for CoordinateError {}
