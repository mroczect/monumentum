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
    Boolean(bool),
}

impl Value {
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[must_use]
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    #[must_use]
    pub const fn is_blob(&self) -> bool {
        matches!(self, Self::Blob(_))
    }

    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Text(_) => "text",
            Self::Blob(_) => "blob",
            Self::Boolean(_) => "boolean",
        }
    }

    #[must_use]
    pub const fn as_integer(&self) -> Option<&Integer> {
        match self {
            Self::Integer(v) => Some(v),
            Self::Null | Self::Float(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => None,
        }
    }

    #[must_use]
    pub const fn as_float(&self) -> Option<&Float> {
        match self {
            Self::Float(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub const fn as_text(&self) -> Option<&Text> {
        match self {
            Self::Text(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub const fn as_blob(&self) -> Option<&Blob> {
        match self {
            Self::Blob(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Text(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Text(_) | Self::Blob(_) => None,
        }
    }

    #[must_use]
    pub fn into_integer(self) -> Option<Integer> {
        match self {
            Self::Integer(v) => Some(v),
            Self::Null | Self::Float(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => None,
        }
    }

    #[must_use]
    pub fn into_float(self) -> Option<Float> {
        match self {
            Self::Float(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub fn into_text(self) -> Option<Text> {
        match self {
            Self::Text(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub fn into_blob(self) -> Option<Blob> {
        match self {
            Self::Blob(v) => Some(v),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Text(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub fn into_boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(b),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Text(_) | Self::Blob(_) => None,
        }
    }

    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(i.as_i64()),
            Self::Null | Self::Float(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => None,
        }
    }

    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(f.as_f64()),
            Self::Null | Self::Integer(_) | Self::Text(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }

    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        self.as_boolean()
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(t) => Some(t.as_str()),
            Self::Null | Self::Integer(_) | Self::Float(_) | Self::Blob(_) | Self::Boolean(_) => {
                None
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Integer(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{:?}", v.as_f64()),
            Self::Text(v) => write!(f, "'{}'", v.as_str().replace('\'', "''")),
            Self::Blob(v) => write!(f, "{v}"),
            Self::Boolean(b) => write!(f, "{b}"),
        }
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
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

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
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

impl TryFrom<String> for Value {
    type Error = crate::error::DbError;
    fn try_from(v: String) -> Result<Self, Self::Error> {
        Ok(Self::Text(Text::try_new(v)?))
    }
}
impl TryFrom<&str> for Value {
    type Error = crate::error::DbError;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        Ok(Self::Text(Text::try_new(v.to_string())?))
    }
}

impl TryFrom<Vec<u8>> for Value {
    type Error = crate::error::DbError;
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::Blob(Blob::try_new(v)?))
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = crate::error::DbError;
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::Blob(Blob::try_new(v.to_vec())?))
    }
}
