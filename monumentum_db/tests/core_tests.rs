use monumentum_db::core::*;
use monumentum_db::error::DbError;
use monumentum_db::types::*;
use std::cmp::Ordering;

fn float(v: f64) -> Float {
    Float::try_new(v).expect("finite float expected")
}

fn int(v: i64) -> Integer {
    Integer::new(v)
}

fn text(s: &str) -> Text {
    Text::new(s.to_string())
}

fn blob(data: &[u8]) -> Blob {
    Blob::new(data.to_vec())
}

#[test]
fn value_default_is_null() {
    let v = Value::default();
    assert!(v.is_null());
    assert_eq!(v.type_name(), "null");
}

#[test]
fn value_variant_checks_and_accessors() {
    let vi = Value::Integer(int(42));
    assert!(vi.is_integer());
    assert!(!vi.is_float());
    assert!(!vi.is_text());
    assert!(!vi.is_blob());
    assert_eq!(vi.as_integer().map(|i| i.as_i64()), Some(42));
    assert_eq!(vi.into_integer().map(|i| i.as_i64()), Some(42));

    let vf = Value::Float(float(2.5));
    assert!(vf.is_float());
    assert_eq!(vf.as_float().map(|f| f.as_f64()), Some(2.5));
    assert_eq!(vf.into_float().map(|f| f.as_f64()), Some(2.5));

    let vt = Value::Text(text("hello"));
    assert!(vt.is_text());
    assert_eq!(vt.as_text().map(|t| t.as_str()), Some("hello"));
    assert_eq!(
        vt.into_text().map(|t| t.as_str().to_string()),
        Some("hello".to_string())
    );

    let vb = Value::Blob(blob(&[1, 2, 3]));
    assert!(vb.is_blob());
    assert_eq!(vb.as_blob().map(|b| b.as_slice()), Some(&[1, 2, 3][..]));
    assert_eq!(
        vb.into_blob().map(|b| b.as_slice().to_vec()),
        Some(vec![1, 2, 3])
    );
}

#[test]
fn value_type_name() {
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::Integer(int(0)).type_name(), "integer");
    assert_eq!(Value::Float(float(0.0)).type_name(), "float");
    assert_eq!(Value::Text(text("")).type_name(), "text");
    assert_eq!(Value::Blob(blob(&[])).type_name(), "blob");
}

#[test]
fn value_display() {
    assert_eq!(format!("{}", Value::Null), "NULL");
    assert_eq!(format!("{}", Value::Integer(int(-5))), "-5");
    assert_eq!(format!("{}", Value::Float(float(2.5))), "2.5");
    assert_eq!(
        format!("{}", Value::Text(text("hello world"))),
        "'hello world'"
    );
    assert_eq!(format!("{}", Value::Text(text("it's"))), "'it''s'");
    assert_eq!(
        format!("{}", Value::Blob(blob(&[1, 2, 3]))),
        "Blob(3 bytes)"
    );
}

#[test]
fn value_from_and_tryfrom() {
    let v: Value = ().into();
    assert_eq!(v, Value::Null);

    let v: Value = int(7).into();
    assert_eq!(v, Value::Integer(int(7)));

    let v: Value = float(1.0).into();
    assert_eq!(v, Value::Float(float(1.0)));

    let v: Value = text("abc").into();
    assert_eq!(v, Value::Text(text("abc")));

    let v: Value = blob(&[0]).into();
    assert_eq!(v, Value::Blob(blob(&[0])));

    let v: Value = 42i64.into();
    assert_eq!(v, Value::Integer(int(42)));

    assert!(matches!(
        Value::try_from(f64::NAN),
        Err(DbError::TypeMismatch(_))
    ));
    assert!(matches!(
        Value::try_from(f64::INFINITY),
        Err(DbError::TypeMismatch(_))
    ));
    assert!(Value::try_from(2.5f64).is_ok());

    let v: Value = String::from("text").into();
    assert_eq!(v, Value::Text(text("text")));

    let v: Value = "text".into();
    assert_eq!(v, Value::Text(text("text")));

    let v: Value = vec![1u8, 2, 3].into();
    assert_eq!(v, Value::Blob(blob(&[1, 2, 3])));

    let v: Value = Value::from(&[1u8, 2, 3][..]);
    assert_eq!(v, Value::Blob(blob(&[1, 2, 3])));
}

#[test]
fn value_partial_eq_and_ord() {
    assert_eq!(Value::Integer(int(1)), Value::Integer(int(1)));
    assert_ne!(Value::Integer(int(1)), Value::Integer(int(2)));
    assert_ne!(Value::Integer(int(1)), Value::Text(text("1")));

    let a = Value::Integer(int(1));
    let b = Value::Integer(int(2));
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));

    let t1 = Value::Text(text("a"));
    let t2 = Value::Text(text("b"));
    assert_eq!(t1.partial_cmp(&t2), Some(Ordering::Less));

    let n = Value::Null;
    let i = Value::Integer(int(1));
    assert_eq!(n.partial_cmp(&i), Some(Ordering::Less));
    assert_eq!(i.partial_cmp(&n), Some(Ordering::Greater));
}

#[test]
fn data_type_as_str_and_display() {
    assert_eq!(DataType::Null.as_str(), "NULL");
    assert_eq!(DataType::Integer.as_str(), "INTEGER");
    assert_eq!(DataType::Float.as_str(), "FLOAT");
    assert_eq!(DataType::Text.as_str(), "TEXT");
    assert_eq!(DataType::Blob.as_str(), "BLOB");

    for dt in [
        DataType::Null,
        DataType::Integer,
        DataType::Float,
        DataType::Text,
        DataType::Blob,
    ] {
        assert_eq!(format!("{dt}"), dt.as_str());
    }
}

#[test]
fn column_def_defaults_and_setters() {
    let mut col = ColumnDef::new("id", DataType::Integer);
    assert_eq!(col.name(), "id");
    assert_eq!(col.data_type(), &DataType::Integer);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());
    assert!(!col.is_unique());

    col.set_nullable(false);
    assert!(!col.is_nullable());

    col.set_nullable(true);
    assert!(col.is_nullable());
    assert!(!col.is_primary_key());

    col.set_primary_key(true);
    assert!(col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());

    col.set_primary_key(false);
    assert!(!col.is_primary_key());
    assert!(!col.is_nullable());
    assert!(col.is_unique());

    col.set_unique(false);
    assert!(!col.is_unique());

    col.set_unique(true);
    assert!(col.is_unique());
    assert!(!col.is_nullable());
    assert!(!col.is_primary_key());
}

#[test]
fn table_schema_try_new_validation() {
    let res = TableSchema::try_new("", vec![ColumnDef::new("a", DataType::Integer)]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = TableSchema::try_new("t", vec![]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = TableSchema::try_new("t", vec![ColumnDef::new("", DataType::Integer)]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("id", DataType::Text),
        ],
    );
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("ID", DataType::Integer),
            ColumnDef::new("id", DataType::Text),
        ],
    );
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(res.is_ok());
    let schema = res.unwrap();
    assert_eq!(schema.name(), "users");
    assert_eq!(schema.columns().len(), 2);
    assert_eq!(schema.column_index("id"), Some(0));
    assert_eq!(schema.column_index("ID"), Some(0));
    assert_eq!(schema.column_index("name"), Some(1));
    assert_eq!(schema.column_index("nonexistent"), None);
    assert_eq!(schema.get_column("id").map(|c| c.name()), Some("id"));
    assert_eq!(schema.get_column("missing"), None);
}

#[test]
fn table_schema_validate_values() {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);
    let schema = TableSchema::try_new("users", vec![id_col, name_col]).unwrap();

    let res = schema.validate_values(&[]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));
    let res = schema.validate_values(&[Value::Integer(int(1))]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = schema.validate_values(&[Value::Null, Value::Text(text("a"))]);
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = schema.validate_values(&[Value::Text(text("x")), Value::Text(text("a"))]);
    assert!(matches!(res, Err(DbError::TypeMismatch(_))));

    let res = schema.validate_values(&[Value::Integer(int(1)), Value::Null]);
    assert!(res.is_ok());

    let null_col = ColumnDef::new("nothing", DataType::Null);
    let schema_null = TableSchema::try_new("t_null", vec![null_col]).unwrap();
    let res = schema_null.validate_values(&[Value::Null]);
    assert!(res.is_ok());
    let res = schema_null.validate_values(&[Value::Integer(int(1))]);
    assert!(matches!(res, Err(DbError::TypeMismatch(_))));
}

#[test]
fn row_basic_operations() {
    let row = Row::new(vec![Value::Integer(int(1)), Value::Text(text("a"))]);
    assert_eq!(row.len(), 2);
    assert!(!row.is_empty());
    assert_eq!(row.get(0), Some(&Value::Integer(int(1))));
    assert_eq!(row.get(1), Some(&Value::Text(text("a"))));
    assert_eq!(row.get(2), None);
    assert_eq!(
        row.values(),
        &[Value::Integer(int(1)), Value::Text(text("a"))]
    );

    let empty = Row::new(vec![]);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());

    let cloned = row.clone();
    assert_eq!(cloned, row);
}

#[test]
fn table_creation_and_insert() {
    let mut pk_col = ColumnDef::new("id", DataType::Integer);
    pk_col.set_primary_key(true);
    let schema = TableSchema::try_new("users", vec![pk_col]).unwrap();
    let mut table = Table::new(schema);

    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
    assert_eq!(table.get(0), None);

    let row = Row::new(vec![Value::Integer(int(1))]);
    assert!(table.insert(row).is_ok());
    assert_eq!(table.len(), 1);
    assert!(!table.is_empty());
    assert_eq!(table.get(0).unwrap().get(0), Some(&Value::Integer(int(1))));
}

#[test]
fn table_insert_validations() {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);
    let schema = TableSchema::try_new("users", vec![id_col, name_col]).unwrap();
    let mut table = Table::new(schema);

    let res = table.insert(Row::new(vec![Value::Integer(int(1))]));
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let res = table.insert(Row::new(vec![
        Value::Text(text("x")),
        Value::Text(text("a")),
    ]));
    assert!(matches!(res, Err(DbError::TypeMismatch(_))));

    let res = table.insert(Row::new(vec![Value::Null, Value::Text(text("a"))]));
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    assert!(
        table
            .insert(Row::new(vec![Value::Integer(int(1)), Value::Null]))
            .is_ok()
    );

    let res = table.insert(Row::new(vec![
        Value::Integer(int(1)),
        Value::Text(text("b")),
    ]));
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));

    let mut unique_col = ColumnDef::new("email", DataType::Text);
    unique_col.set_unique(true);
    let schema2 = TableSchema::try_new("contacts", vec![unique_col]).unwrap();
    let mut table2 = Table::new(schema2);
    assert!(
        table2
            .insert(Row::new(vec![Value::Text(text("a@b.com"))]))
            .is_ok()
    );
    let res = table2.insert(Row::new(vec![Value::Text(text("a@b.com"))]));
    assert!(matches!(res, Err(DbError::InvalidOperation(_))));
}

#[test]
fn catalog_create_and_drop_table() {
    let mut cat = Catalog::new();
    assert_eq!(cat.len(), 0);
    assert!(cat.is_empty());

    let schema =
        TableSchema::try_new("users", vec![ColumnDef::new("id", DataType::Integer)]).unwrap();

    assert!(cat.create_table(schema).is_ok());
    assert_eq!(cat.len(), 1);
    assert!(!cat.is_empty());
    assert!(cat.get_table("users").is_some());

    let schema2 =
        TableSchema::try_new("users", vec![ColumnDef::new("id", DataType::Integer)]).unwrap();
    assert!(matches!(
        cat.create_table(schema2),
        Err(DbError::InvalidOperation(_))
    ));

    assert!(cat.drop_table("users").is_ok());
    assert_eq!(cat.len(), 0);
    assert!(cat.is_empty());
    assert!(cat.get_table("users").is_none());

    assert!(matches!(
        cat.drop_table("missing"),
        Err(DbError::TableNotFound(_))
    ));
}

#[test]
fn catalog_table_access_and_iteration() {
    let mut cat = Catalog::new();
    let schema =
        TableSchema::try_new("users", vec![ColumnDef::new("id", DataType::Integer)]).unwrap();
    cat.create_table(schema).unwrap();

    {
        let table = cat.get_table_mut("users").unwrap();
        let row = Row::new(vec![Value::Integer(int(1))]);
        table.insert(row).unwrap();
    }
    {
        let table = cat.get_table("users").unwrap();
        assert_eq!(table.len(), 1);
    }

    let tables: Vec<(&str, &Table)> = cat.tables().collect();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].0, "users");
    assert_eq!(tables[0].1.len(), 1);
}

#[test]
fn end_to_end_simple_flow() {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);
    let schema = TableSchema::try_new("users", vec![id_col, name_col]).unwrap();

    let mut table = Table::new(schema);
    assert!(
        table
            .insert(Row::new(vec![
                Value::Integer(int(1)),
                Value::Text(text("Alice"))
            ]))
            .is_ok()
    );
    assert!(
        table
            .insert(Row::new(vec![Value::Integer(int(2)), Value::Null]))
            .is_ok()
    );

    assert_eq!(table.len(), 2);
    assert_eq!(
        table.get(0).unwrap().get(1),
        Some(&Value::Text(text("Alice")))
    );
    assert_eq!(table.get(1).unwrap().get(1), Some(&Value::Null));

    let mut cat = Catalog::new();
    cat.create_table(table.schema().clone()).unwrap();
    {
        let t = cat.get_table_mut("users").unwrap();
        assert!(t.is_empty());
    }
}
