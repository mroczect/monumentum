use monumentum_query::formula::FormulaError;

#[test]
fn display_parse_error() {
    let err = FormulaError::Parse("unexpected token".to_string());
    assert_eq!(format!("{err}"), "parse error: unexpected token");
}

#[test]
fn display_eval_error() {
    let err = FormulaError::Eval("failed to evaluate".to_string());
    assert_eq!(format!("{err}"), "evaluation error: failed to evaluate");
}

#[test]
fn display_circular_reference() {
    let err = FormulaError::CircularReference("A1".to_string());
    assert_eq!(format!("{err}"), "circular reference: A1");
}

#[test]
fn display_invalid_reference() {
    let err = FormulaError::InvalidReference("XFE1".to_string());
    assert_eq!(format!("{err}"), "invalid reference: XFE1");
}

#[test]
fn display_division_by_zero() {
    let err = FormulaError::DivisionByZero;
    assert_eq!(format!("{err}"), "division by zero");
}

#[test]
fn display_type_mismatch() {
    let err = FormulaError::TypeMismatch("expected integer".to_string());
    assert_eq!(format!("{err}"), "type mismatch: expected integer");
}

#[test]
fn display_unknown_function() {
    let err = FormulaError::UnknownFunction("FOO".to_string());
    assert_eq!(format!("{err}"), "unknown function: FOO");
}

#[test]
fn display_wrong_arity() {
    let err = FormulaError::WrongArity("SUM expects 1..N arguments".to_string());
    assert_eq!(
        format!("{err}"),
        "wrong number of arguments: SUM expects 1..N arguments"
    );
}

#[test]
fn display_unsupported() {
    let err = FormulaError::Unsupported("nested formulas".to_string());
    assert_eq!(format!("{err}"), "unsupported: nested formulas");
}

#[test]
fn equality_works() {
    assert_eq!(FormulaError::DivisionByZero, FormulaError::DivisionByZero);
    assert_ne!(
        FormulaError::DivisionByZero,
        FormulaError::Parse("division by zero".to_string())
    );
}

#[test]
fn clone_works() {
    let err = FormulaError::InvalidReference("bad".to_string());
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn debug_format_contains_variant_name() {
    let err = FormulaError::DivisionByZero;
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("DivisionByZero"));
}

#[test]
fn implements_std_error() {
    fn assert_error<T: std::error::Error>() {}
    assert_error::<FormulaError>();
}

#[test]
fn is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FormulaError>();
}
