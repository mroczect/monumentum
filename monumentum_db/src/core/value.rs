use crate::types::{Blob, Float, Integer, Text};
use core::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
#[non_exhaustive]
pub enum Value {
    #[default]
    Null,
    Integer(Integer),
    Float(Float),
    Text(Text),
    Blob(Blob),
}

impl Value {
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    #[must_use]
    pub fn is_blob(&self) -> bool {
        matches!(self, Self::Blob(_))
    }

    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Text(_) => "text",
            Self::Blob(_) => "blob",
        }
    }

    #[must_use]
    pub fn as_integer(&self) -> Option<&Integer> {
        match self {
            Self::Integer(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_float(&self) -> Option<&Float> {
        match self {
            Self::Float(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_text(&self) -> Option<&Text> {
        match self {
            Self::Text(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_blob(&self) -> Option<&Blob> {
        match self {
            Self::Blob(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_integer(self) -> Option<Integer> {
        match self {
            Self::Integer(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_float(self) -> Option<Float> {
        match self {
            Self::Float(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_text(self) -> Option<Text> {
        match self {
            Self::Text(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_blob(self) -> Option<Blob> {
        match self {
            Self::Blob(v) => Some(v),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Integer(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{:?}", v.as_f64()),
            Self::Text(v) => {
                write!(f, "'{}'", v.as_str().replace('\'', "''"))
            }
            Self::Blob(v) => write!(f, "{v}"),
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl From<Integer> for Value {
    fn from(v: Integer) -> Self {
        Self::Integer(v)
    }
}

impl From<Float> for Value {
    fn from(v: Float) -> Self {
        Self::Float(v)
    }
}

impl From<Text> for Value {
    fn from(v: Text) -> Self {
        Self::Text(v)
    }
}

impl From<Blob> for Value {
    fn from(v: Blob) -> Self {
        Self::Blob(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Integer(Integer::new(v))
    }
}

impl TryFrom<f64> for Value {
    type Error = crate::error::DbError;

    fn try_from(v: f64) -> Result<Self, Self::Error> {
        Ok(Self::Float(Float::try_new(v)?))
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Text(Text::new(v))
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::Text(Text::new(v.to_string()))
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Self::Blob(Blob::new(v))
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Self::Blob(Blob::new(v.to_vec()))
    }
}
