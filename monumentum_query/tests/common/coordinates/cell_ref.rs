use monumentum_query::coordinates::CellRef;

#[test]
fn cell_ref_display_simple() {
    let cell = CellRef::new(0, 0);
    assert_eq!(cell.to_string(), "A1");
}

#[test]
fn cell_ref_display_with_sheet() {
    let cell = CellRef::new(2, 5).with_sheet("Data");
    assert_eq!(cell.to_string(), "Data!C6");
}

#[test]
fn cell_ref_display_absolute() {
    let mut cell = CellRef::new(1, 1);
    cell.abs_col = true;
    cell.abs_row = true;
    assert_eq!(cell.to_string(), "$B$2");
}
