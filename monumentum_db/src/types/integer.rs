use crate::error::DbError;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(i64);

impl Integer {
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_i64(self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    #[must_use]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    #[must_use]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        match self.0.checked_mul(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    #[must_use]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        match self.0.checked_div(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(i64::from_le_bytes(bytes))
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for Integer {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl TryFrom<&str> for Integer {
    type Error = DbError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parsed = s
            .parse::<i64>()
            .map_err(|e| DbError::type_mismatch(format!("invalid integer: {e}")))?;
        Ok(Self::new(parsed))
    }
}
