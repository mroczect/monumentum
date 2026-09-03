use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::ColumnIndex;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use proptest::prelude::*;

fn float_value(value: f64) -> Result<Value, DbError> {
    Value::try_from(value)
}

fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<i64>().prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        ".*".prop_map(Value::from),
        prop::collection::vec(any::<u8>(), 0..10).prop_map(Value::from),
        any::<f64>()
            .prop_filter("must be finite", |f| f.is_finite())
            .prop_map(|f| Value::try_from(f).unwrap()),
    ]
}

#[test]
fn new_creates_row_with_given_values() {
    let values = vec![Value::from(1_i64), Value::from("hello")];
    let row = Row::new(values.clone());
    assert_eq!(row.values(), &values[..]);
    assert_eq!(row.len(), 2);
    assert!(!row.is_empty());
}

#[test]
fn new_with_empty_vec_creates_empty_row() {
    let row = Row::new(Vec::new());
    assert_eq!(row.len(), 0);
    assert!(row.is_empty());
    assert_eq!(row.values(), &[]);
}

#[test]
fn values_returns_slice_of_all_values() -> Result<(), DbError> {
    let values = vec![Value::Null, Value::from(42_i64), float_value(2.5)?];
    let row = Row::new(values.clone());
    assert_eq!(row.values(), values.as_slice());
    Ok(())
}

#[test]
fn get_returns_value_at_index_if_present() {
    let row = Row::new(vec![Value::from(10_i64), Value::from(20_i64)]);
    let val = row.get(1);
    assert!(val.is_some());
    if let Some(v) = val {
        assert_eq!(v, &Value::from(20_i64));
    }
}

#[test]
fn get_returns_none_if_index_out_of_bounds() {
    let row = Row::new(vec![Value::from(1_i64)]);
    assert!(row.get(1).is_none());
    assert!(row.get(10).is_none());
}

#[test]
fn len_returns_number_of_values() {
    let row = Row::new(vec![Value::Null, Value::from("a"), Value::from("b")]);
    assert_eq!(row.len(), 3);
}

#[test]
fn is_empty_returns_true_only_for_empty_row() {
    let empty_row = Row::new(Vec::new());
    assert!(empty_row.is_empty());

    let non_empty_row = Row::new(vec![Value::Null]);
    assert!(!non_empty_row.is_empty());
}

#[test]
fn row_equality_works() {
    let row1 = Row::new(vec![Value::from(1_i64), Value::from("x")]);
    let row2 = Row::new(vec![Value::from(1_i64), Value::from("x")]);
    assert_eq!(row1, row2);
}

#[test]
fn row_partial_ord_works() {
    let row1 = Row::new(vec![Value::from(1_i64)]);
    let row2 = Row::new(vec![Value::from(2_i64)]);
    assert!(row1 < row2);
}

#[test]
fn clone_creates_independent_copy() {
    let row = Row::new(vec![Value::from("hello")]);
    let cloned = row.clone();
    assert_eq!(row, cloned);
}

#[test]
fn values_mut_returns_mutable_reference() {
    let mut row = Row::new(vec![Value::from(1_i64), Value::from(2_i64)]);
    row.values_mut()[0] = Value::from(10_i64);
    assert_eq!(row.get(0), Some(&Value::from(10_i64)));
}

#[test]
fn get_mut_returns_mutable_reference_at_valid_index() {
    let mut row = Row::new(vec![Value::from("a"), Value::from("b")]);
    if let Some(v) = row.get_mut(1) {
        *v = Value::from("c");
    }
    assert_eq!(row.get(1), Some(&Value::from("c")));
}

#[test]
fn get_mut_returns_none_for_out_of_bounds() {
    let mut row = Row::new(vec![Value::Null]);
    assert!(row.get_mut(5).is_none());
}

#[test]
fn row_partial_ord_with_equal_lengths() -> Result<(), DbError> {
    let row1 = Row::new(vec![Value::from(1_i64), Value::from(2_i64)]);
    let row2 = Row::new(vec![Value::from(1_i64), Value::from(3_i64)]);
    let row3 = Row::new(vec![Value::from(2_i64), Value::from(0_i64)]);
    assert!(row1 < row2);
    assert!(row2 < row3);
    assert!(row1 < row3);
    Ok(())
}

#[test]
fn row_partial_ord_with_different_lengths() {
    let shorter = Row::new(vec![Value::from(1_i64)]);
    let longer = Row::new(vec![Value::from(1_i64), Value::from(2_i64)]);
    assert!(shorter < longer);
}

#[test]
fn row_clone_is_independent() {
    let row = Row::new(vec![Value::from("original")]);
    let mut cloned = row.clone();
    *cloned.get_mut(0).unwrap() = Value::from("changed");
    assert_eq!(row.get(0), Some(&Value::from("original")));
    assert_eq!(cloned.get(0), Some(&Value::from("changed")));
}

#[test]
fn column_index_usize_on_row() {
    let row = Row::new(vec![Value::from(1_i64), Value::from(2_i64)]);
    assert_eq!(0usize.index(&row), Ok(0));
    assert_eq!(1usize.index(&row), Ok(1));
    assert!(2usize.index(&row).is_err());
    assert!(100usize.index(&row).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn new_then_values_returns_same_vec(
        values in prop::collection::vec(value_strategy(), 0..20),
    ) {
        let row = Row::new(values.clone());
        prop_assert_eq!(row.values(), values.as_slice());
    }

    #[test]
    fn len_matches_input_length(
        values in prop::collection::vec(value_strategy(), 0..20),
    ) {
        let len = values.len();
        let row = Row::new(values);
        prop_assert_eq!(row.len(), len);
    }

    #[test]
    fn is_empty_iff_len_zero(
        values in prop::collection::vec(value_strategy(), 0..20),
    ) {
        let expected_empty = values.is_empty();
        let row = Row::new(values);
        prop_assert_eq!(row.is_empty(), expected_empty);
    }

    #[test]
    fn get_within_bounds_returns_value(
        values in prop::collection::vec(value_strategy(), 1..20),
        index in 0_usize..20,
    ) {
        let row = Row::new(values.clone());
        if index < values.len() {
            prop_assert_eq!(row.get(index), Some(&values[index]));
        } else {
            prop_assert_eq!(row.get(index), None);
        }
    }

    #[test]
    fn get_out_of_bounds_returns_none(
        values in prop::collection::vec(value_strategy(), 0..20),
        index in 20_usize..100,
    ) {
        let row = Row::new(values);
        prop_assert_eq!(row.get(index), None);
    }

    #[test]
    fn get_mut_within_bounds_updates_value(
        values in prop::collection::vec(value_strategy(), 1..20),
        index in 0_usize..20,
        new_value in value_strategy(),
    ) {
        let mut row = Row::new(values.clone());
        if index < values.len() {
            if let Some(v) = row.get_mut(index) {
                *v = new_value.clone();
            }
            prop_assert_eq!(row.get(index), Some(&new_value));
        } else {
            prop_assert!(row.get_mut(index).is_none());
        }
    }

    #[test]
    fn values_mut_allows_mutation(
        values in prop::collection::vec(value_strategy(), 1..10),
        index in 0_usize..10,
        new_value in value_strategy(),
    ) {
        let mut row = Row::new(values.clone());
        if index < values.len() {
            row.values_mut()[index] = new_value.clone();
            prop_assert_eq!(row.get(index), Some(&new_value));
        }
    }

    #[test]
    fn equality_consistent_with_original_vec(
        values1 in prop::collection::vec(value_strategy(), 0..10),
        values2 in prop::collection::vec(value_strategy(), 0..10),
    ) {
        let row1 = Row::new(values1.clone());
        let row2 = Row::new(values2.clone());
        prop_assert_eq!(row1 == row2, values1 == values2);
    }

    #[test]
    fn clone_creates_equal_row(
        values in prop::collection::vec(value_strategy(), 0..10),
    ) {
        let row = Row::new(values);
        let cloned = row.clone();
        prop_assert_eq!(row, cloned);
    }

    #[test]
    fn partial_ord_matches_lexicographic_for_integers(
        left in prop::collection::vec(any::<i64>(), 0..10),
        right in prop::collection::vec(any::<i64>(), 0..10),
    ) {
        let left_values: Vec<Value> = left.iter().map(|&x| Value::from(x)).collect();
        let right_values: Vec<Value> = right.iter().map(|&x| Value::from(x)).collect();
        let row_left = Row::new(left_values.clone());
        let row_right = Row::new(right_values.clone());
        prop_assert_eq!(row_left.partial_cmp(&row_right), left_values.partial_cmp(&right_values));
    }
}
