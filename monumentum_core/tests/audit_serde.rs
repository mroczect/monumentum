#![allow(unused_crate_dependencies)]

//! Serde edge-case audit.

use monumentum_core::serde::{decode_row, encode_row};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::types::{Blob, Text};

#[test]
fn test_row_roundtrip_all_types() -> Result<(), monumentum_handler::DbError> {
    let row = Row::new(vec![
        Value::Null,
        Value::from(0i64),
        Value::from(-1i64),
        Value::try_from(1.5)?,
        Value::from(Text::try_new("héllo".into())?),
        Value::from(Blob::try_new(vec![0, 1, 2])?),
        Value::from(true),
        Value::from(false),
    ]);
    let bytes = encode_row(&row)?;
    assert_eq!(decode_row(&bytes)?, row);
    Ok(())
}

#[test]
fn test_row_roundtrip_large_blob() -> Result<(), monumentum_handler::DbError> {
    let row = Row::new(vec![Value::from(Blob::try_new(vec![0xAA; 60_000])?)]);
    let bytes = encode_row(&row)?;
    assert_eq!(decode_row(&bytes)?, row);
    Ok(())
}

#[test]
fn test_row_decode_empty() {
    assert!(decode_row(&[]).is_err());
}

#[test]
fn test_row_decode_truncated_header() {
    assert!(decode_row(&[0u8; 4]).is_err());
}

#[test]
fn test_row_decode_invalid_tag() {
    let payload = {
        let mut v = Vec::new();
        v.extend_from_slice(&1u32.to_le_bytes());
        v.push(0xFF);
        v
    };
    let len_u32 = u32::try_from(payload.len()).unwrap_or_else(|_| unreachable!());
    let mut buf = Vec::new();
    buf.extend_from_slice(&len_u32.to_le_bytes());
    buf.extend_from_slice(&payload);
    assert!(decode_row(&buf).is_err());
}

#[test]
fn test_row_empty_roundtrip() -> Result<(), monumentum_handler::DbError> {
    let row = Row::new(vec![]);
    let bytes = encode_row(&row)?;
    assert_eq!(decode_row(&bytes)?, row);
    Ok(())
}
