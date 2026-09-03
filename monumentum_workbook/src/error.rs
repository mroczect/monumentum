use core::error::Error;
use core::fmt;

use monumentum_db::error::{DbError, ErrorKind, MonumentumError};
use monumentum_query::formula::FormulaError;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum WorkbookError {
    CellTooNarrow,
    FormatOutOfRange,
    NotAvailable,
    InvalidCharacter,
    InvalidArgument,
    InvalidFloatingPointOperation,
    ParameterListError,
    PairMissing,
    MissingOperator,
    MissingVariable,
    MissingVariableForFunction,
    FormulaOverflow,
    StringOverflow,
    InternalOverflow,
    InternalSyntaxError,
    MatrixExpected,
    UnknownCode,
    VariableNotAvailable,
    NoValue,
    Null,
    CircularReference,
    NoConvergence,
    InvalidReference,
    InvalidName,
    ReferenceTooEncapsulated,
    AddInNotFound,
    MacroNotFound,
    DivisionByZero,
    NestedArraysNotSupported,
    ArraySizeExceeded,
    UnsupportedInlineArrayContent,
    ExternalContentDisabled,
    Db(DbError),
    Formula(FormulaError),
    FileExists,
    InvalidExtension,
}

impl fmt::Display for WorkbookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellTooNarrow => write!(f, "###: cell too narrow"),
            Self::FormatOutOfRange => write!(f, "#FMT: value outside format limits"),
            Self::NotAvailable => write!(f, "#N/A: not available"),
            Self::InvalidCharacter => write!(f, "Err:501: invalid character"),
            Self::InvalidArgument => write!(f, "Err:502: invalid argument"),
            Self::InvalidFloatingPointOperation => {
                write!(f, "#NUM!: invalid floating point operation")
            }
            Self::ParameterListError => write!(f, "Err:504: parameter list error"),
            Self::PairMissing => write!(f, "Err:507/508: pair missing"),
            Self::MissingOperator => write!(f, "Err:509: missing operator"),
            Self::MissingVariable => write!(f, "Err:510: missing variable"),
            Self::MissingVariableForFunction => {
                write!(f, "Err:511: function requires more variables")
            }
            Self::FormulaOverflow => write!(f, "Err:512: formula overflow"),
            Self::StringOverflow => write!(f, "Err:513: string overflow"),
            Self::InternalOverflow => write!(f, "Err:514: internal overflow"),
            Self::InternalSyntaxError => write!(f, "Err:515: internal syntax error"),
            Self::MatrixExpected => write!(f, "Err:516: matrix expected"),
            Self::UnknownCode => write!(f, "Err:517: unknown code"),
            Self::VariableNotAvailable => write!(f, "Err:518: variable not available"),
            Self::NoValue => write!(f, "#VALUE!: no value"),
            Self::Null => write!(f, "#NULL!: null"),
            Self::CircularReference => write!(f, "Err:522: circular reference"),
            Self::NoConvergence => write!(f, "Err:523: no convergence"),
            Self::InvalidReference => write!(f, "#REF!: invalid reference"),
            Self::InvalidName => write!(f, "#NAME?: invalid names"),
            Self::ReferenceTooEncapsulated => write!(f, "Err:527: reference too encapsulated"),
            Self::AddInNotFound => write!(f, "Err:530: add-in not found"),
            Self::MacroNotFound => write!(f, "Err:531: macro not found"),
            Self::DivisionByZero => write!(f, "#DIV/0!: division by zero"),
            Self::NestedArraysNotSupported => write!(f, "Err:533: nested arrays not supported"),
            Self::ArraySizeExceeded => write!(f, "Err:538: array size exceeded"),
            Self::UnsupportedInlineArrayContent => {
                write!(f, "Err:539: unsupported inline array content")
            }
            Self::ExternalContentDisabled => write!(f, "Err:540: external content disabled"),
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Formula(e) => write!(f, "formula error: {e}"),
            Self::FileExists => write!(f, "file already exists"),
            Self::InvalidExtension => write!(f, "invalid file extension, expected .monumentum"),
        }
    }
}

impl Error for WorkbookError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Db(e) => Some(e),
            Self::Formula(e) => Some(e),
            Self::CellTooNarrow
            | Self::FormatOutOfRange
            | Self::NotAvailable
            | Self::InvalidCharacter
            | Self::InvalidArgument
            | Self::InvalidFloatingPointOperation
            | Self::ParameterListError
            | Self::PairMissing
            | Self::MissingOperator
            | Self::MissingVariable
            | Self::MissingVariableForFunction
            | Self::FormulaOverflow
            | Self::StringOverflow
            | Self::InternalOverflow
            | Self::InternalSyntaxError
            | Self::MatrixExpected
            | Self::UnknownCode
            | Self::VariableNotAvailable
            | Self::NoValue
            | Self::Null
            | Self::CircularReference
            | Self::NoConvergence
            | Self::InvalidReference
            | Self::InvalidName
            | Self::ReferenceTooEncapsulated
            | Self::AddInNotFound
            | Self::MacroNotFound
            | Self::DivisionByZero
            | Self::NestedArraysNotSupported
            | Self::ArraySizeExceeded
            | Self::UnsupportedInlineArrayContent
            | Self::ExternalContentDisabled
            | Self::FileExists
            | Self::InvalidExtension => None,
        }
    }
}

impl MonumentumError for WorkbookError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Db(e) => e.kind(),
            Self::Formula(e) => match e {
                FormulaError::DivisionByZero => ErrorKind::Other,
                FormulaError::TypeMismatch(_) => ErrorKind::TypeMismatch,
                FormulaError::InvalidReference(_) => ErrorKind::InvalidOperation,
                FormulaError::Parse(_)
                | FormulaError::Eval(_)
                | FormulaError::CircularReference(_)
                | FormulaError::UnknownFunction(_)
                | FormulaError::WrongArity(_)
                | FormulaError::Unsupported(_) => ErrorKind::Other,
            },
            Self::FileExists | Self::InvalidExtension => ErrorKind::InvalidOperation,
            Self::CircularReference => ErrorKind::Other,
            Self::InvalidReference | Self::InvalidName | Self::ReferenceTooEncapsulated => {
                ErrorKind::InvalidOperation
            }
            Self::InvalidArgument | Self::MissingOperator | Self::MissingVariable => {
                ErrorKind::InvalidOperation
            }
            Self::CellTooNarrow
            | Self::FormatOutOfRange
            | Self::NotAvailable
            | Self::InvalidCharacter
            | Self::InvalidFloatingPointOperation
            | Self::ParameterListError
            | Self::PairMissing
            | Self::MissingVariableForFunction
            | Self::FormulaOverflow
            | Self::StringOverflow
            | Self::InternalOverflow
            | Self::InternalSyntaxError
            | Self::MatrixExpected
            | Self::UnknownCode
            | Self::VariableNotAvailable
            | Self::NoValue
            | Self::Null
            | Self::NoConvergence
            | Self::AddInNotFound
            | Self::MacroNotFound
            | Self::DivisionByZero
            | Self::NestedArraysNotSupported
            | Self::ArraySizeExceeded
            | Self::UnsupportedInlineArrayContent
            | Self::ExternalContentDisabled => ErrorKind::Other,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Db(e) => e.message(),
            Self::Formula(e) => e.message(),
            Self::CellTooNarrow
            | Self::FormatOutOfRange
            | Self::NotAvailable
            | Self::InvalidCharacter
            | Self::InvalidArgument
            | Self::InvalidFloatingPointOperation
            | Self::ParameterListError
            | Self::PairMissing
            | Self::MissingOperator
            | Self::MissingVariable
            | Self::MissingVariableForFunction
            | Self::FormulaOverflow
            | Self::StringOverflow
            | Self::InternalOverflow
            | Self::InternalSyntaxError
            | Self::MatrixExpected
            | Self::UnknownCode
            | Self::VariableNotAvailable
            | Self::NoValue
            | Self::Null
            | Self::CircularReference
            | Self::NoConvergence
            | Self::InvalidReference
            | Self::InvalidName
            | Self::ReferenceTooEncapsulated
            | Self::AddInNotFound
            | Self::MacroNotFound
            | Self::DivisionByZero
            | Self::NestedArraysNotSupported
            | Self::ArraySizeExceeded
            | Self::UnsupportedInlineArrayContent
            | Self::ExternalContentDisabled
            | Self::FileExists
            | Self::InvalidExtension => "Workbook error",
        }
    }

    fn constraint(&self) -> Option<&str> {
        match self {
            Self::Db(e) => e.constraint(),
            Self::CellTooNarrow
            | Self::FormatOutOfRange
            | Self::NotAvailable
            | Self::InvalidCharacter
            | Self::InvalidArgument
            | Self::InvalidFloatingPointOperation
            | Self::ParameterListError
            | Self::PairMissing
            | Self::MissingOperator
            | Self::MissingVariable
            | Self::MissingVariableForFunction
            | Self::FormulaOverflow
            | Self::StringOverflow
            | Self::InternalOverflow
            | Self::InternalSyntaxError
            | Self::MatrixExpected
            | Self::UnknownCode
            | Self::VariableNotAvailable
            | Self::NoValue
            | Self::Null
            | Self::CircularReference
            | Self::NoConvergence
            | Self::InvalidReference
            | Self::InvalidName
            | Self::ReferenceTooEncapsulated
            | Self::AddInNotFound
            | Self::MacroNotFound
            | Self::DivisionByZero
            | Self::NestedArraysNotSupported
            | Self::ArraySizeExceeded
            | Self::UnsupportedInlineArrayContent
            | Self::ExternalContentDisabled
            | Self::Formula(_)
            | Self::FileExists
            | Self::InvalidExtension => None,
        }
    }

    fn table(&self) -> Option<&str> {
        match self {
            Self::Db(e) => e.table(),
            Self::CellTooNarrow
            | Self::FormatOutOfRange
            | Self::NotAvailable
            | Self::InvalidCharacter
            | Self::InvalidArgument
            | Self::InvalidFloatingPointOperation
            | Self::ParameterListError
            | Self::PairMissing
            | Self::MissingOperator
            | Self::MissingVariable
            | Self::MissingVariableForFunction
            | Self::FormulaOverflow
            | Self::StringOverflow
            | Self::InternalOverflow
            | Self::InternalSyntaxError
            | Self::MatrixExpected
            | Self::UnknownCode
            | Self::VariableNotAvailable
            | Self::NoValue
            | Self::Null
            | Self::CircularReference
            | Self::NoConvergence
            | Self::InvalidReference
            | Self::InvalidName
            | Self::ReferenceTooEncapsulated
            | Self::AddInNotFound
            | Self::MacroNotFound
            | Self::DivisionByZero
            | Self::NestedArraysNotSupported
            | Self::ArraySizeExceeded
            | Self::UnsupportedInlineArrayContent
            | Self::ExternalContentDisabled
            | Self::Formula(_)
            | Self::FileExists
            | Self::InvalidExtension => None,
        }
    }
}

impl From<DbError> for WorkbookError {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}

impl From<FormulaError> for WorkbookError {
    fn from(e: FormulaError) -> Self {
        Self::Formula(e)
    }
}
