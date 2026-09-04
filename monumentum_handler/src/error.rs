use alloc::sync::Arc;
use core::error::Error;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    UniqueViolation,
    ForeignKeyViolation,
    NotNullViolation,
    CheckViolation,
    TypeMismatch,
    InvalidOperation,
    InvalidQuery,
    Io,
    Corruption,
    Unsupported,
    Other,
}

pub trait MonumentumError: Error + Send + Sync {
    fn kind(&self) -> ErrorKind;
    fn message(&self) -> &str;
    fn constraint(&self) -> Option<&str> {
        None
    }
    fn table(&self) -> Option<&str> {
        None
    }

    fn is_unique_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::UniqueViolation)
    }
    fn is_foreign_key_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::ForeignKeyViolation)
    }
    fn is_not_null_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::NotNullViolation)
    }
    fn is_check_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::CheckViolation)
    }
    fn is_type_mismatch(&self) -> bool {
        matches!(self.kind(), ErrorKind::TypeMismatch)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DbError {
    Io(Arc<std::io::Error>),
    Corruption(Arc<dyn Error + Send + Sync>),
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    InvalidOperation(String),
    InvalidQuery(String),
    Transaction(Arc<dyn Error + Send + Sync>),
    Unsupported(String),
    ConstraintViolation {
        kind: ErrorKind,
        message: String,
        constraint: Option<String>,
        table: Option<String>,
    },
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
        Self::Corruption(Arc::new(err))
    }

    #[must_use]
    pub fn transaction<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::Transaction(Arc::new(err))
    }

    #[must_use]
    pub fn constraint_violation(
        kind: ErrorKind,
        message: impl Into<String>,
        constraint: Option<String>,
        table: Option<String>,
    ) -> Self {
        Self::ConstraintViolation {
            kind,
            message: message.into(),
            constraint,
            table,
        }
    }

    #[must_use]
    pub fn from_io(err: std::io::Error) -> Self {
        Self::Io(Arc::new(err))
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
            Self::ConstraintViolation { message, .. } => {
                write!(f, "Constraint violation: {message}")
            }
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e.as_ref()),
            Self::Corruption(e) | Self::Transaction(e) => Some(e.as_ref()),
            Self::TableNotFound(_)
            | Self::ColumnNotFound(_)
            | Self::TypeMismatch(_)
            | Self::InvalidOperation(_)
            | Self::InvalidQuery(_)
            | Self::Unsupported(_)
            | Self::ConstraintViolation { .. } => None,
        }
    }
}

impl MonumentumError for DbError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(_) => ErrorKind::Io,
            Self::Corruption(_) => ErrorKind::Corruption,
            Self::TableNotFound(_) | Self::ColumnNotFound(_) => ErrorKind::InvalidOperation,
            Self::TypeMismatch(_) => ErrorKind::TypeMismatch,
            Self::InvalidOperation(_) => ErrorKind::InvalidOperation,
            Self::InvalidQuery(_) => ErrorKind::InvalidQuery,
            Self::Transaction(_) => ErrorKind::Other,
            Self::Unsupported(_) => ErrorKind::Unsupported,
            Self::ConstraintViolation { kind, .. } => *kind,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Io(_) => "I/O error",
            Self::Corruption(_) => "Data corruption",
            Self::TableNotFound(name) => name.as_str(),
            Self::ColumnNotFound(name) => name.as_str(),
            Self::TypeMismatch(msg)
            | Self::InvalidOperation(msg)
            | Self::InvalidQuery(msg)
            | Self::Unsupported(msg) => msg.as_str(),
            Self::Transaction(_) => "Transaction error",
            Self::ConstraintViolation { message, .. } => message.as_str(),
        }
    }

    fn constraint(&self) -> Option<&str> {
        match self {
            Self::ConstraintViolation { constraint, .. } => constraint.as_deref(),
            Self::Io(_)
            | Self::Corruption(_)
            | Self::TableNotFound(_)
            | Self::ColumnNotFound(_)
            | Self::TypeMismatch(_)
            | Self::InvalidOperation(_)
            | Self::InvalidQuery(_)
            | Self::Transaction(_)
            | Self::Unsupported(_) => None,
        }
    }

    fn table(&self) -> Option<&str> {
        match self {
            Self::ConstraintViolation { table, .. } => table.as_deref(),
            Self::TableNotFound(name) => Some(name.as_str()),
            Self::Io(_)
            | Self::Corruption(_)
            | Self::ColumnNotFound(_)
            | Self::TypeMismatch(_)
            | Self::InvalidOperation(_)
            | Self::InvalidQuery(_)
            | Self::Transaction(_)
            | Self::Unsupported(_) => None,
        }
    }
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind() && a.to_string() == b.to_string(),
            (Self::Corruption(a), Self::Corruption(b)) => a.to_string() == b.to_string(),
            (Self::TableNotFound(a), Self::TableNotFound(b)) => a == b,
            (Self::ColumnNotFound(a), Self::ColumnNotFound(b)) => a == b,
            (Self::TypeMismatch(a), Self::TypeMismatch(b)) => a == b,
            (Self::InvalidOperation(a), Self::InvalidOperation(b)) => a == b,
            (Self::InvalidQuery(a), Self::InvalidQuery(b)) => a == b,
            (Self::Transaction(a), Self::Transaction(b)) => a.to_string() == b.to_string(),
            (Self::Unsupported(a), Self::Unsupported(b)) => a == b,
            (
                Self::ConstraintViolation {
                    kind: k1,
                    message: m1,
                    constraint: c1,
                    table: t1,
                },
                Self::ConstraintViolation {
                    kind: k2,
                    message: m2,
                    constraint: c2,
                    table: t2,
                },
            ) => k1 == k2 && m1 == m2 && c1 == c2 && t1 == t2,
            _ => false,
        }
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        Self::from_io(e)
    }
}
