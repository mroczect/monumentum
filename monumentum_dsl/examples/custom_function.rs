use core::error::Error;
use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::{FunctionRegistry, QueryBuilder, ScalarFunction};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;

#[derive(Debug)]
struct ConcatFunction;

impl ScalarFunction for ConcatFunction {
    fn name(&self) -> &str {
        "concat"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let mut result = String::new();
        for arg in args {
            if let Some(s) = arg.as_str() {
                result.push_str(s);
            } else {
                return Err(DbError::type_mismatch("expected text"));
            }
        }
        Value::try_from(result).map_err(Into::into)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join("custom_function.db");
    let mut storage = FileStorage::open(&path, 10)?;

    let schema = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("first", DataType::Text),
            ColumnDef::new("last", DataType::Text),
        ],
    )?;
    storage.create_table(schema)?;

    storage.insert_row(
        "users",
        &Row::new(vec![
            Value::try_from("John".to_string())?,
            Value::try_from("Doe".to_string())?,
        ]),
    )?;

    let mut registry = FunctionRegistry::new();
    registry.register_scalar(Box::new(ConcatFunction));

    let names: Vec<Value> = QueryBuilder::new(&mut storage, "users")
        .project(|row| {
            let first = row
                .get(0)
                .ok_or_else(|| DbError::type_mismatch("missing first"))?
                .clone();
            let last = row
                .get(1)
                .ok_or_else(|| DbError::type_mismatch("missing last"))?
                .clone();
            let concat = registry
                .get_scalar("concat")
                .ok_or_else(|| DbError::unsupported("missing concat function"))?;
            concat.call(&[first, last])
        })?
        .execute()?;

    for name in names {
        println!("{}", name.as_str().unwrap_or(""));
    }

    storage.checkpoint()?;
    storage.close()?;
    Ok(())
}
