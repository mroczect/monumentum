use crate::constants::MAX_TEXT_SIZE;
use crate::error::DbError;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(String);

impl Text {
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    pub fn try_new(value: String) -> Result<Self, DbError> {
        if value.len() > MAX_TEXT_SIZE {
            return Err(DbError::invalid_operation(format!(
                "text size {} exceeds maximum {}",
                value.len(),
                MAX_TEXT_SIZE
            )));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn to_lowercase(&self) -> Self {
        Self(self.0.to_lowercase())
    }

    #[must_use]
    pub fn to_uppercase(&self) -> Self {
        Self(self.0.to_uppercase())
    }

    #[must_use]
    pub fn contains_ignore_case(&self, needle: &str) -> bool {
        self.0.to_lowercase().contains(&needle.to_lowercase())
    }
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
