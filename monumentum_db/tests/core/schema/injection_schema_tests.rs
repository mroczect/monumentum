use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn int_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Integer)
}

fn text_col(name: &str) -> ColumnDef {
    ColumnDef::new(name, DataType::Text)
}

#[test]
fn schema_allows_sql_keywords_as_identifiers() -> Result<(), DbError> {
    let schema = TableSchema::try_new("SELECT", vec![int_col("DROP")])?;
    assert_eq!(schema.name(), "SELECT");
    assert_eq!(schema.columns()[0].name(), "DROP");
    Ok(())
}

#[test]
fn schema_allows_names_with_sql_comment_sequences() -> Result<(), DbError> {
    let name = "users; DROP TABLE users;--";
    let schema = TableSchema::try_new(name, vec![int_col("id")])?;
    assert_eq!(schema.name(), name);
    Ok(())
}

#[test]
fn schema_rejects_too_many_columns() {
    let mut columns = Vec::new();
    for i in 0..=1024 {
        columns.push(int_col(&format!("col_{i}")));
    }
    let result = TableSchema::try_new("wide", columns);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("too many columns"));
    }
}

#[test]
fn schema_rejects_duplicate_column_case_insensitive_even_with_sql_chars() {
    let col1 = int_col("ID; DROP TABLE");
    let col2 = int_col("id; drop table");
    let result = TableSchema::try_new("t", vec![col1, col2]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("duplicate column name"));
    }
}

#[test]
fn insert_rejects_type_mismatch_looking_like_injection() -> Result<(), DbError> {
    let schema = TableSchema::try_new("users", vec![int_col("id")])?;
    let mut table = Table::new(schema);
    let malicious_value = Value::from("1; DROP TABLE users;--");
    let result = table.insert(Row::new(vec![malicious_value]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TypeMismatch(_)));
        assert!(e.to_string().contains("expects INTEGER"));
    }
    Ok(())
}

#[test]
fn insert_rejects_null_in_non_nullable_column_with_injection_name() -> Result<(), DbError> {
    let mut col = int_col("id; DROP TABLE");
    col.set_nullable(false);
    let schema = TableSchema::try_new("t", vec![col])?;
    let mut table = Table::new(schema);
    let result = table.insert(Row::new(vec![Value::Null]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("is not nullable"));
    }
    Ok(())
}

#[test]
fn insert_succeeds_with_column_name_looking_like_injection() -> Result<(), DbError> {
    let col = int_col("id; DROP TABLE users;--");
    let schema = TableSchema::try_new("users", vec![col])?;
    let mut table = Table::new(schema);
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    assert_eq!(table.len(), 1);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&Value::from(1_i64)));
    }
    Ok(())
}

#[test]
fn catalog_can_create_table_with_injection_like_name() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("users; DROP TABLE users;--", vec![int_col("id")])?;
    catalog.create_table(schema)?;
    assert!(catalog.get_table("users; DROP TABLE users;--").is_some());
    Ok(())
}

#[test]
fn long_identifier_names_do_not_crash_or_error() -> Result<(), DbError> {
    let long_table_name = "t".repeat(10_000);
    let long_col_name = "c".repeat(10_000);
    let schema = TableSchema::try_new(long_table_name.clone(), vec![int_col(&long_col_name)])?;
    assert_eq!(schema.name(), long_table_name);
    assert_eq!(schema.columns()[0].name(), long_col_name);
    Ok(())
}

#[test]
fn column_name_with_null_character_is_accepted_as_plain_string() -> Result<(), DbError> {
    let col_name = "id\0; DROP TABLE";
    let schema = TableSchema::try_new("t", vec![int_col(col_name)])?;
    assert_eq!(schema.columns()[0].name(), col_name);
    Ok(())
}

#[test]
fn insert_with_value_containing_sql_comment_succeeds_for_text_column() -> Result<(), DbError> {
    let col = text_col("name");
    let schema = TableSchema::try_new("users", vec![col])?;
    let mut table = Table::new(schema);
    let value = Value::from("Robert'); DROP TABLE users;--");
    table.insert(Row::new(vec![value.clone()]))?;
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&value));
    } else {
        return Err(DbError::invalid_operation("row missing"));
    }
    Ok(())
}

#[test]
fn schema_allows_control_characters_in_names() -> Result<(), DbError> {
    let weird_name = "line\nbreak\ttab\rreturn";
    let schema = TableSchema::try_new(weird_name, vec![int_col("col\nname")])?;
    assert_eq!(schema.name(), weird_name);
    assert_eq!(schema.columns()[0].name(), "col\nname");
    Ok(())
}

#[test]
fn schema_allows_quotes_and_backslashes_in_names() -> Result<(), DbError> {
    let table_name = "table'\"`\\";
    let col_name = "col'\"`\\";
    let schema = TableSchema::try_new(table_name, vec![int_col(col_name)])?;
    assert_eq!(schema.name(), table_name);
    assert_eq!(schema.columns()[0].name(), col_name);
    Ok(())
}

#[test]
fn schema_allows_unicode_identifiers() -> Result<(), DbError> {
    let table_name = "таблица🔥";
    let col_name = "колонка🚀";
    let schema = TableSchema::try_new(table_name, vec![int_col(col_name)])?;
    assert_eq!(schema.name(), table_name);
    assert_eq!(schema.columns()[0].name(), col_name);
    Ok(())
}

#[test]
fn schema_allows_leading_trailing_whitespace() -> Result<(), DbError> {
    let table_name = "  padded  ";
    let col_name = "  col  ";
    let schema = TableSchema::try_new(table_name, vec![int_col(col_name)])?;
    assert_eq!(schema.name(), table_name);
    assert_eq!(schema.columns()[0].name(), col_name);
    Ok(())
}

#[test]
fn insert_rejects_boolean_in_integer_column() -> Result<(), DbError> {
    let schema = TableSchema::try_new("t", vec![int_col("id")])?;
    let mut table = Table::new(schema);
    let result = table.insert(Row::new(vec![Value::Boolean(true)]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TypeMismatch(_)));
        assert!(e.to_string().contains("expects INTEGER"));
    }
    Ok(())
}

#[test]
fn insert_accepts_formula_in_integer_column() -> Result<(), DbError> {
    let schema = TableSchema::try_new("t", vec![int_col("id")])?;
    let mut table = Table::new(schema);
    let formula = Value::Formula("1+1".to_string());
    table.insert(Row::new(vec![formula.clone()]))?;
    assert_eq!(table.len(), 1);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&formula));
    }
    Ok(())
}

#[test]
fn insert_blob_with_null_bytes_and_sql_sequence() -> Result<(), DbError> {
    let col = ColumnDef::new("data", DataType::Blob);
    let schema = TableSchema::try_new("t", vec![col])?;
    let mut table = Table::new(schema);
    let blob = Value::from(vec![0_u8, 1, 2, b';', b' ', b'D', b'R', b'O', b'P']);
    table.insert(Row::new(vec![blob.clone()]))?;
    assert_eq!(table.len(), 1);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&blob));
    }
    Ok(())
}

#[test]
fn long_value_does_not_crash() -> Result<(), DbError> {
    let col = text_col("text");
    let schema = TableSchema::try_new("t", vec![col])?;
    let mut table = Table::new(schema);
    let long_text = Value::from("A".repeat(10_000));
    table.insert(Row::new(vec![long_text.clone()]))?;
    assert_eq!(table.len(), 1);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&long_text));
    }
    Ok(())
}

#[test]
fn duplicate_column_names_unicode_case_insensitive() {
    let col1 = int_col("Ä");
    let col2 = int_col("ä");
    let result = TableSchema::try_new("t", vec![col1, col2]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("duplicate column name"));
    }
}

#[test]
fn column_lookup_works_with_special_names() -> Result<(), DbError> {
    let col_name = "id; DROP TABLE";
    let schema = TableSchema::try_new("t", vec![int_col(col_name)])?;
    assert_eq!(schema.column_index(col_name), Some(0));
    assert_eq!(schema.column_index("ID; DROP TABLE"), Some(0));
    assert_eq!(schema.column_index("nonexistent"), None);
    Ok(())
}

#[test]
fn value_with_escape_sequences_stored_verbatim() -> Result<(), DbError> {
    let col = text_col("content");
    let schema = TableSchema::try_new("t", vec![col])?;
    let mut table = Table::new(schema);
    let text = Value::from("line\\nbreak\\ttab\\\\backslash");
    table.insert(Row::new(vec![text.clone()]))?;
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&text));
    }
    Ok(())
}
