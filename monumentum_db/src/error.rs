#![allow(clippy::std_instead_of_core)]

use core::error::Error;
use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum DbError {
    Io(std::io::Error),
    Corruption(Box<dyn Error + Send + Sync>),
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    InvalidOperation(String),
    InvalidQuery(String),
    Transaction(Box<dyn Error + Send + Sync>),
    Unsupported(String),
}

impl DbError {
    #[must_use]
    pub fn table_not_found(name: impl Into<String>) -> Self {
        Self::TableNotFound(name.into())
    }

    #[must_use]
    pub fn column_not_found(name: impl Into<String>) -> Self {
        Self::ColumnNotFound(name.into())
    }

    #[must_use]
    pub fn type_mismatch(msg: impl Into<String>) -> Self {
        Self::TypeMismatch(msg.into())
    }

    #[must_use]
    pub fn invalid_operation(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }

    #[must_use]
    pub fn invalid_query(msg: impl Into<String>) -> Self {
        Self::InvalidQuery(msg.into())
    }

    #[must_use]
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    #[must_use]
    pub fn corruption<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::Corruption(Box::new(err))
    }

    #[must_use]
    pub fn transaction<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::Transaction(Box::new(err))
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Corruption(e) => write!(f, "Data corruption: {e}"),
            Self::TableNotFound(name) => write!(f, "Table not found: {name}"),
            Self::ColumnNotFound(name) => write!(f, "Column not found: {name}"),
            Self::TypeMismatch(msg) => write!(f, "Type mismatch: {msg}"),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            Self::InvalidQuery(msg) => write!(f, "Invalid query: {msg}"),
            Self::Transaction(e) => write!(f, "Transaction error: {e}"),
            Self::Unsupported(msg) => write!(f, "Unsupported: {msg}"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Corruption(e) => Some(e.as_ref()),
            Self::Transaction(e) => Some(e.as_ref()),
            Self::TableNotFound(_)
            | Self::ColumnNotFound(_)
            | Self::TypeMismatch(_)
            | Self::InvalidOperation(_)
            | Self::InvalidQuery(_)
            | Self::Unsupported(_) => None,
        }
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
