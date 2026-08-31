use crate::error::DbError;
use core::cmp::Ordering;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Float(f64);

impl Float {
    pub fn try_new(value: f64) -> Result<Self, DbError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(DbError::type_mismatch(
                "float must be finite (no NaN or infinity)",
            ))
        }
    }

    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }

    pub fn try_from_le_bytes(bytes: [u8; 8]) -> Result<Self, DbError> {
        let value = f64::from_le_bytes(bytes);
        Self::try_new(value)
    }

    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<f64> for Float {
    type Error = DbError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for Float {
    type Error = DbError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parsed = s
            .parse::<f64>()
            .map_err(|e| DbError::type_mismatch(format!("invalid float: {e}")))?;
        Self::try_new(parsed)
    }
}
