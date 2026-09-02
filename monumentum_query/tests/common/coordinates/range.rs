use monumentum_query::coordinates::{CellRange, CellRef, CoordinateError};

#[test]
fn cell_range_try_new_success_same_sheet() {
    let a = CellRef::new(0, 0);
    let b = CellRef::new(2, 2);
    let range = CellRange::try_new(a, b).unwrap();
    assert_eq!(range.start.to_string(), "A1");
    assert_eq!(range.end.to_string(), "C3");
}

#[test]
fn cell_range_try_new_rejects_sheet_mismatch() {
    let a = CellRef::new(0, 0).with_sheet("Sheet1");
    let b = CellRef::new(2, 2).with_sheet("Sheet2");
    assert!(matches!(
        CellRange::try_new(a, b),
        Err(CoordinateError::InvalidRange(_))
    ));
}

#[test]
fn cell_range_new_unchecked_normalizes_order() {
    let start = CellRef::new(5, 5);
    let end = CellRef::new(0, 0);
    let range = CellRange::new_unchecked(start, end);
    assert_eq!(range.start.to_string(), "A1");
    assert_eq!(range.end.to_string(), "F6");
}

#[test]
fn cell_range_contains() {
    let range = CellRange::new_unchecked(CellRef::new(0, 0), CellRef::new(2, 2));
    let inside = CellRef::new(1, 1);
    let outside = CellRef::new(3, 3);
    let boundary_start = CellRef::new(0, 0);
    let boundary_end = CellRef::new(2, 2);
    assert!(range.contains(&inside));
    assert!(!range.contains(&outside));
    assert!(range.contains(&boundary_start));
    assert!(range.contains(&boundary_end));
}

#[test]
fn cell_range_iter_order() {
    let range = CellRange::new_unchecked(CellRef::new(1, 1), CellRef::new(2, 3));
    let cells: Vec<String> = range.iter().map(|c| c.to_string()).collect();
    assert_eq!(cells, vec!["B2", "C2", "B3", "C3", "B4", "C4"]);
}

#[test]
fn cell_range_iter_flag_absolute_false() {
    let start = CellRef::new(0, 0);
    let mut end = CellRef::new(1, 1);
    end.abs_col = true;
    end.abs_row = true;
    let range = CellRange::new_unchecked(start, end);
    for cell in range.iter() {
        assert!(!cell.abs_col);
        assert!(!cell.abs_row);
    }
}

#[test]
fn cell_range_is_valid() {
    let good = CellRange::new_unchecked(CellRef::new(0, 0), CellRef::new(2, 2));
    assert!(good.is_valid());

    let bad_order = CellRange {
        start: CellRef::new(2, 2),
        end: CellRef::new(0, 0),
    };
    assert!(!bad_order.is_valid());
}
