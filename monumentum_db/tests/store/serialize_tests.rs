use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{
    CheckConstraint, ColumnDef, ComparisonOp, DataType, ForeignKey,
};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::store::serde::{decode_catalog, encode_catalog};
use proptest::prelude::*;

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn minimal_integer_catalog_bytes() -> Vec<u8> {
    let col = ColumnDef::new("c", DataType::Integer);
    let schema = TableSchema::try_new("t", vec![col]).unwrap();
    let mut catalog = Catalog::new();
    catalog.create_table(schema).unwrap();
    encode_catalog(&catalog).unwrap()
}

fn catalog_with_check_constraint_bytes() -> Vec<u8> {
    let mut col = ColumnDef::new("c", DataType::Integer);
    col.set_check(Some(CheckConstraint {
        column: "c".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    }));
    let schema = TableSchema::try_new("t", vec![col]).unwrap();
    let mut catalog = Catalog::new();
    catalog.create_table(schema).unwrap();
    encode_catalog(&catalog).unwrap()
}

fn catalog_with_integer_value_bytes() -> Vec<u8> {
    let col = ColumnDef::new("id", DataType::Integer);
    let schema = TableSchema::try_new("t", vec![col]).unwrap();
    let mut catalog = Catalog::new();
    catalog.create_table(schema).unwrap();
    if let Some(table) = catalog.get_table_mut("t") {
        table.insert(Row::new(vec![Value::from(1_i64)])).unwrap();
    }
    encode_catalog(&catalog).unwrap()
}

fn catalog_with_float_value_bytes() -> Vec<u8> {
    let col = ColumnDef::new("v", DataType::Float);
    let schema = TableSchema::try_new("t", vec![col]).unwrap();
    let mut catalog = Catalog::new();
    catalog.create_table(schema).unwrap();
    if let Some(table) = catalog.get_table_mut("t") {
        table
            .insert(Row::new(vec![Value::try_from(1.0_f64).unwrap()]))
            .unwrap();
    }
    encode_catalog(&catalog).unwrap()
}

fn create_catalog_with_table(
    table_name: &str,
    columns: Vec<ColumnDef>,
    rows: Vec<Row>,
) -> Result<Catalog, DbError> {
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new(table_name, columns)?;
    catalog.create_table(schema)?;
    if let Some(table) = catalog.get_table_mut(table_name) {
        for row in rows {
            table.insert(row)?;
        }
    }
    Ok(catalog)
}

#[test]
fn encode_decode_catalog_empty_roundtrip() -> Result<(), DbError> {
    let catalog = Catalog::new();
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn roundtrip_catalog_with_boolean_and_formula() -> Result<(), DbError> {
    let col_bool = ColumnDef::new("flag", DataType::Boolean);
    let col_formula = ColumnDef::new("expr", DataType::Integer);
    let rows = vec![
        Row::new(vec![
            Value::Boolean(true),
            Value::Formula("1+1".to_string()),
        ]),
        Row::new(vec![Value::Boolean(false), Value::Integer(42.into())]),
    ];
    let catalog = create_catalog_with_table("t", vec![col_bool, col_formula], rows)?;
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn roundtrip_catalog_with_full_column_constraints() -> Result<(), DbError> {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_primary_key(true);
    col.set_default(Some(Value::from(0_i64)));
    col.set_check(Some(CheckConstraint {
        column: "id".to_string(),
        op: ComparisonOp::Gte,
        value: Value::from(0_i64),
    }));
    col.set_foreign_key(Some(ForeignKey {
        table: "other".to_string(),
        column: "id".to_string(),
    }));
    col.set_allowed_values(Some(vec![Value::from(0_i64), Value::from(1_i64)]));

    let rows = vec![Row::new(vec![Value::from(1_i64)])];
    let catalog = create_catalog_with_table("t", vec![col], rows)?;
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn roundtrip_catalog_with_read_only_table() -> Result<(), DbError> {
    let col = ColumnDef::new("id", DataType::Integer);
    let mut catalog = Catalog::new();
    let schema = TableSchema::try_new("t", vec![col])?;
    catalog.create_table(schema)?;
    if let Some(table) = catalog.get_table_mut("t") {
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.set_read_only(true);
    }
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn roundtrip_catalog_with_many_tables_and_unicode_names() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    for i in 0..50 {
        let name = format!("таблица_{i}");
        let col = ColumnDef::new("колонка", DataType::Text);
        let schema = TableSchema::try_new(name.clone(), vec![col])?;
        catalog.create_table(schema)?;
        if let Some(table) = catalog.get_table_mut(&name) {
            table.insert(Row::new(vec![Value::from("значение")]))?;
        }
    }
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn roundtrip_catalog_with_long_names() -> Result<(), DbError> {
    let long_name = "t".repeat(10_000);
    let long_col = "c".repeat(10_000);
    let col = ColumnDef::new(long_col.clone(), DataType::Text);
    let schema = TableSchema::try_new(long_name.clone(), vec![col])?;
    let mut catalog = Catalog::new();
    catalog.create_table(schema)?;
    if let Some(table) = catalog.get_table_mut(&long_name) {
        table.insert(Row::new(vec![Value::from("data")]))?;
    }
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn decode_catalog_unsupported_version_returns_error() {
    let mut data = Vec::new();
    write_u32(&mut data, 999);
    write_u32(&mut data, 0);
    let result = decode_catalog(&data);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: unsupported format version 999"
        );
    }
}

#[test]
fn decode_catalog_too_many_tables_returns_error() {
    let mut data = Vec::new();
    write_u32(&mut data, 1);
    write_u32(&mut data, 1025);
    let result = decode_catalog(&data);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: too many tables");
    }
}

#[test]
fn decode_catalog_utf8_error() {
    let mut data = Vec::new();
    write_u32(&mut data, 1);
    write_u32(&mut data, 1);
    write_u64(&mut data, 2);
    data.extend_from_slice(&[0xFF, 0xFE]);
    let result = decode_catalog(&data);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("invalid utf-8"));
    }
}

#[test]
fn decode_catalog_length_exceeds_max() {
    let mut data = Vec::new();
    write_u32(&mut data, 1);
    write_u32(&mut data, 1);
    write_u64(&mut data, u64::MAX);
    data.push(0);
    let result = decode_catalog(&data);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("declared length"));
    }
}

#[test]
fn decode_catalog_truncated_at_various_points() {
    let valid = {
        let catalog = Catalog::new();
        encode_catalog(&catalog).unwrap()
    };
    for len in 0..valid.len() {
        let truncated = &valid[..len];
        let result = decode_catalog(truncated);
        assert!(
            result.is_err(),
            "decoding truncated data at len {len} should fail"
        );
    }
}

#[test]
fn decode_catalog_invalid_data_type_tag() {
    let mut bytes = minimal_integer_catalog_bytes();
    bytes[39] = 6;
    let result = decode_catalog(&bytes);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: invalid data type tag");
    }
}

#[test]
fn decode_catalog_invalid_comparison_op_tag() {
    let mut bytes = catalog_with_check_constraint_bytes();
    bytes[54] = 6;
    let result = decode_catalog(&bytes);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: invalid comparison op tag");
    }
}

#[test]
fn decode_catalog_invalid_value_tag() {
    let mut bytes = catalog_with_integer_value_bytes();
    let tag_pos = 57;
    bytes[tag_pos] = 7;

    let result = decode_catalog(&bytes);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: invalid value tag");
    }
}

#[test]
fn decode_catalog_float_nan_returns_error() {
    let mut bytes = catalog_with_float_value_bytes();
    let one = 1.0_f64.to_bits();
    let pos = bytes
        .windows(8)
        .position(|w| u64::from_le_bytes(w.try_into().unwrap()) == one)
        .expect("float 1.0 not found");
    let nan_bits = f64::NAN.to_bits();
    bytes[pos..pos + 8].copy_from_slice(&nan_bits.to_le_bytes());

    let result = decode_catalog(&bytes);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("float must be finite"));
    }
}

fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<i64>().prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        ".*".prop_map(Value::from),
        prop::collection::vec(any::<u8>(), 0..10).prop_map(Value::from),
        any::<f64>()
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(|f| Value::try_from(f).unwrap()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn catalog_roundtrip_with_single_table_single_column(
        values in prop::collection::vec(value_strategy(), 0..20),
    ) {
        let col = ColumnDef::new("col", DataType::Text);
        let text_values: Vec<Value> = values.into_iter()
            .filter(|v| v.is_text() || v.is_null())
            .collect();
        let rows: Vec<Row> = text_values.into_iter().map(|v| Row::new(vec![v])).collect();

        let mut catalog = Catalog::new();
        let schema = TableSchema::try_new("t", vec![col]).unwrap();
        catalog.create_table(schema).unwrap();
        if let Some(table) = catalog.get_table_mut("t") {
            for row in rows {
                table.insert(row).unwrap();
            }
        }

        let bytes = encode_catalog(&catalog).unwrap();
        let decoded = decode_catalog(&bytes).unwrap();
        prop_assert_eq!(catalog, decoded);
    }

    #[test]
    fn catalog_roundtrip_with_integer_column(
        values in prop::collection::vec(any::<i64>(), 0..20),
    ) {
        let col = ColumnDef::new("id", DataType::Integer);
        let rows: Vec<Row> = values.iter().map(|&v| Row::new(vec![Value::from(v)])).collect();

        let mut catalog = Catalog::new();
        let schema = TableSchema::try_new("t", vec![col]).unwrap();
        catalog.create_table(schema).unwrap();
        if let Some(table) = catalog.get_table_mut("t") {
            for row in rows {
                table.insert(row).unwrap();
            }
        }

        let bytes = encode_catalog(&catalog).unwrap();
        let decoded = decode_catalog(&bytes).unwrap();
        prop_assert_eq!(catalog, decoded);
    }
}
