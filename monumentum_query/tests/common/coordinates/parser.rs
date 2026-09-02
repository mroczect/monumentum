use monumentum_query::coordinates::{
    col_index_to_letter, col_letter_to_index, parse_cell_ref, parse_range,
};

#[test]
fn test_col_letter_to_index() {
    assert_eq!(col_letter_to_index("A").unwrap(), 0);
    assert_eq!(col_letter_to_index("Z").unwrap(), 25);
    assert_eq!(col_letter_to_index("AA").unwrap(), 26);
    assert_eq!(col_letter_to_index("XFD").unwrap(), 16383);
    assert!(col_letter_to_index("").is_err());
    assert!(col_letter_to_index("a").is_err());
    assert!(col_letter_to_index("XFE").is_err());
    assert!(col_letter_to_index("ZZZ").is_err());
}

#[test]
fn test_col_index_to_letter() {
    assert_eq!(col_index_to_letter(0), "A");
    assert_eq!(col_index_to_letter(25), "Z");
    assert_eq!(col_index_to_letter(26), "AA");
    assert_eq!(col_index_to_letter(16383), "XFD");
    assert_eq!(col_index_to_letter(16384), "#REF!");
}

#[test]
fn test_parse_cell_ref_basic() {
    let cell = parse_cell_ref("A1").unwrap();
    assert_eq!(cell.col, 0);
    assert_eq!(cell.row, 0);
    assert_eq!(cell.to_string(), "A1");
}

#[test]
fn test_parse_cell_ref_absolute() {
    let cell = parse_cell_ref("$B$2").unwrap();
    assert_eq!(cell.col, 1);
    assert_eq!(cell.row, 1);
    assert!(cell.abs_col);
    assert!(cell.abs_row);
    assert_eq!(cell.to_string(), "$B$2");
}

#[test]
fn test_parse_cell_ref_with_sheet() {
    let cell = parse_cell_ref("Sheet2!C3").unwrap();
    assert_eq!(cell.sheet.as_deref(), Some("Sheet2"));
    assert_eq!(cell.col, 2);
    assert_eq!(cell.row, 2);
    assert_eq!(cell.to_string(), "Sheet2!C3");
}

#[test]
fn test_parse_cell_ref_errors() {
    assert!(parse_cell_ref("").is_err());
    assert!(parse_cell_ref("A").is_err());
    assert!(parse_cell_ref("A0").is_err());
    assert!(parse_cell_ref("1A").is_err());
    assert!(parse_cell_ref("A1048577").is_err());
    assert!(parse_cell_ref("XFE1").is_err());
    assert!(parse_cell_ref("Sheet1!A1!B2").is_err());
    assert!(parse_cell_ref("Sheet:1!A1").is_err());
    assert!(parse_cell_ref(&"A1".repeat(1025)).is_err());
}

#[test]
fn test_parse_range() {
    let range = parse_range("A1:C3").unwrap();
    assert_eq!(range.start.to_string(), "A1");
    assert_eq!(range.end.to_string(), "C3");
    let cells: Vec<String> = range.iter().map(|c| c.to_string()).collect();
    assert_eq!(
        cells,
        vec!["A1", "B1", "C1", "A2", "B2", "C2", "A3", "B3", "C3"]
    );
}

#[test]
fn test_parse_range_with_sheet() {
    let range = parse_range("Sheet1!A1:B2").unwrap();
    assert_eq!(range.start.sheet.as_deref(), Some("Sheet1"));
    assert_eq!(range.start.to_string(), "Sheet1!A1");
    assert_eq!(range.end.to_string(), "Sheet1!B2");
}

#[test]
fn test_parse_range_single_cell() {
    let range = parse_range("A1").unwrap();
    assert_eq!(range.start.to_string(), "A1");
    assert_eq!(range.end.to_string(), "A1");
}

#[test]
fn test_parse_range_errors() {
    assert!(parse_range("").is_err());
    assert!(parse_range("A1:B2:C3").is_err());
    assert!(parse_range("Sheet1!A1:Sheet2!B2").is_err());
    assert!(parse_range("A1:B").is_err());
    assert!(parse_range("Sheet1!A1:B2!C3").is_err());
    assert!(parse_range(&"A1:B2".repeat(1024)).is_err());
}
