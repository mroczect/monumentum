use monumentum_db::core::{ColumnDef, DataType, TableSchema, Value};
use monumentum_db::handler::{Executor, QueryResult, SimpleExecutor, Statement, WhereClause};
use monumentum_db::store::{FileStorage, StorageEngine};
use std::io::{self, Write};
use std::path::PathBuf;

fn parse_value(s: &str) -> Result<Value, String> {
    if s.eq_ignore_ascii_case("null") {
        Ok(Value::Null)
    } else if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        Ok(Value::from(inner.to_string()))
    } else if let Ok(i) = s.parse::<i64>() {
        Ok(Value::from(i))
    } else if let Ok(f) = s.parse::<f64>() {
        Value::try_from(f).map_err(|e| e.to_string())
    } else {
        Err(format!("cannot parse value: {}", s))
    }
}

fn parse_data_type(s: &str) -> Result<DataType, String> {
    match s.to_ascii_uppercase().as_str() {
        "NULL" => Ok(DataType::Null),
        "INTEGER" | "INT" => Ok(DataType::Integer),
        "FLOAT" | "REAL" => Ok(DataType::Float),
        "TEXT" | "STRING" => Ok(DataType::Text),
        "BLOB" => Ok(DataType::Blob),
        _ => Err(format!("unknown data type: {}", s)),
    }
}

fn parse_create_table(args: &[&str]) -> Result<Statement, String> {
    if args.len() < 3 {
        return Err("usage: CREATE TABLE table_name (col1 TYPE, col2 TYPE, ...)".into());
    }
    let table_name = args[2];
    let input = args.join(" ");
    let start = input.find('(').ok_or("missing '('")?;
    let end = input.rfind(')').ok_or("missing ')'")?;
    if start >= end {
        return Err("invalid column definitions".into());
    }
    let columns_str = &input[start + 1..end];
    let mut columns = Vec::new();
    for col_def in columns_str.split(',') {
        let parts: Vec<&str> = col_def.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err("column definition must be: name TYPE".into());
        }
        let name = parts[0];
        let dtype = parse_data_type(parts[1])?;
        let mut col = ColumnDef::new(name, dtype);
        for opt in parts.iter().skip(2) {
            match opt.to_ascii_uppercase().as_str() {
                "PRIMARY" => {}
                "KEY" => col.set_primary_key(true),
                "NOT" => {}
                "NULL" => col.set_nullable(false),
                "UNIQUE" => col.set_unique(true),
                _ => {}
            }
        }
        columns.push(col);
    }
    let schema = TableSchema::try_new(table_name, columns).map_err(|e| e.to_string())?;
    Ok(Statement::CreateTable(schema))
}

fn parse_insert(args: &[&str]) -> Result<Statement, String> {
    if args.len() < 5
        || !args[0].eq_ignore_ascii_case("insert")
        || !args[1].eq_ignore_ascii_case("into")
    {
        return Err("usage: INSERT INTO table VALUES (v1, v2, ...)".into());
    }
    let table = args[2].to_string();
    let input = args.join(" ");
    let start = input.find('(').ok_or("missing '('")?;
    let end = input.rfind(')').ok_or("missing ')'")?;
    if start >= end {
        return Err("invalid values".into());
    }
    let values_str = &input[start + 1..end];
    let mut values = Vec::new();
    for v in values_str.split(',') {
        values.push(parse_value(v.trim())?);
    }
    Ok(Statement::Insert { table, values })
}

fn parse_select(args: &[&str]) -> Result<Statement, String> {
    if args.len() < 4
        || !args[0].eq_ignore_ascii_case("select")
        || !args[2].eq_ignore_ascii_case("from")
    {
        return Err("usage: SELECT * FROM table [WHERE col = value]".into());
    }
    let columns_str = args[1];
    let columns = if columns_str == "*" {
        vec![]
    } else {
        columns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };
    let table = args[3].to_string();
    let mut where_clause = None;
    if args.len() > 4 {
        if !args[4].eq_ignore_ascii_case("where") {
            return Err("expected WHERE".into());
        }
        if args.len() < 7 {
            return Err("usage: SELECT ... WHERE column = value".into());
        }
        let column = args[5].to_string();
        let value = parse_value(args[6])?;
        where_clause = Some(WhereClause { column, value });
    }
    Ok(Statement::Select {
        table,
        columns,
        where_clause,
    })
}

fn parse_update(args: &[&str]) -> Result<Statement, String> {
    if args.len() < 5
        || !args[0].eq_ignore_ascii_case("update")
        || !args[2].eq_ignore_ascii_case("set")
    {
        return Err("usage: UPDATE table SET col = value [WHERE col = value]".into());
    }
    let table = args[1].to_string();
    let mut assignments = Vec::new();
    let mut where_clause = None;
    let mut i = 3;
    while i < args.len() && !args[i].eq_ignore_ascii_case("where") {
        if i + 2 >= args.len() || args[i + 1] != "=" {
            return Err("SET assignment must be column = value".into());
        }
        let col = args[i].to_string();
        let val = parse_value(args[i + 2])?;
        assignments.push((col, val));
        i += 3;
        if i < args.len() && args[i] == "," {
            i += 1;
        }
    }
    if i < args.len() && args[i].eq_ignore_ascii_case("where") {
        if i + 2 >= args.len() {
            return Err("WHERE requires column = value".into());
        }
        let column = args[i + 1].to_string();
        let value = parse_value(args[i + 2])?;
        where_clause = Some(WhereClause { column, value });
    }
    Ok(Statement::Update {
        table,
        assignments,
        where_clause,
    })
}

fn parse_delete(args: &[&str]) -> Result<Statement, String> {
    if args.len() < 3
        || !args[0].eq_ignore_ascii_case("delete")
        || !args[1].eq_ignore_ascii_case("from")
    {
        return Err("usage: DELETE FROM table [WHERE col = value]".into());
    }
    let table = args[2].to_string();
    let mut where_clause = None;
    if args.len() > 3 {
        if !args[3].eq_ignore_ascii_case("where") {
            return Err("expected WHERE".into());
        }
        if args.len() < 6 {
            return Err("WHERE requires column = value".into());
        }
        let column = args[4].to_string();
        let value = parse_value(args[5])?;
        where_clause = Some(WhereClause { column, value });
    }
    Ok(Statement::Delete {
        table,
        where_clause,
    })
}

fn parse_drop_table(args: &[&str]) -> Result<Statement, String> {
    if args.len() != 3
        || !args[0].eq_ignore_ascii_case("drop")
        || !args[1].eq_ignore_ascii_case("table")
    {
        return Err("usage: DROP TABLE table_name".into());
    }
    Ok(Statement::DropTable(args[2].to_string()))
}

fn parse_statement(input: &str) -> Result<Statement, String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("empty command".into());
    }
    match tokens[0].to_ascii_lowercase().as_str() {
        "create" => parse_create_table(&tokens),
        "insert" => parse_insert(&tokens),
        "select" => parse_select(&tokens),
        "update" => parse_update(&tokens),
        "delete" => parse_delete(&tokens),
        "drop" => parse_drop_table(&tokens),
        _ => Err("unsupported command".into()),
    }
}

fn print_result(result: QueryResult) {
    match result {
        QueryResult::Empty => println!("OK"),
        QueryResult::AffectedRows(n) => println!("Affected rows: {}", n),
        QueryResult::Rows(rows) => {
            if rows.is_empty() {
                println!("(no rows)");
            } else {
                for row in rows {
                    let vals: Vec<String> = row.values().iter().map(|v| v.to_string()).collect();
                    println!("{}", vals.join(" | "));
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("monumentum.db");
    let mut storage = FileStorage::open(&path)?;
    let mut catalog = storage.load_catalog()?;

    println!("Monumentum DB CLI (type 'exit' to quit)");

    loop {
        print!("monumentum> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        match parse_statement(trimmed) {
            Ok(statement) => {
                let result = {
                    let mut executor = SimpleExecutor::new(&mut catalog);
                    executor.execute(statement)
                };
                match result {
                    Ok(query_result) => {
                        print_result(query_result);
                        storage.save_catalog(&catalog)?;
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }

    println!("Bye!");
    Ok(())
}
