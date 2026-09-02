use monumentum_db::core::value::Value;
use monumentum_query::formula::{BinaryOp, Expr, UnaryOp, parse, tokenize};

fn parse_str(input: &str) -> Result<Expr, monumentum_query::formula::FormulaError> {
    let tokens = tokenize(input)?;
    parse(&tokens)
}

#[test]
fn parse_integer_literal() {
    let expr = parse_str("42").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Integer(42.into())));
}

#[test]
fn parse_integer_negative_literal() {
    let expr = parse_str("-42").unwrap();
    match expr {
        Expr::UnaryOp(UnaryOp::Neg, operand) => {
            assert_eq!(*operand, Expr::Literal(Value::Integer(42.into())));
        }
        other => panic!("expected unary negation, got {other:?}"),
    }
}

#[test]
fn parse_integer_zero() {
    let expr = parse_str("0").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Integer(0.into())));
}

#[test]
fn parse_float_literal() {
    let expr = parse_str("2.5").unwrap();
    match expr {
        Expr::Literal(Value::Float(f)) => assert_eq!(f.as_f64(), 2.5),
        other => panic!("expected float literal, got {other:?}"),
    }
}

#[test]
fn parse_float_scientific_notation() {
    let expr = parse_str("1e3").unwrap();
    match expr {
        Expr::Literal(Value::Float(f)) => assert_eq!(f.as_f64(), 1000.0),
        other => panic!("expected float literal, got {other:?}"),
    }
}

#[test]
fn parse_float_overflow_returns_error() {
    let result = parse_str("1e999");
    assert!(result.is_err());
}

#[test]
fn parse_string_literal() {
    let expr = parse_str("\"hello\"").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Text("hello".to_string().into())));
}

#[test]
fn parse_boolean_true() {
    let expr = parse_str("true").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Boolean(true)));
}

#[test]
fn parse_boolean_false() {
    let expr = parse_str("false").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Boolean(false)));
}

#[test]
fn parse_null() {
    let expr = parse_str("null").unwrap();
    assert_eq!(expr, Expr::Literal(Value::Null));
}

#[test]
fn parse_simple_addition() {
    let expr = parse_str("1 + 2").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Add, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Integer(1.into())));
            assert_eq!(*rhs, Expr::Literal(Value::Integer(2.into())));
        }
        other => panic!("expected addition, got {other:?}"),
    }
}

#[test]
fn parse_simple_subtraction() {
    let expr = parse_str("5 - 3").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Sub, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Integer(5.into())));
            assert_eq!(*rhs, Expr::Literal(Value::Integer(3.into())));
        }
        other => panic!("expected subtraction, got {other:?}"),
    }
}

#[test]
fn parse_multiplication_and_division_precedence() {
    let expr = parse_str("2 * 3 + 4 / 2").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Add, lhs, rhs) => {
            match *lhs {
                Expr::BinaryOp(BinaryOp::Mul, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Integer(2.into())));
                    assert_eq!(*r, Expr::Literal(Value::Integer(3.into())));
                }
                other => panic!("expected multiplication, got {other:?}"),
            }
            match *rhs {
                Expr::BinaryOp(BinaryOp::Div, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Integer(4.into())));
                    assert_eq!(*r, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected division, got {other:?}"),
            }
        }
        other => panic!("expected addition, got {other:?}"),
    }
}

#[test]
fn parse_modulo() {
    let expr = parse_str("7 % 3").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Mod, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Integer(7.into())));
            assert_eq!(*rhs, Expr::Literal(Value::Integer(3.into())));
        }
        other => panic!("expected modulo, got {other:?}"),
    }
}

#[test]
fn parse_power_right_associative() {
    let expr = parse_str("2 ^ 3 ^ 2").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Pow, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Integer(2.into())));
            match *rhs {
                Expr::BinaryOp(BinaryOp::Pow, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Integer(3.into())));
                    assert_eq!(*r, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected inner power, got {other:?}"),
            }
        }
        other => panic!("expected power, got {other:?}"),
    }
}

#[test]
fn parse_multiplication_before_power() {
    let expr = parse_str("2 * 3 ^ 2").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Mul, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Integer(2.into())));
            match *rhs {
                Expr::BinaryOp(BinaryOp::Pow, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Integer(3.into())));
                    assert_eq!(*r, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected power, got {other:?}"),
            }
        }
        other => panic!("expected multiplication, got {other:?}"),
    }
}

#[test]
fn parse_parentheses_override_precedence() {
    let expr = parse_str("(1 + 2) * 3").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Mul, lhs, rhs) => {
            match *lhs {
                Expr::BinaryOp(BinaryOp::Add, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Integer(1.into())));
                    assert_eq!(*r, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected addition, got {other:?}"),
            }
            assert_eq!(*rhs, Expr::Literal(Value::Integer(3.into())));
        }
        other => panic!("expected multiplication, got {other:?}"),
    }
}

#[test]
fn parse_nested_parentheses() {
    let expr = parse_str("((1 + 2) * 3) + 4").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Add, lhs, rhs) => {
            assert_eq!(*rhs, Expr::Literal(Value::Integer(4.into())));
            match *lhs {
                Expr::BinaryOp(BinaryOp::Mul, l, r) => {
                    match *l {
                        Expr::BinaryOp(BinaryOp::Add, ll, rr) => {
                            assert_eq!(*ll, Expr::Literal(Value::Integer(1.into())));
                            assert_eq!(*rr, Expr::Literal(Value::Integer(2.into())));
                        }
                        other => panic!("expected addition inside parens, got {other:?}"),
                    }
                    assert_eq!(*r, Expr::Literal(Value::Integer(3.into())));
                }
                other => panic!("expected multiplication, got {other:?}"),
            }
        }
        other => panic!("expected addition, got {other:?}"),
    }
}

#[test]
fn parse_unary_not() {
    let expr = parse_str("!true").unwrap();
    match expr {
        Expr::UnaryOp(UnaryOp::Not, operand) => {
            assert_eq!(*operand, Expr::Literal(Value::Boolean(true)));
        }
        other => panic!("expected unary not, got {other:?}"),
    }
}

#[test]
fn parse_double_negation() {
    let expr = parse_str("--5").unwrap();
    match expr {
        Expr::UnaryOp(UnaryOp::Neg, operand) => match *operand {
            Expr::UnaryOp(UnaryOp::Neg, inner) => {
                assert_eq!(*inner, Expr::Literal(Value::Integer(5.into())));
            }
            other => panic!("expected inner negation, got {other:?}"),
        },
        other => panic!("expected outer negation, got {other:?}"),
    }
}

#[test]
fn parse_unary_precedence_over_multiplication() {
    let expr = parse_str("-2 * 3").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Mul, lhs, rhs) => {
            match *lhs {
                Expr::UnaryOp(UnaryOp::Neg, operand) => {
                    assert_eq!(*operand, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected negation, got {other:?}"),
            }
            assert_eq!(*rhs, Expr::Literal(Value::Integer(3.into())));
        }
        other => panic!("expected multiplication, got {other:?}"),
    }
}

#[test]
fn parse_comparison_operators() {
    let cases = [
        ("1 < 2", BinaryOp::Lt),
        ("1 <= 2", BinaryOp::Lte),
        ("1 > 2", BinaryOp::Gt),
        ("1 >= 2", BinaryOp::Gte),
        ("1 == 2", BinaryOp::Eq),
        ("1 != 2", BinaryOp::NotEq),
    ];
    for (input, expected_op) in cases {
        let expr = parse_str(input).unwrap();
        match expr {
            Expr::BinaryOp(op, lhs, rhs) => {
                assert_eq!(op, expected_op);
                assert_eq!(*lhs, Expr::Literal(Value::Integer(1.into())));
                assert_eq!(*rhs, Expr::Literal(Value::Integer(2.into())));
            }
            other => panic!("expected binary op, got {other:?}"),
        }
    }
}

#[test]
fn parse_logical_and_or() {
    let expr = parse_str("true && false || true").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Or, lhs, rhs) => {
            match *lhs {
                Expr::BinaryOp(BinaryOp::And, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Boolean(true)));
                    assert_eq!(*r, Expr::Literal(Value::Boolean(false)));
                }
                other => panic!("expected and, got {other:?}"),
            }
            assert_eq!(*rhs, Expr::Literal(Value::Boolean(true)));
        }
        other => panic!("expected or, got {other:?}"),
    }
}

#[test]
fn parse_logical_and_higher_precedence_than_or() {
    let expr = parse_str("true || false && false").unwrap();
    match expr {
        Expr::BinaryOp(BinaryOp::Or, lhs, rhs) => {
            assert_eq!(*lhs, Expr::Literal(Value::Boolean(true)));
            match *rhs {
                Expr::BinaryOp(BinaryOp::And, l, r) => {
                    assert_eq!(*l, Expr::Literal(Value::Boolean(false)));
                    assert_eq!(*r, Expr::Literal(Value::Boolean(false)));
                }
                other => panic!("expected and, got {other:?}"),
            }
        }
        other => panic!("expected or, got {other:?}"),
    }
}

#[test]
fn parse_function_call_no_args() {
    let expr = parse_str("NOW()").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "NOW");
            assert!(args.is_empty());
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_function_call_single_arg() {
    let expr = parse_str("SQRT(9)").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "SQRT");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Expr::Literal(Value::Integer(9.into())));
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_function_call_multiple_args() {
    let expr = parse_str("MAX(1, 2, 3)").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "MAX");
            assert_eq!(args.len(), 3);
            assert_eq!(args[0], Expr::Literal(Value::Integer(1.into())));
            assert_eq!(args[1], Expr::Literal(Value::Integer(2.into())));
            assert_eq!(args[2], Expr::Literal(Value::Integer(3.into())));
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_function_call_with_expression_arg() {
    let expr = parse_str("SUM(1 + 2)").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::BinaryOp(BinaryOp::Add, l, r) => {
                    assert_eq!(**l, Expr::Literal(Value::Integer(1.into())));
                    assert_eq!(**r, Expr::Literal(Value::Integer(2.into())));
                }
                other => panic!("expected addition argument, got {other:?}"),
            }
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_nested_function_calls() {
    let expr = parse_str("MAX(MIN(1,2), 3)").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "MAX");
            assert_eq!(args.len(), 2);
            match &args[0] {
                Expr::FunctionCall(inner_name, inner_args) => {
                    assert_eq!(inner_name, "MIN");
                    assert_eq!(inner_args.len(), 2);
                }
                other => panic!("expected nested function, got {other:?}"),
            }
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_cell_reference_simple() {
    let expr = parse_str("B2").unwrap();
    match expr {
        Expr::CellRef(s) => assert_eq!(s, "B2"),
        other => panic!("expected cell ref, got {other:?}"),
    }
}

#[test]
fn parse_cell_reference_absolute() {
    let expr = parse_str("$B$2").unwrap();
    match expr {
        Expr::CellRef(s) => assert_eq!(s, "$B$2"),
        other => panic!("expected cell ref, got {other:?}"),
    }
}

#[test]
fn parse_sheet_reference() {
    let expr = parse_str("Sheet2!C3").unwrap();
    match expr {
        Expr::CellRef(s) => assert_eq!(s, "Sheet2!C3"),
        other => panic!("expected sheet cell ref, got {other:?}"),
    }
}

#[test]
fn parse_range_simple() {
    let expr = parse_str("A1:B2").unwrap();
    match expr {
        Expr::Range(range) => {
            assert_eq!(range.start.to_string(), "A1");
            assert_eq!(range.end.to_string(), "B2");
        }
        other => panic!("expected range, got {other:?}"),
    }
}

#[test]
fn parse_range_with_sheet() {
    let expr = parse_str("Sheet1!A1:B2").unwrap();
    match expr {
        Expr::Range(range) => {
            assert_eq!(range.start.sheet.as_deref(), Some("Sheet1"));
            assert_eq!(range.start.to_string(), "Sheet1!A1");
            assert_eq!(range.end.to_string(), "Sheet1!B2");
        }
        other => panic!("expected range with sheet, got {other:?}"),
    }
}

#[test]
fn parse_function_call_with_range_arg() {
    let expr = parse_str("SUM(Sheet1!A1:B2)").unwrap();
    match expr {
        Expr::FunctionCall(name, args) => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::Range(range) => {
                    assert_eq!(range.start.sheet.as_deref(), Some("Sheet1"));
                    assert_eq!(range.start.to_string(), "Sheet1!A1");
                    assert_eq!(range.end.to_string(), "Sheet1!B2");
                }
                other => panic!("expected range argument, got {other:?}"),
            }
        }
        other => panic!("expected function call, got {other:?}"),
    }
}

#[test]
fn parse_error_empty_input() {
    assert!(parse_str("").is_err());
}

#[test]
fn parse_error_trailing_tokens() {
    let result = parse_str("1 2");
    assert!(result.is_err());
}

#[test]
fn parse_error_missing_rparen() {
    let result = parse_str("(1+2");
    assert!(result.is_err());
}

#[test]
fn parse_error_missing_operand_after_operator() {
    let result = parse_str("1 +");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_token() {
    let result = parse_str(")");
    assert!(result.is_err());
}

#[test]
fn parse_error_function_missing_comma_or_rparen() {
    let result = parse_str("SUM(1 2)");
    assert!(result.is_err());
}

#[test]
fn parse_error_range_missing_end() {
    let result = parse_str("A1:");
    assert!(result.is_err());
}

#[test]
fn parse_error_sheet_mismatch_in_range() {
    let result = parse_str("Sheet1!A1:Sheet2!B2");
    assert!(result.is_err());
}

#[test]
fn parse_error_invalid_cell_ref_in_range() {
    let result = parse_str("A1:1B");
    assert!(result.is_err());
}

#[test]
fn parse_error_unclosed_string_handled_by_tokenizer() {
    let result = parse_str("\"unclosed");
    assert!(result.is_err());
}

#[test]
fn parse_error_float_non_finite() {
    let result = parse_str("1e999");
    assert!(result.is_err());
}
