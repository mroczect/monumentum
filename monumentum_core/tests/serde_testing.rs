#![allow(unused_crate_dependencies)]

use monumentum_core::serde::{decode_row, encode_row};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::types::{Blob, Text};

#[test]
fn test_row_encode_decode_roundtrip() {
    let row = Row::new(vec![
        Value::from(42i64),
        Value::try_from(2.5).unwrap_or(Value::Null),
        Value::from(Text::try_new("text".to_string()).unwrap_or_else(|_| unreachable!())),
        Value::from(Blob::try_new(vec![1, 2, 3]).unwrap_or_else(|_| unreachable!())),
        Value::from(true),
    ]);

    let encoded = encode_row(&row);
    assert!(encoded.is_ok());
    let Ok(encoded_bytes) = encoded else {
        return;
    };
    let decoded = decode_row(&encoded_bytes);
    assert!(decoded.is_ok());
    let Ok(decoded_row) = decoded else {
        return;
    };
    assert_eq!(row.len(), decoded_row.len());
    for (orig, dec) in row.values().iter().zip(decoded_row.values().iter()) {
        if let (Value::Float(f1), Value::Float(f2)) = (orig, dec) {
            assert!((f1.as_f64() - f2.as_f64()).abs() < 1e-12);
        } else {
            assert_eq!(orig, dec);
        }
    }
}
