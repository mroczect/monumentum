use monumentum_query::coordinates::{col_index_to_letter, col_letter_to_index};
use proptest::prelude::*;

proptest! {
    #[test]
    fn col_roundtrip(index in 0u32..16384) {
        let letter = col_index_to_letter(index);
        let parsed = col_letter_to_index(&letter).unwrap();
        prop_assert_eq!(parsed, index);
    }

    #[test]
    fn col_letter_to_index_no_panic(letters in "[A-Z]{1,3}") {
        let _ = col_letter_to_index(&letters);
    }

    #[test]
    fn parse_cell_ref_roundtrip(col in 0u32..16384, row in 1u32..1048576) {
        let cell = monumentum_query::coordinates::CellRef::new(col, row - 1);
        let s = cell.to_string();
        let parsed = monumentum_query::coordinates::parse_cell_ref(&s).unwrap();
        prop_assert_eq!(parsed, cell);
    }
}
