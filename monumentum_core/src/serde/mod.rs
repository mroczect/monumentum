mod decode;
mod encode;

use monumentum_handler::{
    Value,
    error::DbError,
    types::{Blob, Float, Integer, Text},
};
use std::io::{Cursor, Read};

pub(crate) const TAG_NULL: u8 = 0;
pub(crate) const TAG_INTEGER: u8 = 1;
pub(crate) const TAG_FLOAT: u8 = 2;
pub(crate) const TAG_TEXT: u8 = 3;
pub(crate) const TAG_BLOB: u8 = 4;
pub(crate) const TAG_BOOLEAN: u8 = 5;

pub(crate) const FORMAT_VERSION: u32 = 1;
pub(crate) const MAX_READ_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_VEC_ELEMENTS: usize = 1_000_000;

pub(crate) trait Encode {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError>;
}

impl Encode for u8 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.push(*self);
        Ok(())
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }
}

impl Encode for i64 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }
}

impl Encode for f64 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        buf.push(u8::from(*self));
        Ok(())
    }
}

impl Encode for &[u8] {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        let len = u64::try_from(self.len())
            .map_err(|e| DbError::invalid_operation(format!("length too large: {e}")))?;
        len.encode(buf)?;
        buf.extend_from_slice(self);
        Ok(())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        match self {
            Some(v) => {
                true.encode(buf)?;
                v.encode(buf)?;
            }
            None => {
                false.encode(buf)?;
            }
        }
        Ok(())
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        let len = u32::try_from(self.len())
            .map_err(|e| DbError::invalid_operation(format!("too many elements: {e}")))?;
        len.encode(buf)?;
        for item in self {
            item.encode(buf)?;
        }
        Ok(())
    }
}

impl<T: Encode> Encode for &T {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        (*self).encode(buf)
    }
}

impl Encode for Value {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        match self {
            Self::Null => TAG_NULL.encode(buf),
            Self::Integer(i) => {
                TAG_INTEGER.encode(buf)?;
                i.as_i64().encode(buf)
            }
            Self::Float(f) => {
                TAG_FLOAT.encode(buf)?;
                f.as_f64().encode(buf)
            }
            Self::Text(t) => {
                TAG_TEXT.encode(buf)?;
                t.as_bytes().encode(buf)
            }
            Self::Blob(b) => {
                TAG_BLOB.encode(buf)?;
                b.as_slice().encode(buf)
            }
            Self::Boolean(b) => {
                TAG_BOOLEAN.encode(buf)?;
                b.encode(buf)
            }
            _ => Err(DbError::unsupported("unsupported value type")),
        }
    }
}

pub(crate) trait Decode: Sized {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError>;
}

impl Decode for u8 {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let mut buf = [0_u8; 1];
        cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Decode for u32 {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let mut buf = [0_u8; 4];
        cursor.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl Decode for u64 {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let mut buf = [0_u8; 8];
        cursor.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl Decode for i64 {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let mut buf = [0_u8; 8];
        cursor.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl Decode for f64 {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let mut buf = [0_u8; 8];
        cursor.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl Decode for bool {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        Ok(u8::decode(cursor)? != 0)
    }
}

impl Decode for String {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let bytes = Vec::<u8>::decode(cursor)?;
        Self::from_utf8(bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid UTF-8: {e}"),
            ))
        })
    }
}

impl Decode for Vec<u8> {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let len = u64::decode(cursor)?;
        let len = usize::try_from(len).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("length too large for platform: {e}"),
            ))
        })?;
        if len > MAX_READ_BYTES {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("declared length {len} exceeds maximum allowed"),
            )));
        }

        let remaining = cursor
            .get_ref()
            .len()
            .saturating_sub(usize::try_from(cursor.position()).unwrap_or(usize::MAX));
        if len > remaining {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "not enough bytes for declared length",
            )));
        }

        let mut buf = vec![0_u8; len];
        cursor.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let has_value = bool::decode(cursor)?;
        if has_value {
            Ok(Some(T::decode(cursor)?))
        } else {
            Ok(None)
        }
    }
}

impl Decode for Vec<Value> {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let count = u32::decode(cursor)? as usize;
        if count > MAX_VEC_ELEMENTS {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "vector too large",
            )));
        }
        let mut vec = Self::with_capacity(count);
        for _ in 0..count {
            vec.push(Value::decode(cursor)?);
        }
        Ok(vec)
    }
}

impl Decode for Value {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let tag = u8::decode(cursor)?;
        match tag {
            TAG_NULL => Ok(Self::Null),
            TAG_INTEGER => {
                let v = i64::decode(cursor)?;
                Ok(Self::Integer(Integer::new(v)))
            }
            TAG_FLOAT => {
                let v = f64::decode(cursor)?;
                Float::try_new(v).map(Self::Float)
            }
            TAG_TEXT => {
                let s = String::decode(cursor)?;
                Ok(Self::Text(Text::try_new(s)?))
            }
            TAG_BLOB => {
                let b = Vec::<u8>::decode(cursor)?;
                Ok(Self::Blob(Blob::try_new(b)?))
            }
            TAG_BOOLEAN => {
                let b = bool::decode(cursor)?;
                Ok(Self::Boolean(b))
            }
            _ => Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid value tag",
            ))),
        }
    }
}

pub fn encode_catalog(catalog: &crate::catalog::Catalog) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::new();
    catalog.encode(&mut buf)?;
    Ok(buf)
}

pub fn decode_catalog(data: &[u8]) -> Result<crate::catalog::Catalog, DbError> {
    let mut cursor = Cursor::new(data);
    crate::catalog::Catalog::decode(&mut cursor)
}
