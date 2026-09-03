use core::fmt;

use monumentum_db::error::{ErrorKind, MonumentumError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaError {
    Parse(String),
    Eval(String),
    CircularReference(String),
    InvalidReference(String),
    DivisionByZero,
    TypeMismatch(String),
    UnknownFunction(String),
    WrongArity(String),
    Unsupported(String),
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::Eval(msg) => write!(f, "evaluation error: {msg}"),
            Self::CircularReference(msg) => write!(f, "circular reference: {msg}"),
            Self::InvalidReference(msg) => write!(f, "invalid reference: {msg}"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::TypeMismatch(msg) => write!(f, "type mismatch: {msg}"),
            Self::UnknownFunction(name) => write!(f, "unknown function: {name}"),
            Self::WrongArity(msg) => write!(f, "wrong number of arguments: {msg}"),
            Self::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for FormulaError {}

impl MonumentumError for FormulaError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Parse(_) | Self::Eval(_) | Self::Unsupported(_) => ErrorKind::Other,
            Self::CircularReference(_) | Self::InvalidReference(_) => ErrorKind::InvalidOperation,
            Self::DivisionByZero => ErrorKind::Other,
            Self::TypeMismatch(_) => ErrorKind::TypeMismatch,
            Self::UnknownFunction(_) | Self::WrongArity(_) => ErrorKind::InvalidOperation,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Parse(msg)
            | Self::Eval(msg)
            | Self::CircularReference(msg)
            | Self::InvalidReference(msg)
            | Self::TypeMismatch(msg)
            | Self::WrongArity(msg)
            | Self::Unsupported(msg) => msg.as_str(),
            Self::UnknownFunction(name) => name.as_str(),
            Self::DivisionByZero => "division by zero",
        }
    }
}
