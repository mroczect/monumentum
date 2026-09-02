mod decode;
mod encode;

pub use decode::*;
pub use encode::*;

use crate::core::value::Value;
use crate::error::DbError;
use crate::types::{Blob, Float, Integer, Text};
use std::io::{Cursor, Read};

pub(crate) const TAG_NULL: u8 = 0;
pub(crate) const TAG_INTEGER: u8 = 1;
pub(crate) const TAG_FLOAT: u8 = 2;
pub(crate) const TAG_TEXT: u8 = 3;
pub(crate) const TAG_BLOB: u8 = 4;
pub(crate) const TAG_BOOLEAN: u8 = 5;

pub(crate) const FORMAT_VERSION: u32 = 1;
pub(crate) const MAX_READ_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn write_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

pub(crate) fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn write_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn write_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    write_u64(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, DbError> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

pub(crate) fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, DbError> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

pub(crate) fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64, DbError> {
    let mut b = [0u8; 8];
    cursor.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

pub(crate) fn read_bytes(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>, DbError> {
    let len_u64 = read_u64(cursor)?;
    let len = usize::try_from(len_u64).map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "length too large for platform",
        ))
    })?;
    if len > MAX_READ_BYTES {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("declared length {} exceeds maximum allowed", len),
        )));
    }
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}

pub(crate) fn encode_data_type(dt: &crate::core::schema::column::DataType) -> u8 {
    match dt {
        crate::core::schema::column::DataType::Null => 0,
        crate::core::schema::column::DataType::Integer => 1,
        crate::core::schema::column::DataType::Float => 2,
        crate::core::schema::column::DataType::Text => 3,
        crate::core::schema::column::DataType::Blob => 4,
    }
}

pub(crate) fn decode_data_type(v: u8) -> Result<crate::core::schema::column::DataType, DbError> {
    match v {
        0 => Ok(crate::core::schema::column::DataType::Null),
        1 => Ok(crate::core::schema::column::DataType::Integer),
        2 => Ok(crate::core::schema::column::DataType::Float),
        3 => Ok(crate::core::schema::column::DataType::Text),
        4 => Ok(crate::core::schema::column::DataType::Blob),
        _ => Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid data type tag",
        ))),
    }
}

pub(crate) fn encode_value(value: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    match value {
        Value::Null => {
            write_u8(&mut buf, TAG_NULL);
        }
        Value::Integer(i) => {
            write_u8(&mut buf, TAG_INTEGER);
            write_u64(&mut buf, i.as_i64() as u64);
        }
        Value::Float(f) => {
            write_u8(&mut buf, TAG_FLOAT);
            buf.extend_from_slice(&f.as_f64().to_le_bytes());
        }
        Value::Text(t) => {
            write_u8(&mut buf, TAG_TEXT);
            write_bytes(&mut buf, t.as_bytes());
        }
        Value::Blob(b) => {
            write_u8(&mut buf, TAG_BLOB);
            write_bytes(&mut buf, b.as_slice());
        }
        Value::Boolean(b) => {
            write_u8(&mut buf, TAG_BOOLEAN);
            write_u8(&mut buf, *b as u8);
        }
    }
    buf
}

pub(crate) fn decode_value(cursor: &mut Cursor<&[u8]>) -> Result<Value, DbError> {
    let tag = read_u8(cursor)?;
    match tag {
        TAG_NULL => Ok(Value::Null),
        TAG_INTEGER => {
            let raw = read_u64(cursor)?;
            Ok(Value::Integer(Integer::new(raw as i64)))
        }
        TAG_FLOAT => {
            let mut b = [0u8; 8];
            cursor.read_exact(&mut b)?;
            let f = f64::from_le_bytes(b);
            Float::try_new(f).map(Value::Float)
        }
        TAG_TEXT => {
            let bytes = read_bytes(cursor)?;
            let s = String::from_utf8(bytes).map_err(|e| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            })?;
            Ok(Value::Text(Text::new(s)))
        }
        TAG_BLOB => {
            let bytes = read_bytes(cursor)?;
            Ok(Value::Blob(Blob::new(bytes)))
        }
        TAG_BOOLEAN => {
            let b = read_u8(cursor)? != 0;
            Ok(Value::Boolean(b))
        }
        _ => Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid value tag",
        ))),
    }
}
