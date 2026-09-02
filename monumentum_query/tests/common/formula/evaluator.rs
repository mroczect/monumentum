use monumentum_db::core::value::Value;
use monumentum_db::types::{Integer, Text};
use monumentum_query::coordinates::CellRef;
use monumentum_query::formula::{FormulaContext, FormulaError, evaluate, parse, tokenize};
use std::collections::HashMap;

struct DummyContext {
    cells: HashMap<String, Value>,
}

impl DummyContext {
    fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    fn set(&mut self, cell: &str, val: Value) {
        self.cells.insert(cell.to_string(), val);
    }
}

impl FormulaContext for DummyContext {
    fn get_cell_value(&self, cell: &CellRef) -> Result<Value, FormulaError> {
        let key = cell.to_string();
        self.cells
            .get(&key)
            .cloned()
            .ok_or_else(|| FormulaError::InvalidReference(format!("cell {key} not found")))
    }
}

fn eval_str(ctx: &DummyContext, input: &str) -> Result<Value, FormulaError> {
    let tokens = tokenize(input)?;
    let expr = parse(&tokens)?;
    evaluate(&expr, ctx)
}

fn eval_str_no_ctx(input: &str) -> Result<Value, FormulaError> {
    let ctx = DummyContext::new();
    eval_str(&ctx, input)
}

fn assert_integer(result: Result<Value, FormulaError>, expected: i64) {
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i.as_i64(), expected),
        other => panic!("expected integer {expected}, got {other:?}"),
    }
}

fn assert_float(result: Result<Value, FormulaError>, expected: f64) {
    match result {
        Ok(Value::Float(f)) => assert!((f.as_f64() - expected).abs() < f64::EPSILON),
        other => panic!("expected float {expected}, got {other:?}"),
    }
}

fn assert_text(result: Result<Value, FormulaError>, expected: &str) {
    match result {
        Ok(Value::Text(t)) => assert_eq!(t.as_str(), expected),
        other => panic!("expected text {expected}, got {other:?}"),
    }
}

fn assert_boolean(result: Result<Value, FormulaError>, expected: bool) {
    match result {
        Ok(Value::Boolean(b)) => assert_eq!(b, expected),
        other => panic!("expected boolean {expected}, got {other:?}"),
    }
}

fn assert_null(result: Result<Value, FormulaError>) {
    match result {
        Ok(Value::Null) => {}
        other => panic!("expected null, got {other:?}"),
    }
}

fn assert_error(result: Result<Value, FormulaError>, variant: FormulaError) {
    match result {
        Err(e) => assert_eq!(e, variant),
        other => panic!("expected error {variant:?}, got {other:?}"),
    }
}

#[test]
fn eval_integer_literal() {
    assert_integer(eval_str_no_ctx("42"), 42);
}

#[test]
fn eval_float_literal() {
    assert_float(eval_str_no_ctx("2.5"), 2.5);
}

#[test]
fn eval_string_literal() {
    assert_text(eval_str_no_ctx("\"hello\""), "hello");
}

#[test]
fn eval_boolean_true() {
    assert_boolean(eval_str_no_ctx("true"), true);
}

#[test]
fn eval_boolean_false() {
    assert_boolean(eval_str_no_ctx("false"), false);
}

#[test]
fn eval_null_literal() {
    assert_null(eval_str_no_ctx("null"));
}

#[test]
fn eval_negation_integer() {
    assert_integer(eval_str_no_ctx("-42"), -42);
}

#[test]
fn eval_negation_float() {
    assert_float(eval_str_no_ctx("-2.5"), -2.5);
}

#[test]
fn eval_negation_integer_min_parse_error() {
    assert!(eval_str_no_ctx("-9223372036854775808").is_err());
}

#[test]
fn eval_not_true() {
    assert_boolean(eval_str_no_ctx("!true"), false);
}

#[test]
fn eval_not_false() {
    assert_boolean(eval_str_no_ctx("!false"), true);
}

#[test]
fn eval_not_non_boolean_error() {
    assert_error(
        eval_str_no_ctx("!42"),
        FormulaError::TypeMismatch("cannot apply logical NOT to non-boolean value".to_string()),
    );
}

#[test]
fn eval_double_negation() {
    assert_integer(eval_str_no_ctx("--5"), 5);
}

#[test]
fn eval_add_integer() {
    assert_integer(eval_str_no_ctx("1 + 2"), 3);
}

#[test]
fn eval_sub_integer() {
    assert_integer(eval_str_no_ctx("5 - 3"), 2);
}

#[test]
fn eval_mul_integer() {
    assert_integer(eval_str_no_ctx("3 * 4"), 12);
}

#[test]
fn eval_div_integer() {
    assert_integer(eval_str_no_ctx("10 / 2"), 5);
}

#[test]
fn eval_div_integer_truncates() {
    assert_integer(eval_str_no_ctx("7 / 2"), 3);
}

#[test]
fn eval_mod_integer() {
    assert_integer(eval_str_no_ctx("7 % 3"), 1);
}

#[test]
fn eval_pow_integer() {
    assert_integer(eval_str_no_ctx("2 ^ 3"), 8);
}

#[test]
fn eval_arithmetic_precedence() {
    assert_integer(eval_str_no_ctx("1 + 2 * 3"), 7);
}

#[test]
fn eval_parentheses() {
    assert_integer(eval_str_no_ctx("(1 + 2) * 3"), 9);
}

#[test]
fn eval_float_add() {
    assert_float(eval_str_no_ctx("2.5 + 3.5"), 6.0);
}

#[test]
fn eval_float_sub() {
    assert_float(eval_str_no_ctx("5.0 - 2.0"), 3.0);
}

#[test]
fn eval_float_mul() {
    assert_float(eval_str_no_ctx("2.5 * 4.0"), 10.0);
}

#[test]
fn eval_float_div() {
    assert_float(eval_str_no_ctx("10.0 / 4.0"), 2.5);
}

#[test]
fn eval_float_pow() {
    assert_float(eval_str_no_ctx("2.0 ^ 3.0"), 8.0);
}

#[test]
fn eval_mixed_int_float_add() {
    assert_float(eval_str_no_ctx("2 + 3.5"), 5.5);
}

#[test]
fn eval_mixed_int_float_sub() {
    assert_float(eval_str_no_ctx("5 - 2.5"), 2.5);
}

#[test]
fn eval_mixed_int_float_mul() {
    assert_float(eval_str_no_ctx("2 * 2.5"), 5.0);
}

#[test]
fn eval_mixed_int_float_div() {
    assert_float(eval_str_no_ctx("5 / 2.0"), 2.5);
}

#[test]
fn eval_text_concat() {
    assert_text(eval_str_no_ctx("\"Hello\" + \" World\""), "Hello World");
}

#[test]
fn eval_add_type_mismatch() {
    assert_error(
        eval_str_no_ctx("1 + true"),
        FormulaError::TypeMismatch("cannot add values of these types".to_string()),
    );
}

#[test]
fn eval_sub_type_mismatch() {
    assert_error(
        eval_str_no_ctx("\"a\" - 1"),
        FormulaError::TypeMismatch("cannot subtract values of these types".to_string()),
    );
}

#[test]
fn eval_mul_type_mismatch() {
    assert_error(
        eval_str_no_ctx("2 * \"a\""),
        FormulaError::TypeMismatch("cannot multiply values of these types".to_string()),
    );
}

#[test]
fn eval_div_type_mismatch() {
    assert_error(
        eval_str_no_ctx("true / 2"),
        FormulaError::TypeMismatch("cannot divide values of these types".to_string()),
    );
}

#[test]
fn eval_division_by_zero_integer() {
    assert_error(eval_str_no_ctx("1 / 0"), FormulaError::DivisionByZero);
}

#[test]
fn eval_division_by_zero_float() {
    assert_error(eval_str_no_ctx("1.0 / 0.0"), FormulaError::DivisionByZero);
}

#[test]
fn eval_mod_by_zero_integer() {
    assert_error(eval_str_no_ctx("1 % 0"), FormulaError::DivisionByZero);
}

#[test]
fn eval_mod_by_zero_float() {
    assert_error(eval_str_no_ctx("1.0 % 0.0"), FormulaError::DivisionByZero);
}

#[test]
fn eval_integer_overflow_add() {
    let result = eval_str_no_ctx("9223372036854775807 + 1");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_integer_overflow_mul() {
    let result = eval_str_no_ctx("9223372036854775807 * 2");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_float_non_finite_result() {
    let result = eval_str_no_ctx("1e308 * 10");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_pow_negative_exponent_error() {
    let result = eval_str_no_ctx("2 ^ -1");
    assert!(result.is_err());
}

#[test]
fn eval_eq_integer() {
    assert_boolean(eval_str_no_ctx("5 == 5"), true);
}

#[test]
fn eval_neq_integer() {
    assert_boolean(eval_str_no_ctx("5 != 3"), true);
}

#[test]
fn eval_lt_integer() {
    assert_boolean(eval_str_no_ctx("2 < 3"), true);
}

#[test]
fn eval_lte_integer() {
    assert_boolean(eval_str_no_ctx("3 <= 3"), true);
}

#[test]
fn eval_gt_integer() {
    assert_boolean(eval_str_no_ctx("5 > 3"), true);
}

#[test]
fn eval_gte_integer() {
    assert_boolean(eval_str_no_ctx("3 >= 4"), false);
}

#[test]
fn eval_eq_text() {
    assert_boolean(eval_str_no_ctx("\"a\" == \"a\""), true);
}

#[test]
fn eval_neq_text() {
    assert_boolean(eval_str_no_ctx("\"a\" != \"b\""), true);
}

#[test]
fn eval_comparison_cross_type_returns_false_not_error() {
    assert_boolean(eval_str_no_ctx("1 == \"1\""), false);
}

#[test]
fn eval_comparison_text_vs_integer() {
    assert_boolean(eval_str_no_ctx("\"abc\" < 5"), false);
}

#[test]
fn eval_and_true_true() {
    assert_boolean(eval_str_no_ctx("true && true"), true);
}

#[test]
fn eval_and_true_false() {
    assert_boolean(eval_str_no_ctx("true && false"), false);
}

#[test]
fn eval_or_false_true() {
    assert_boolean(eval_str_no_ctx("false || true"), true);
}

#[test]
fn eval_or_false_false() {
    assert_boolean(eval_str_no_ctx("false || false"), false);
}

#[test]
fn eval_and_non_boolean_error() {
    assert_error(
        eval_str_no_ctx("true && 1"),
        FormulaError::TypeMismatch("AND requires boolean operands".to_string()),
    );
}

#[test]
fn eval_or_non_boolean_error() {
    assert_error(
        eval_str_no_ctx("1 || false"),
        FormulaError::TypeMismatch("OR requires boolean operands".to_string()),
    );
}

#[test]
fn eval_cell_reference() {
    let mut ctx = DummyContext::new();
    ctx.set("A1", Value::Integer(Integer::new(10)));
    assert_integer(eval_str(&ctx, "A1"), 10);
}

#[test]
fn eval_cell_reference_in_expression() {
    let mut ctx = DummyContext::new();
    ctx.set("A1", Value::Integer(Integer::new(10)));
    assert_integer(eval_str(&ctx, "A1 + 5"), 15);
}

#[test]
fn eval_missing_cell_reference_error() {
    let ctx = DummyContext::new();
    assert_error(
        eval_str(&ctx, "A1"),
        FormulaError::InvalidReference("cell A1 not found".to_string()),
    );
}

#[test]
fn eval_sheet_cell_reference() {
    let mut ctx = DummyContext::new();
    ctx.set("Sheet2!C3", Value::Text(Text::new("hi".to_string())));
    assert_text(eval_str(&ctx, "Sheet2!C3"), "hi");
}

#[test]
fn eval_range_in_function_sum() {
    let mut ctx = DummyContext::new();
    ctx.set("A1", Value::Integer(Integer::new(1)));
    ctx.set("A2", Value::Integer(Integer::new(2)));
    ctx.set("A3", Value::Integer(Integer::new(3)));
    assert_integer(eval_str(&ctx, "SUM(A1:A3)"), 6);
}

#[test]
fn eval_range_not_allowed_scalar() {
    let ctx = DummyContext::new();
    assert_error(
        eval_str(&ctx, "A1:A3"),
        FormulaError::Eval("range not allowed in scalar context".to_string()),
    );
}

#[test]
fn eval_sum_empty_args_error() {
    let result = eval_str_no_ctx("SUM()");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_sum_integer() {
    assert_integer(eval_str_no_ctx("SUM(1, 2, 3)"), 6);
}

#[test]
fn eval_sum_float() {
    assert_float(eval_str_no_ctx("SUM(1.5, 2.5)"), 4.0);
}

#[test]
fn eval_sum_mixed_types() {
    assert_float(eval_str_no_ctx("SUM(1, 2.5, 3)"), 6.5);
}

#[test]
fn eval_sum_type_mismatch() {
    assert_error(
        eval_str_no_ctx("SUM(1, \"a\")"),
        FormulaError::TypeMismatch("SUM expects numeric values".to_string()),
    );
}

#[test]
fn eval_average_integer() {
    assert_float(eval_str_no_ctx("AVERAGE(1, 2, 3)"), 2.0);
}

#[test]
fn eval_average_single() {
    assert_float(eval_str_no_ctx("AVG(5)"), 5.0);
}

#[test]
fn eval_average_empty_error() {
    let result = eval_str_no_ctx("AVERAGE()");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_min_integer() {
    assert_integer(eval_str_no_ctx("MIN(5, 2, 8)"), 2);
}

#[test]
fn eval_max_integer() {
    assert_integer(eval_str_no_ctx("MAX(5, 2, 8)"), 8);
}

#[test]
fn eval_min_empty_error() {
    let result = eval_str_no_ctx("MIN()");
    assert!(result.is_err());
    match result {
        Err(FormulaError::Eval(_)) => {}
        other => panic!("expected Eval error, got {other:?}"),
    }
}

#[test]
fn eval_if_true() {
    assert_integer(eval_str_no_ctx("IF(true, 1, 2)"), 1);
}

#[test]
fn eval_if_false() {
    assert_integer(eval_str_no_ctx("IF(false, 1, 2)"), 2);
}

#[test]
fn eval_if_wrong_arity() {
    assert_error(
        eval_str_no_ctx("IF(true, 1)"),
        FormulaError::WrongArity("IF expects 3 arguments".to_string()),
    );
}

#[test]
fn eval_if_non_boolean_condition() {
    assert_error(
        eval_str_no_ctx("IF(1, 2, 3)"),
        FormulaError::TypeMismatch("IF condition must be boolean".to_string()),
    );
}

#[test]
fn eval_and_function_all_true() {
    assert_boolean(eval_str_no_ctx("AND(true, true, true)"), true);
}

#[test]
fn eval_and_function_false_short_circuit() {
    assert_boolean(eval_str_no_ctx("AND(true, false, true)"), false);
}

#[test]
fn eval_or_function_false_true() {
    assert_boolean(eval_str_no_ctx("OR(false, true, false)"), true);
}

#[test]
fn eval_or_function_all_false() {
    assert_boolean(eval_str_no_ctx("OR(false, false)"), false);
}

#[test]
fn eval_not_function() {
    assert_boolean(eval_str_no_ctx("NOT(false)"), true);
}

#[test]
fn eval_not_function_wrong_arity() {
    assert_error(
        eval_str_no_ctx("NOT(true, false)"),
        FormulaError::WrongArity("NOT expects 1 argument".to_string()),
    );
}

#[test]
fn eval_concat_text() {
    assert_text(
        eval_str_no_ctx("CONCAT(\"Hello\", \" \", \"World\")"),
        "Hello World",
    );
}

#[test]
fn eval_concat_with_numbers() {
    assert_text(eval_str_no_ctx("CONCAT(1, 2.5, true)"), "12.5true");
}

#[test]
fn eval_trim() {
    assert_text(eval_str_no_ctx("TRIM(\"  hello  \")"), "hello");
}

#[test]
fn eval_upper() {
    assert_text(eval_str_no_ctx("UPPER(\"hello\")"), "HELLO");
}

#[test]
fn eval_lower() {
    assert_text(eval_str_no_ctx("LOWER(\"HELLO\")"), "hello");
}

#[test]
fn eval_len() {
    assert_integer(eval_str_no_ctx("LEN(\"hello\")"), 5);
}

#[test]
fn eval_unknown_function() {
    assert_error(
        eval_str_no_ctx("FOO(1)"),
        FormulaError::UnknownFunction("FOO".to_string()),
    );
}

#[test]
fn eval_nested_function_calls() {
    assert_integer(eval_str_no_ctx("MAX(MIN(1, 2), 3)"), 3);
}

#[test]
fn eval_function_with_expression_arg() {
    assert_integer(eval_str_no_ctx("SUM(1 + 2, 3 * 2)"), 9);
}

#[test]
fn eval_complex_expression() {
    assert_boolean(eval_str_no_ctx("(1 + 2) * (3 - 1) > 4"), true);
}
