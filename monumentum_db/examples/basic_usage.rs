use monumentum_db::core::{Catalog, ColumnDef, DataType, TableSchema, Value};
use monumentum_db::handler::{Executor, QueryResult, SimpleExecutor, Statement, WhereClause};

fn print_rows(result: &QueryResult) {
    if let QueryResult::Rows(rows) = result {
        for row in rows {
            let values: Vec<String> = row.values().iter().map(|v| v.to_string()).collect();
            println!("{}", values.join(" | "));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);

    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);

    let schema = TableSchema::try_new("users", vec![id_col, name_col])?;

    let mut catalog = Catalog::new();
    let mut executor = SimpleExecutor::new(&mut catalog);

    executor.execute(Statement::CreateTable(schema))?;

    executor.execute(Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from(1i64), Value::from("Alice")],
    })?;
    executor.execute(Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from(2i64), Value::from("Bob")],
    })?;
    executor.execute(Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from(3i64), Value::from("Charlie")],
    })?;

    println!("=== Select all ===");
    let result = executor.execute(Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: None,
    })?;
    print_rows(&result);

    println!("=== Select WHERE id = 2 ===");
    let result = executor.execute(Statement::Select {
        table: "users".to_string(),
        columns: vec!["id".to_string(), "name".to_string()],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(2i64),
        }),
    })?;
    print_rows(&result);

    println!("=== Update Bob -> Bobby ===");
    let affected = executor.execute(Statement::Update {
        table: "users".to_string(),
        assignments: vec![("name".to_string(), Value::from("Bobby"))],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(2i64),
        }),
    })?;
    if let QueryResult::AffectedRows(n) = affected {
        println!("Updated {} row(s)", n);
    }

    let result = executor.execute(Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(2i64),
        }),
    })?;
    print_rows(&result);

    println!("=== Delete user id = 1 ===");
    let affected = executor.execute(Statement::Delete {
        table: "users".to_string(),
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(1i64),
        }),
    })?;
    if let QueryResult::AffectedRows(n) = affected {
        println!("Deleted {} row(s)", n);
    }

    println!("=== Select all after delete ===");
    let result = executor.execute(Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: None,
    })?;
    print_rows(&result);

    Ok(())
}
