use crate::error::DbError;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob(Vec<u8>);

impl Blob {
    pub fn try_new(value: Vec<u8>) -> Result<Self, DbError> {
        if value.len() > crate::constants::MAX_BLOB_SIZE {
            return Err(DbError::invalid_operation(format!(
                "blob size {} exceeds maximum {}",
                value.len(),
                crate::constants::MAX_BLOB_SIZE
            )));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
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
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blob({} bytes)", self.0.len())
    }
}

impl TryFrom<Vec<u8>> for Blob {
    type Error = DbError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&[u8]> for Blob {
    type Error = DbError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_new(value.to_vec())
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
