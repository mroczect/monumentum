use monumentum_db::core::row::Row;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn float_value(value: f64) -> Result<Value, DbError> {
    Value::try_from(value)
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
