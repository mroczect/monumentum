use core::error::Error;
use core::fmt;
use monumentum_db::error::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Db(String),
    Formula(String),
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
            Self::Db(msg) => write!(f, "database error: {msg}"),
            Self::Formula(msg) => write!(f, "formula error: {msg}"),
            Self::FileExists => write!(f, "file already exists"),
            Self::InvalidExtension => write!(f, "invalid file extension, expected .monumentum"),
        }
    }
}

impl Error for WorkbookError {}

impl From<DbError> for WorkbookError {
    fn from(e: DbError) -> Self {
        Self::Db(e.to_string())
    }
}

impl From<monumentum_query::formula::FormulaError> for WorkbookError {
    fn from(e: monumentum_query::formula::FormulaError) -> Self {
        Self::Formula(e.to_string())
    }
}
