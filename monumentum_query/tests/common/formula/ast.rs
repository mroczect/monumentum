use monumentum_db::core::value::Value;
use monumentum_query::coordinates::CellRange;
use monumentum_query::formula::{BinaryOp, Expr, UnaryOp};

fn sample_cell_range() -> CellRange {
    CellRange::new_unchecked(
        monumentum_query::coordinates::CellRef::new(0, 0),
        monumentum_query::coordinates::CellRef::new(1, 1),
    )
}

#[test]
fn literal_expr_creation_and_equality() {
    let expr1 = Expr::Literal(Value::from(42_i64));
    let expr2 = Expr::Literal(Value::from(42_i64));
    let expr3 = Expr::Literal(Value::from("hello"));

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

#[test]
fn cell_ref_expr_creation_and_equality() {
    let expr1 = Expr::CellRef("A1".to_string());
    let expr2 = Expr::CellRef("A1".to_string());
    let expr3 = Expr::CellRef("B2".to_string());

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

#[test]
fn range_expr_creation_and_equality() {
    let range1 = sample_cell_range();
    let range2 = sample_cell_range();
    let expr1 = Expr::Range(range1);
    let expr2 = Expr::Range(range2);

    assert_eq!(expr1, expr2);
}

#[test]
fn unary_op_expr_creation_and_equality() {
    let inner = Expr::Literal(Value::from(10_i64));
    let expr1 = Expr::UnaryOp(UnaryOp::Neg, Box::new(inner.clone()));
    let expr2 = Expr::UnaryOp(UnaryOp::Neg, Box::new(inner));
    let expr3 = Expr::UnaryOp(UnaryOp::Not, Box::new(Expr::Literal(Value::Null)));

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

#[test]
fn binary_op_expr_creation_and_equality() {
    let left = Expr::Literal(Value::from(5_i64));
    let right = Expr::Literal(Value::from(3_i64));
    let expr1 = Expr::BinaryOp(
        BinaryOp::Add,
        Box::new(left.clone()),
        Box::new(right.clone()),
    );
    let expr2 = Expr::BinaryOp(BinaryOp::Add, Box::new(left), Box::new(right));
    let expr3 = Expr::BinaryOp(
        BinaryOp::Sub,
        Box::new(Expr::Literal(Value::from(5_i64))),
        Box::new(Expr::Literal(Value::from(3_i64))),
    );

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

#[test]
fn function_call_expr_creation_and_equality() {
    let args = vec![
        Expr::Literal(Value::from(1_i64)),
        Expr::Literal(Value::from(2_i64)),
    ];
    let expr1 = Expr::FunctionCall("SUM".to_string(), args.clone());
    let expr2 = Expr::FunctionCall("SUM".to_string(), args);
    let expr3 = Expr::FunctionCall("AVERAGE".to_string(), vec![]);

    assert_eq!(expr1, expr2);
    assert_ne!(expr1, expr3);
}

#[test]
fn binary_op_all_variants_are_distinct() {
    let variants = [
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::Pow,
        BinaryOp::Eq,
        BinaryOp::NotEq,
        BinaryOp::Lt,
        BinaryOp::Lte,
        BinaryOp::Gt,
        BinaryOp::Gte,
        BinaryOp::And,
        BinaryOp::Or,
    ];
    for i in 0..variants.len() {
        for j in 0..variants.len() {
            if i == j {
                assert_eq!(variants[i], variants[j]);
            } else {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }
}

#[test]
fn unary_op_all_variants_are_distinct() {
    assert_ne!(UnaryOp::Neg, UnaryOp::Not);
}

#[test]
fn expr_clone_works() {
    let expr = Expr::BinaryOp(
        BinaryOp::Add,
        Box::new(Expr::Literal(Value::from(1_i64))),
        Box::new(Expr::Literal(Value::from(2_i64))),
    );
    let cloned = expr.clone();
    assert_eq!(expr, cloned);
}

#[test]
fn expr_debug_format_contains_variant_name() {
    let expr = Expr::Literal(Value::Null);
    let debug_str = format!("{expr:?}");
    assert!(debug_str.contains("Literal"));
}
