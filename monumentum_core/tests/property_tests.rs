#![allow(unused_crate_dependencies)]

use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{decode_catalog, decode_row, encode_catalog, encode_row};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::types::{Blob, Text};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

fn valid_string(min_len: usize, max_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::char::range('a', 'z'), min_len..max_len)
        .prop_map(|v| v.into_iter().collect::<String>())
}

fn valid_text() -> impl Strategy<Value = Text> {
    valid_string(0, 100).prop_map(|s| Text::try_new(s).unwrap_or_else(|_| unreachable!()))
}

fn valid_blob(max_len: usize) -> impl Strategy<Value = Blob> {
    proptest::collection::vec(any::<u8>(), 0..max_len)
        .prop_map(|v| Blob::try_new(v).unwrap_or_else(|_| unreachable!()))
}

fn valid_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<i64>().prop_map(Value::from),
        any::<f64>().prop_map(|f| Value::try_from(f).unwrap_or(Value::Null)),
        valid_text().prop_map(Value::from),
        valid_blob(50).prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        Just(Value::Null),
    ]
}

fn valid_row(max_cols: usize) -> impl Strategy<Value = Row> {
    proptest::collection::vec(valid_value(), 1..max_cols).prop_map(Row::new)
}

fn valid_table_name() -> impl Strategy<Value = String> {
    valid_string(1, 20)
}

proptest! {
    #[test]
    fn prop_value_roundtrip(val in valid_value()) {
        let row = Row::new(vec![val.clone()]);
        let encoded = encode_row(&row).map_err(|e| TestCaseError::fail(format!("encode failed: {e}")))?;
        let decoded_row = decode_row(&encoded).map_err(|e| TestCaseError::fail(format!("decode failed: {e}")))?;
        prop_assert_eq!(decoded_row.len(), 1);
        let decoded_val = decoded_row.get(0).cloned().unwrap_or(Value::Null);
        if let (Value::Float(f1), Value::Float(f2)) = (&val, &decoded_val) {
            prop_assert!((f1.as_f64() - f2.as_f64()).abs() < 1e-12);
        } else {
            prop_assert_eq!(val, decoded_val);
        }
    }

    #[test]
    fn prop_row_roundtrip(row in valid_row(5)) {
        let encoded = encode_row(&row).map_err(|e| TestCaseError::fail(format!("encode failed: {e}")))?;
        let decoded_row = decode_row(&encoded).map_err(|e| TestCaseError::fail(format!("decode failed: {e}")))?;
        prop_assert_eq!(row, decoded_row);
    }

    #[test]
    fn prop_catalog_roundtrip(
        tables in proptest::collection::btree_set(valid_table_name(), 0..5)
            .prop_map(|set| set.into_iter().collect::<Vec<_>>())
    ) {
        let mut catalog = Catalog::new();
        for name in tables {
            let schema = TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)])
                .map_err(|e| TestCaseError::fail(format!("schema failed: {e}")))?;
            catalog.create_table(schema).map_err(|e| TestCaseError::fail(format!("create table failed: {e}")))?;
        }
        let encoded = encode_catalog(&catalog).map_err(|e| TestCaseError::fail(format!("encode failed: {e}")))?;
        let decoded_catalog = decode_catalog(&encoded).map_err(|e| TestCaseError::fail(format!("decode failed: {e}")))?;
        prop_assert_eq!(catalog, decoded_catalog);
    }
}
