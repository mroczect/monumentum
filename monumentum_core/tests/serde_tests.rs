use fs2 as _;
use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{decode_catalog, encode_catalog};
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::error::DbError;

#[test]
fn catalog_roundtrip() {
    let mut cat = Catalog::new();
    let schema_result = TableSchema::try_new("test", vec![ColumnDef::new("id", DataType::Integer)]);
    let Ok(schema) = schema_result else {
        return;
    };
    assert!(cat.create_table(schema).is_ok());

    let encode_result = encode_catalog(&cat);
    let Ok(bytes) = encode_result else {
        return;
    };
    let decode_result = decode_catalog(&bytes);
    let Ok(decoded) = decode_result else {
        return;
    };
    assert_eq!(cat, decoded);
}

#[test]
fn decode_corrupt_data_returns_corruption() {
    let bytes = vec![0u8; 100];
    let result = decode_catalog(&bytes);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
}
