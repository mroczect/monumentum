use super::{CellRef, CoordinateError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellRef,
    pub end: CellRef,
}

impl CellRange {
    pub fn try_new(start: CellRef, end: CellRef) -> Result<Self, CoordinateError> {
        if start.sheet != end.sheet {
            return Err(CoordinateError::InvalidRange(
                "sheet mismatch in range".to_string(),
            ));
        }
        let (start, end) = if start.row > end.row || (start.row == end.row && start.col > end.col) {
            (end, start)
        } else {
            (start, end)
        };
        Ok(Self { start, end })
    }

    pub fn new_unchecked(start: CellRef, end: CellRef) -> Self {
        let (start, end) = if start.row > end.row || (start.row == end.row && start.col > end.col) {
            (end, start)
        } else {
            (start, end)
        };
        Self { start, end }
    }

    pub fn iter(&self) -> CellRangeIter<'_> {
        CellRangeIter {
            range: self,
            current_row: self.start.row,
            current_col: self.start.col,
            done: false,
        }
    }

    pub fn contains(&self, cell: &CellRef) -> bool {
        debug_assert!(self.start.row <= self.end.row);
        debug_assert!(self.start.col <= self.end.col || self.start.row < self.end.row);
        cell.row >= self.start.row
            && cell.row <= self.end.row
            && cell.col >= self.start.col
            && cell.col <= self.end.col
    }

    pub fn is_valid(&self) -> bool {
        self.start.row <= self.end.row
            && (self.start.row < self.end.row || self.start.col <= self.end.col)
            && self.start.sheet == self.end.sheet
    }
}

pub struct CellRangeIter<'a> {
    range: &'a CellRange,
    current_row: u32,
    current_col: u32,
    done: bool,
}

impl<'a> Iterator for CellRangeIter<'a> {
    type Item = CellRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if !self.range.is_valid() {
            self.done = true;
            return None;
        }
        if self.current_row > self.range.end.row {
            self.done = true;
            return None;
        }
        let cell = CellRef {
            col: self.current_col,
            row: self.current_row,
            abs_col: false,
            abs_row: false,
            sheet: self.range.start.sheet.clone(),
        };
        if self.current_col >= self.range.end.col {
            self.current_col = self.range.start.col;
            self.current_row += 1;
        } else {
            self.current_col += 1;
        }
        Some(cell)
    }
}
