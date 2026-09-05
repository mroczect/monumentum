use monumentum_handler::error::DbError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IndexKey {
    Integer(i64),
    Float(u64),
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
}

impl IndexKey {
    #[must_use]
    pub fn from_value(v: &monumentum_handler::Value) -> Option<Self> {
        match v {
            monumentum_handler::Value::Null => None,
            monumentum_handler::Value::Integer(i) => Some(Self::Integer(i.as_i64())),
            monumentum_handler::Value::Float(f) => {
                let bits = f.as_f64().to_bits();
                let bits = if f.as_f64() == 0.0 {
                    0.0_f64.to_bits()
                } else {
                    bits
                };
                Some(Self::Float(bits))
            }
            monumentum_handler::Value::Text(t) => Some(Self::Text(t.as_str().to_string())),
            monumentum_handler::Value::Blob(b) => Some(Self::Blob(b.as_slice().to_vec())),
            monumentum_handler::Value::Boolean(b) => Some(Self::Boolean(*b)),
            _ => None,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, DbError> {
        let mut buf = Vec::new();
        match self {
            Self::Integer(i) => {
                buf.push(0);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            Self::Float(bits) => {
                buf.push(1);
                buf.extend_from_slice(&bits.to_le_bytes());
            }
            Self::Text(s) => {
                buf.push(2);
                let len = u32::try_from(s.len())
                    .map_err(|e| DbError::invalid_operation(format!("text key too long: {e}")))?;
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(s.as_bytes());
            }
            Self::Blob(b) => {
                buf.push(3);
                let len = u32::try_from(b.len())
                    .map_err(|e| DbError::invalid_operation(format!("blob key too long: {e}")))?;
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(b);
            }
            Self::Boolean(b) => {
                buf.push(4);
                buf.push(u8::from(*b));
            }
        }
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DbError> {
        let tag = bytes.first().ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "empty key bytes",
            ))
        })?;
        match *tag {
            0 => Self::decode_integer(bytes),
            1 => Self::decode_float(bytes),
            2 => Self::decode_text(bytes),
            3 => Self::decode_blob(bytes),
            4 => Self::decode_boolean(bytes),
            _ => Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid key tag",
            ))),
        }
    }

    fn decode_integer(bytes: &[u8]) -> Result<Self, DbError> {
        let arr = bytes.get(1..9).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid integer key length",
            ))
        })?;
        let i = i64::from_le_bytes(arr.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid integer key: {e}"),
            ))
        })?);
        Ok(Self::Integer(i))
    }

    fn decode_float(bytes: &[u8]) -> Result<Self, DbError> {
        let arr = bytes.get(1..9).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid float key length",
            ))
        })?;
        let bits = u64::from_le_bytes(arr.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid float key: {e}"),
            ))
        })?);
        Ok(Self::Float(bits))
    }

    fn decode_text(bytes: &[u8]) -> Result<Self, DbError> {
        let len_bytes = bytes.get(1..5).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid text key length prefix",
            ))
        })?;
        let len = u32::from_le_bytes(len_bytes.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid text key length: {e}"),
            ))
        })?) as usize;
        let end = 5_usize.checked_add(len).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "key length overflow",
            ))
        })?;
        let text_bytes = bytes.get(5..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "text key bytes missing",
            ))
        })?;
        let s = core::str::from_utf8(text_bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid utf8 in text key: {e}"),
            ))
        })?;
        Ok(Self::Text(s.to_string()))
    }

    fn decode_blob(bytes: &[u8]) -> Result<Self, DbError> {
        let len_bytes = bytes.get(1..5).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid blob key length prefix",
            ))
        })?;
        let len = u32::from_le_bytes(len_bytes.try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid blob key length: {e}"),
            ))
        })?) as usize;
        let end = 5_usize.checked_add(len).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "key length overflow",
            ))
        })?;
        let blob_bytes = bytes.get(5..end).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "blob key bytes missing",
            ))
        })?;
        Ok(Self::Blob(blob_bytes.to_vec()))
    }

    fn decode_boolean(bytes: &[u8]) -> Result<Self, DbError> {
        let b = bytes.get(1).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "boolean key byte missing",
            ))
        })?;
        Ok(Self::Boolean(*b != 0))
    }
}
