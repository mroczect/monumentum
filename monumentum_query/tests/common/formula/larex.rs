use monumentum_query::formula::{Token, tokenize};

#[test]
fn tokenize_empty_input_returns_empty_vec() {
    let tokens = tokenize("").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_whitespace_only_returns_empty_vec() {
    let tokens = tokenize(" \t\n ").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_integer_positive() {
    let tokens = tokenize("42").unwrap();
    assert_eq!(tokens, vec![Token::Integer(42)]);
}

#[test]
fn tokenize_integer_zero() {
    let tokens = tokenize("0").unwrap();
    assert_eq!(tokens, vec![Token::Integer(0)]);
}

#[test]
fn tokenize_integer_max_i64() {
    let tokens = tokenize("9223372036854775807").unwrap();
    assert_eq!(tokens, vec![Token::Integer(i64::MAX)]);
}

#[test]
fn tokenize_integer_overflow_returns_error() {
    assert!(tokenize("9223372036854775808").is_err());
}

#[test]
fn tokenize_integer_min_i64_as_negative_operator_and_number() {
    assert!(tokenize("-9223372036854775808").is_err());
}

#[test]
fn tokenize_negative_integer_is_minus_then_integer() {
    let tokens = tokenize("-42").unwrap();
    assert_eq!(tokens, vec![Token::Minus, Token::Integer(42)]);
}

#[test]
fn tokenize_float_simple() {
    let tokens = tokenize("2.5").unwrap();
    assert_eq!(tokens, vec![Token::Float(2.5)]);
}

#[test]
fn tokenize_float_trailing_dot() {
    let tokens = tokenize("1.").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        Token::Float(v) => assert_eq!(*v, 1.0),
        other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn tokenize_float_multiple_dots_returns_error() {
    assert!(tokenize("1.2.3").is_err());
}

#[test]
fn tokenize_float_leading_dot_returns_error() {
    assert!(tokenize(".5").is_err());
}

#[test]
fn tokenize_exponent_lowercase_e() {
    let tokens = tokenize("1e5").unwrap();
    assert_eq!(tokens, vec![Token::Float(100_000.0)]);
}

#[test]
fn tokenize_exponent_uppercase_e() {
    let tokens = tokenize("1E3").unwrap();
    assert_eq!(tokens, vec![Token::Float(1_000.0)]);
}

#[test]
fn tokenize_exponent_positive_sign() {
    let tokens = tokenize("1e+5").unwrap();
    assert_eq!(tokens, vec![Token::Float(100_000.0)]);
}

#[test]
fn tokenize_exponent_negative_sign() {
    let tokens = tokenize("1e-5").unwrap();
    assert_eq!(tokens, vec![Token::Float(0.000_01)]);
}

#[test]
fn tokenize_exponent_without_digits_returns_error() {
    assert!(tokenize("1e").is_err());
    assert!(tokenize("1e+").is_err());
    assert!(tokenize("1e-").is_err());
}

#[test]
fn tokenize_float_overflow_to_infinity() {
    let tokens = tokenize("1e999").unwrap();
    assert_eq!(tokens.len(), 1);
    match tokens[0] {
        Token::Float(v) => assert!(v.is_infinite() && v.is_sign_positive()),
        ref other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn tokenize_string_empty() {
    let tokens = tokenize("\"\"").unwrap();
    assert_eq!(tokens, vec![Token::String(String::new())]);
}

#[test]
fn tokenize_string_with_spaces() {
    let tokens = tokenize("\"hello world\"").unwrap();
    assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
}

#[test]
fn tokenize_string_with_unicode() {
    let tokens = tokenize("\"héllo wörld 😀\"").unwrap();
    assert_eq!(tokens, vec![Token::String("héllo wörld 😀".to_string())]);
}

#[test]
fn tokenize_string_unterminated_returns_error() {
    assert!(tokenize("\"hello").is_err());
}

#[test]
fn tokenize_boolean_true() {
    let tokens = tokenize("true").unwrap();
    assert_eq!(tokens, vec![Token::Boolean(true)]);
}

#[test]
fn tokenize_boolean_false() {
    let tokens = tokenize("false").unwrap();
    assert_eq!(tokens, vec![Token::Boolean(false)]);
}

#[test]
fn tokenize_null_literal() {
    let tokens = tokenize("null").unwrap();
    assert_eq!(tokens, vec![Token::Null]);
}

#[test]
fn tokenize_boolean_and_null_without_spaces() {
    let tokens = tokenize("truefalse null").unwrap();
    assert_eq!(
        tokens,
        vec![Token::Identifier("truefalse".to_string()), Token::Null]
    );
}

#[test]
fn tokenize_identifier_simple() {
    let tokens = tokenize("myVar _private $A$1").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("myVar".to_string()),
            Token::Identifier("_private".to_string()),
            Token::Identifier("$A$1".to_string()),
        ]
    );
}

#[test]
fn tokenize_identifier_starting_with_underscore_or_dollar() {
    let tokens = tokenize("_x $y").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("_x".to_string()),
            Token::Identifier("$y".to_string()),
        ]
    );
}

#[test]
fn tokenize_identifier_unicode_returns_error() {
    assert!(tokenize("é").is_err());
}

#[test]
fn tokenize_number_followed_by_identifier_without_space() {
    let tokens = tokenize("1abc").unwrap();
    assert_eq!(
        tokens,
        vec![Token::Integer(1), Token::Identifier("abc".to_string()),]
    );
}

#[test]
fn tokenize_plus_operator() {
    let tokens = tokenize("+").unwrap();
    assert_eq!(tokens, vec![Token::Plus]);
}

#[test]
fn tokenize_minus_operator() {
    let tokens = tokenize("-").unwrap();
    assert_eq!(tokens, vec![Token::Minus]);
}

#[test]
fn tokenize_star_operator() {
    let tokens = tokenize("*").unwrap();
    assert_eq!(tokens, vec![Token::Star]);
}

#[test]
fn tokenize_slash_operator() {
    let tokens = tokenize("/").unwrap();
    assert_eq!(tokens, vec![Token::Slash]);
}

#[test]
fn tokenize_percent_operator() {
    let tokens = tokenize("%").unwrap();
    assert_eq!(tokens, vec![Token::Percent]);
}

#[test]
fn tokenize_caret_operator() {
    let tokens = tokenize("^").unwrap();
    assert_eq!(tokens, vec![Token::Caret]);
}

#[test]
fn tokenize_eqeq_operator() {
    let tokens = tokenize("==").unwrap();
    assert_eq!(tokens, vec![Token::EqEq]);
}

#[test]
fn tokenize_single_equals_returns_error() {
    assert!(tokenize("=").is_err());
}

#[test]
fn tokenize_noteq_operator() {
    let tokens = tokenize("!=").unwrap();
    assert_eq!(tokens, vec![Token::NotEq]);
}

#[test]
fn tokenize_bang_operator() {
    let tokens = tokenize("!").unwrap();
    assert_eq!(tokens, vec![Token::Bang]);
}

#[test]
fn tokenize_double_bang() {
    let tokens = tokenize("!!").unwrap();
    assert_eq!(tokens, vec![Token::Bang, Token::Bang]);
}

#[test]
fn tokenize_lt_operator() {
    let tokens = tokenize("<").unwrap();
    assert_eq!(tokens, vec![Token::Lt]);
}

#[test]
fn tokenize_lte_operator() {
    let tokens = tokenize("<=").unwrap();
    assert_eq!(tokens, vec![Token::LtEq]);
}

#[test]
fn tokenize_gt_operator() {
    let tokens = tokenize(">").unwrap();
    assert_eq!(tokens, vec![Token::Gt]);
}

#[test]
fn tokenize_gte_operator() {
    let tokens = tokenize(">=").unwrap();
    assert_eq!(tokens, vec![Token::GtEq]);
}

#[test]
fn tokenize_andand_operator() {
    let tokens = tokenize("&&").unwrap();
    assert_eq!(tokens, vec![Token::AndAnd]);
}

#[test]
fn tokenize_single_ampersand_returns_error() {
    assert!(tokenize("&").is_err());
}

#[test]
fn tokenize_oror_operator() {
    let tokens = tokenize("||").unwrap();
    assert_eq!(tokens, vec![Token::OrOr]);
}

#[test]
fn tokenize_single_pipe_returns_error() {
    assert!(tokenize("|").is_err());
}

#[test]
fn tokenize_lparen() {
    let tokens = tokenize("(").unwrap();
    assert_eq!(tokens, vec![Token::LParen]);
}

#[test]
fn tokenize_rparen() {
    let tokens = tokenize(")").unwrap();
    assert_eq!(tokens, vec![Token::RParen]);
}

#[test]
fn tokenize_comma() {
    let tokens = tokenize(",").unwrap();
    assert_eq!(tokens, vec![Token::Comma]);
}

#[test]
fn tokenize_colon() {
    let tokens = tokenize(":").unwrap();
    assert_eq!(tokens, vec![Token::Colon]);
}

#[test]
fn tokenize_expression_without_spaces() {
    let tokens = tokenize("1+2*3").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Integer(1),
            Token::Plus,
            Token::Integer(2),
            Token::Star,
            Token::Integer(3),
        ]
    );
}

#[test]
fn tokenize_function_call() {
    let tokens = tokenize("SUM(A1:B2)").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("SUM".to_string()),
            Token::LParen,
            Token::Identifier("A1".to_string()),
            Token::Colon,
            Token::Identifier("B2".to_string()),
            Token::RParen,
        ]
    );
}

#[test]
fn tokenize_reference_with_sheet_and_bang() {
    let tokens = tokenize("Sheet2!A1").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Identifier("Sheet2".to_string()),
            Token::Bang,
            Token::Identifier("A1".to_string()),
        ]
    );
}

#[test]
fn tokenize_error_unexpected_character() {
    assert!(tokenize("@").is_err());
    assert!(tokenize("#").is_err());
    assert!(tokenize("~").is_err());
}
