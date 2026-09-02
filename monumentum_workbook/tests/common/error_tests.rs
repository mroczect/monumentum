use monumentum_db::error::DbError;
use monumentum_workbook::WorkbookError;

const fn assert_std_error<T: core::error::Error>() {}
const fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn implements_std_error() {
    assert_std_error::<WorkbookError>();
}

#[test]
fn is_send_and_sync() {
    assert_send_sync::<WorkbookError>();
}

#[test]
fn clone_works() {
    let err = WorkbookError::InvalidReference;
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn equality_works() {
    assert_eq!(WorkbookError::DivisionByZero, WorkbookError::DivisionByZero);
    assert_ne!(WorkbookError::DivisionByZero, WorkbookError::Null);
}

#[test]
fn debug_contains_variant_name() {
    let err = WorkbookError::InvalidName;
    let debug = format!("{err:?}");
    assert!(debug.contains("InvalidName"));
}

#[test]
fn display_cell_too_narrow() {
    assert_eq!(
        format!("{}", WorkbookError::CellTooNarrow),
        "###: cell too narrow"
    );
}

#[test]
fn display_format_out_of_range() {
    assert_eq!(
        format!("{}", WorkbookError::FormatOutOfRange),
        "#FMT: value outside format limits"
    );
}

#[test]
fn display_not_available() {
    assert_eq!(
        format!("{}", WorkbookError::NotAvailable),
        "#N/A: not available"
    );
}

#[test]
fn display_invalid_character() {
    assert_eq!(
        format!("{}", WorkbookError::InvalidCharacter),
        "Err:501: invalid character"
    );
}

#[test]
fn display_invalid_argument() {
    assert_eq!(
        format!("{}", WorkbookError::InvalidArgument),
        "Err:502: invalid argument"
    );
}

#[test]
fn display_invalid_floating_point_operation() {
    assert_eq!(
        format!("{}", WorkbookError::InvalidFloatingPointOperation),
        "#NUM!: invalid floating point operation"
    );
}

#[test]
fn display_parameter_list_error() {
    assert_eq!(
        format!("{}", WorkbookError::ParameterListError),
        "Err:504: parameter list error"
    );
}

#[test]
fn display_pair_missing() {
    assert_eq!(
        format!("{}", WorkbookError::PairMissing),
        "Err:507/508: pair missing"
    );
}

#[test]
fn display_missing_operator() {
    assert_eq!(
        format!("{}", WorkbookError::MissingOperator),
        "Err:509: missing operator"
    );
}

#[test]
fn display_missing_variable() {
    assert_eq!(
        format!("{}", WorkbookError::MissingVariable),
        "Err:510: missing variable"
    );
}

#[test]
fn display_missing_variable_for_function() {
    assert_eq!(
        format!("{}", WorkbookError::MissingVariableForFunction),
        "Err:511: function requires more variables"
    );
}

#[test]
fn display_formula_overflow() {
    assert_eq!(
        format!("{}", WorkbookError::FormulaOverflow),
        "Err:512: formula overflow"
    );
}

#[test]
fn display_string_overflow() {
    assert_eq!(
        format!("{}", WorkbookError::StringOverflow),
        "Err:513: string overflow"
    );
}

#[test]
fn display_internal_overflow() {
    assert_eq!(
        format!("{}", WorkbookError::InternalOverflow),
        "Err:514: internal overflow"
    );
}

#[test]
fn display_internal_syntax_error() {
    assert_eq!(
        format!("{}", WorkbookError::InternalSyntaxError),
        "Err:515: internal syntax error"
    );
}

#[test]
fn display_matrix_expected() {
    assert_eq!(
        format!("{}", WorkbookError::MatrixExpected),
        "Err:516: matrix expected"
    );
}

#[test]
fn display_unknown_code() {
    assert_eq!(
        format!("{}", WorkbookError::UnknownCode),
        "Err:517: unknown code"
    );
}

#[test]
fn display_variable_not_available() {
    assert_eq!(
        format!("{}", WorkbookError::VariableNotAvailable),
        "Err:518: variable not available"
    );
}

#[test]
fn display_no_value() {
    assert_eq!(format!("{}", WorkbookError::NoValue), "#VALUE!: no value");
}

#[test]
fn display_null() {
    assert_eq!(format!("{}", WorkbookError::Null), "#NULL!: null");
}

#[test]
fn display_circular_reference() {
    assert_eq!(
        format!("{}", WorkbookError::CircularReference),
        "Err:522: circular reference"
    );
}

#[test]
fn display_no_convergence() {
    assert_eq!(
        format!("{}", WorkbookError::NoConvergence),
        "Err:523: no convergence"
    );
}

#[test]
fn display_invalid_reference() {
    assert_eq!(
        format!("{}", WorkbookError::InvalidReference),
        "#REF!: invalid reference"
    );
}

#[test]
fn display_invalid_name() {
    assert_eq!(
        format!("{}", WorkbookError::InvalidName),
        "#NAME?: invalid names"
    );
}

#[test]
fn display_reference_too_encapsulated() {
    assert_eq!(
        format!("{}", WorkbookError::ReferenceTooEncapsulated),
        "Err:527: reference too encapsulated"
    );
}

#[test]
fn display_addin_not_found() {
    assert_eq!(
        format!("{}", WorkbookError::AddInNotFound),
        "Err:530: add-in not found"
    );
}

#[test]
fn display_macro_not_found() {
    assert_eq!(
        format!("{}", WorkbookError::MacroNotFound),
        "Err:531: macro not found"
    );
}

#[test]
fn display_division_by_zero() {
    assert_eq!(
        format!("{}", WorkbookError::DivisionByZero),
        "#DIV/0!: division by zero"
    );
}

#[test]
fn display_nested_arrays_not_supported() {
    assert_eq!(
        format!("{}", WorkbookError::NestedArraysNotSupported),
        "Err:533: nested arrays not supported"
    );
}

#[test]
fn display_array_size_exceeded() {
    assert_eq!(
        format!("{}", WorkbookError::ArraySizeExceeded),
        "Err:538: array size exceeded"
    );
}

#[test]
fn display_unsupported_inline_array_content() {
    assert_eq!(
        format!("{}", WorkbookError::UnsupportedInlineArrayContent),
        "Err:539: unsupported inline array content"
    );
}

#[test]
fn display_external_content_disabled() {
    assert_eq!(
        format!("{}", WorkbookError::ExternalContentDisabled),
        "Err:540: external content disabled"
    );
}

#[test]
fn display_db_error() {
    let err = WorkbookError::Db("custom message".to_string());
    assert_eq!(format!("{err}"), "database error: custom message");
}

#[test]
fn from_db_error_table_not_found() {
    let db_err = DbError::table_not_found("users");
    let wb_err = WorkbookError::from(db_err);
    assert_eq!(
        format!("{wb_err}"),
        "database error: Table not found: users"
    );
}

#[test]
fn from_db_error_type_mismatch() {
    let db_err = DbError::type_mismatch("expected integer");
    let wb_err = WorkbookError::from(db_err);
    assert_eq!(
        format!("{wb_err}"),
        "database error: Type mismatch: expected integer"
    );
}

#[test]
fn from_db_error_invalid_operation() {
    let db_err = DbError::invalid_operation("bad operation");
    let wb_err = WorkbookError::from(db_err);
    assert_eq!(
        format!("{wb_err}"),
        "database error: Invalid operation: bad operation"
    );
}
