use super::{CellRange, CellRef, CoordinateError};

const MAX_COLUMNS: u32 = 16384;
const MAX_ROWS: u32 = 1048576;
const MAX_REFERENCE_LENGTH: usize = 1024;

pub fn col_letter_to_index(letters: &str) -> Result<u32, CoordinateError> {
    if letters.is_empty() {
        return Err(CoordinateError::InvalidColumn);
    }
    let mut result: u32 = 0;
    for c in letters.chars() {
        if !c.is_ascii_uppercase() {
            return Err(CoordinateError::InvalidColumn);
        }
        let val = (c as u32) - ('A' as u32) + 1;
        result = result
            .checked_mul(26)
            .and_then(|r| r.checked_add(val))
            .ok_or(CoordinateError::InvalidColumn)?;
        if result > MAX_COLUMNS {
            return Err(CoordinateError::InvalidColumn);
        }
    }
    Ok(result - 1)
}

pub fn col_index_to_letter(index: u32) -> String {
    if index >= MAX_COLUMNS {
        return "#REF!".to_string();
    }
    let mut s = String::new();
    let mut n = index + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        s.insert(0, (b'A' + rem as u8) as char);
        n = (n - 1) / 26;
    }
    s
}

pub fn parse_cell_ref(input: &str) -> Result<CellRef, CoordinateError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(CoordinateError::InvalidReference(input.to_string()));
    }
    if input.len() > MAX_REFERENCE_LENGTH {
        return Err(CoordinateError::InvalidReference(
            "reference exceeds maximum length".to_string(),
        ));
    }

    let (sheet, local) = match input.split_once('!') {
        Some((s, l)) => (Some(s.to_string()), l),
        None => (None, input),
    };

    if let Some(sheet_name) = &sheet
        && (sheet_name.is_empty() || sheet_name.contains('!') || sheet_name.contains(':'))
    {
        return Err(CoordinateError::InvalidReference(input.to_string()));
    }

    let mut chars = local.chars().peekable();

    let mut abs_col = false;
    if chars.peek() == Some(&'$') {
        abs_col = true;
        chars.next();
    }

    let mut col_letters = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_uppercase() {
            col_letters.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if col_letters.is_empty() {
        return Err(CoordinateError::InvalidReference(input.to_string()));
    }

    let mut abs_row = false;
    if chars.peek() == Some(&'$') {
        abs_row = true;
        chars.next();
    }

    let mut row_digits = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            row_digits.push(c);
            chars.next();
        } else {
            return Err(CoordinateError::InvalidReference(input.to_string()));
        }
    }

    if row_digits.is_empty() {
        return Err(CoordinateError::InvalidReference(input.to_string()));
    }

    let row: u32 = row_digits
        .parse()
        .map_err(|_| CoordinateError::InvalidRow)?;
    if row == 0 || row > MAX_ROWS {
        return Err(CoordinateError::InvalidRow);
    }

    let col = col_letter_to_index(&col_letters)?;

    Ok(CellRef {
        col,
        row: row - 1,
        abs_col,
        abs_row,
        sheet,
    })
}

pub fn parse_range(input: &str) -> Result<CellRange, CoordinateError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(CoordinateError::InvalidRange(input.to_string()));
    }
    if input.len() > MAX_REFERENCE_LENGTH {
        return Err(CoordinateError::InvalidRange(
            "range exceeds maximum length".to_string(),
        ));
    }

    let (sheet, local) = match input.split_once('!') {
        Some((s, l)) => (Some(s.to_string()), l),
        None => (None, input),
    };

    if let Some(sheet_name) = &sheet
        && (sheet_name.is_empty() || sheet_name.contains('!') || sheet_name.contains(':'))
    {
        return Err(CoordinateError::InvalidRange(input.to_string()));
    }

    let parts: Vec<&str> = local.split(':').collect();
    match parts.as_slice() {
        [_] => {
            let cell = parse_cell_ref(input)?;
            CellRange::try_new(cell.clone(), cell)
        }
        [start, end] => {
            let start_str = if let Some(s) = &sheet {
                format!("{}!{}", s, start)
            } else {
                (*start).to_string()
            };
            let end_str = if let Some(s) = &sheet {
                format!("{}!{}", s, end)
            } else {
                (*end).to_string()
            };

            let start_ref = parse_cell_ref(&start_str)?;
            let end_ref = parse_cell_ref(&end_str)?;

            CellRange::try_new(start_ref, end_ref)
        }
        _ => Err(CoordinateError::InvalidRange(input.to_string())),
    }
}
