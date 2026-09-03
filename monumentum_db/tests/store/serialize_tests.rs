use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::store::serde::{decode_catalog, encode_catalog};

fn create_schema_with_all_types() -> Result<TableSchema, DbError> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);

    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(true);

    let mut price_col = ColumnDef::new("price", DataType::Float);
    price_col.set_nullable(true);

    let mut data_col = ColumnDef::new("data", DataType::Blob);
    data_col.set_nullable(true);

    TableSchema::try_new("products", vec![id_col, name_col, price_col, data_col])
}

fn create_sample_catalog() -> Result<Catalog, DbError> {
    let mut catalog = Catalog::new();
    let schema = create_schema_with_all_types()?;
    catalog.create_table(schema)?;

    if let Some(table) = catalog.get_table_mut("products") {
        let row1 = Row::new(vec![
            Value::from(1_i64),
            Value::from("Apple"),
            Value::try_from(9.99)?,
            Value::from(vec![1_u8, 2, 3]),
        ]);
        table.insert(row1)?;

        let row2 = Row::new(vec![
            Value::from(2_i64),
            Value::Null,
            Value::Null,
            Value::Null,
        ]);
        table.insert(row2)?;
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
fn encode_decode_catalog_with_tables_roundtrip() -> Result<(), DbError> {
    let catalog = create_sample_catalog()?;
    let bytes = encode_catalog(&catalog)?;
    let decoded = decode_catalog(&bytes)?;
    assert_eq!(catalog, decoded);
    Ok(())
}

#[test]
fn decode_catalog_unsupported_version_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&999u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

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
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&1025u32.to_le_bytes());

    let result = decode_catalog(&data);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: too many tables");
    }
}

#[test]
fn decode_catalog_truncated_data_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes());

    let result = decode_catalog(&data);
    assert!(result.is_err());
}
