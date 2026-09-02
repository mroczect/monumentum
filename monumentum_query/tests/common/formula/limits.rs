use monumentum_db::core::value::Value;
use monumentum_query::coordinates::CellRef;
use monumentum_query::formula::{
    FormulaContext, FormulaError, FunctionRegistry, evaluate, parse, tokenize,
};
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

fn dummy_fn(_args: &[Value]) -> Result<Value, FormulaError> {
    Ok(Value::Null)
}

#[test]
fn lexer_rejects_too_long_input() {
    let long_input = "1".repeat(65 * 1024);
    let result = tokenize(&long_input);
    assert!(matches!(result, Err(FormulaError::Parse(_))));
}

#[test]
fn parser_rejects_deeply_nested_parens() {
    let depth = 200;
    let input = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    let tokens = tokenize(&input).unwrap();
    let result = parse(&tokens);
    assert!(matches!(result, Err(FormulaError::Parse(_))));
}

#[test]
fn parser_rejects_deeply_nested_unary() {
    let depth = 200;
    let input = format!("{}1", "-".repeat(depth));
    let tokens = tokenize(&input).unwrap();
    let result = parse(&tokens);
    assert!(matches!(result, Err(FormulaError::Parse(_))));
}

#[test]
fn evaluator_rejects_huge_range() {
    let ctx = DummyContext::new();
    let mut registry = FunctionRegistry::new();
    registry.register("SUM", dummy_fn);

    let input = "SUM(A1:XFD1048576)";
    let tokens = tokenize(input).unwrap();
    let expr = parse(&tokens).unwrap();
    let result = evaluate(&expr, &ctx, &registry);
    assert!(matches!(result, Err(FormulaError::Eval(_))));
    match result {
        Err(FormulaError::Eval(msg)) => assert!(msg.contains("range too large")),
        other => panic!("expected range too large error, got {other:?}"),
    }
}
