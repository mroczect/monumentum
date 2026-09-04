use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob(Vec<u8>);

impl Blob {
    #[must_use]
    pub const fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub fn try_new(value: Vec<u8>) -> Result<Self, crate::error::DbError> {
        if value.len() > crate::constants::MAX_BLOB_SIZE {
            return Err(crate::error::DbError::invalid_operation(format!(
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

impl From<Vec<u8>> for Blob {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for Blob {
    fn from(value: &[u8]) -> Self {
        Self::new(value.to_vec())
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
