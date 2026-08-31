use monumentum_db::core::{Catalog, ColumnDef, DataType, TableSchema, Value};
use monumentum_db::error::DbError;
use monumentum_db::handler::{Executor, QueryResult, SimpleExecutor, Statement, WhereClause};

fn setup_catalog() -> Catalog {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);
    let schema = TableSchema::try_new("users", vec![id_col, name_col]).unwrap();
    let mut catalog = Catalog::new();
    catalog.create_table(schema).unwrap();
    catalog
}

fn insert_user(executor: &mut SimpleExecutor, id: i64, name: &str) {
    let stmt = Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from(id), Value::from(name)],
    };
    executor.execute(stmt).unwrap();
}

#[test]
fn create_table_and_drop_table() {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("t", vec![ColumnDef::new("a", DataType::Integer)]).unwrap();

    {
        let mut executor = SimpleExecutor::new(&mut catalog);
        let result = executor.execute(Statement::CreateTable(schema)).unwrap();
        assert_eq!(result, QueryResult::Empty);
    }

    assert!(catalog.get_table("t").is_some());

    {
        let mut executor = SimpleExecutor::new(&mut catalog);
        let result = executor
            .execute(Statement::DropTable("t".to_string()))
            .unwrap();
        assert_eq!(result, QueryResult::Empty);
    }

    assert!(catalog.get_table("t").is_none());
}

#[test]
fn insert_and_select_all() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");
    insert_user(&mut executor, 2, "Bob");

    let select_stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: None,
    };
    match executor.execute(select_stmt).unwrap() {
        QueryResult::Rows(rows) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].get(0), Some(&Value::from(1i64)));
            assert_eq!(rows[0].get(1), Some(&Value::from("Alice")));
            assert_eq!(rows[1].get(0), Some(&Value::from(2i64)));
            assert_eq!(rows[1].get(1), Some(&Value::from("Bob")));
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn select_specific_columns() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");

    let select_stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec!["name".to_string()],
        where_clause: None,
    };
    match executor.execute(select_stmt).unwrap() {
        QueryResult::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 1);
            assert_eq!(rows[0].get(0), Some(&Value::from("Alice")));
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn select_with_where_clause() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");
    insert_user(&mut executor, 2, "Bob");

    let select_stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(2i64),
        }),
    };
    match executor.execute(select_stmt).unwrap() {
        QueryResult::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get(0), Some(&Value::from(2i64)));
            assert_eq!(rows[0].get(1), Some(&Value::from("Bob")));
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn insert_duplicate_primary_key_fails() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");
    let stmt = Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from(1i64), Value::from("Bob")],
    };
    let result = executor.execute(stmt);
    assert!(matches!(result, Err(DbError::InvalidOperation(_))));
}

#[test]
fn update_single_row() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");
    insert_user(&mut executor, 2, "Bob");

    let stmt = Statement::Update {
        table: "users".to_string(),
        assignments: vec![("name".to_string(), Value::from("Alice Updated"))],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(1i64),
        }),
    };
    let result = executor.execute(stmt).unwrap();
    assert_eq!(result, QueryResult::AffectedRows(1));

    let select_stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(1i64),
        }),
    };
    match executor.execute(select_stmt).unwrap() {
        QueryResult::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get(1), Some(&Value::from("Alice Updated")));
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn delete_with_where_clause() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");
    insert_user(&mut executor, 2, "Bob");

    let stmt = Statement::Delete {
        table: "users".to_string(),
        where_clause: Some(WhereClause {
            column: "id".to_string(),
            value: Value::from(1i64),
        }),
    };
    let result = executor.execute(stmt).unwrap();
    assert_eq!(result, QueryResult::AffectedRows(1));

    let select_stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec![],
        where_clause: None,
    };
    match executor.execute(select_stmt).unwrap() {
        QueryResult::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get(0), Some(&Value::from(2i64)));
        }
        _ => panic!("expected rows"),
    }
}

#[test]
fn table_not_found_error() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    let stmt = Statement::Select {
        table: "missing".to_string(),
        columns: vec![],
        where_clause: None,
    };
    assert!(matches!(
        executor.execute(stmt),
        Err(DbError::TableNotFound(_))
    ));
}

#[test]
fn column_not_found_error() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    insert_user(&mut executor, 1, "Alice");

    let stmt = Statement::Select {
        table: "users".to_string(),
        columns: vec!["nonexistent".to_string()],
        where_clause: None,
    };
    assert!(matches!(
        executor.execute(stmt),
        Err(DbError::ColumnNotFound(_))
    ));
}

#[test]
fn type_mismatch_on_insert() {
    let mut catalog = setup_catalog();
    let mut executor = SimpleExecutor::new(&mut catalog);

    let stmt = Statement::Insert {
        table: "users".to_string(),
        values: vec![Value::from("not an int"), Value::from("Alice")],
    };
    assert!(matches!(
        executor.execute(stmt),
        Err(DbError::TypeMismatch(_))
    ));
}
