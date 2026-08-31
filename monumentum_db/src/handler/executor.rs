use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::DataType;
use crate::core::schema::table_schema::TableSchema;
use crate::core::value::Value;
use crate::error::DbError;
use crate::handler::result::QueryResult;
use crate::handler::statement::{Statement, WhereClause};
use crate::handler::traits::Executor;

pub struct SimpleExecutor<'a> {
    catalog: &'a mut Catalog,
}

impl<'a> SimpleExecutor<'a> {
    #[must_use]
    pub fn new(catalog: &'a mut Catalog) -> Self {
        Self { catalog }
    }
}

impl Executor for SimpleExecutor<'_> {
    fn execute(&mut self, statement: Statement) -> Result<QueryResult, DbError> {
        match statement {
            Statement::CreateTable(schema) => {
                self.catalog.create_table(schema)?;
                Ok(QueryResult::Empty)
            }
            Statement::DropTable(name) => {
                self.catalog.drop_table(&name)?;
                Ok(QueryResult::Empty)
            }
            Statement::Insert { table, values } => {
                let table_ref = self
                    .catalog
                    .get_table_mut(&table)
                    .ok_or_else(|| DbError::table_not_found(&table))?;
                let row = Row::new(values);
                table_ref.insert(row)?;
                Ok(QueryResult::AffectedRows(1))
            }
            Statement::Select {
                table,
                columns,
                where_clause,
            } => {
                let table_ref = self
                    .catalog
                    .get_table(&table)
                    .ok_or_else(|| DbError::table_not_found(&table))?;
                let schema = table_ref.schema();

                let column_indices: Vec<usize> = if columns.is_empty() {
                    (0..schema.columns().len()).collect()
                } else {
                    columns
                        .iter()
                        .map(|col| {
                            schema
                                .column_index(col)
                                .ok_or_else(|| DbError::column_not_found(col))
                        })
                        .collect::<Result<_, _>>()?
                };

                let mut selected_rows = Vec::new();
                for row in table_ref.rows() {
                    if let Some(ref wc) = where_clause
                        && !matches_where(row, schema, wc)?
                    {
                        continue;
                    }
                    let mut new_row_values = Vec::with_capacity(column_indices.len());
                    for &idx in &column_indices {
                        let value = row.get(idx).cloned().unwrap_or(Value::Null);
                        new_row_values.push(value);
                    }
                    selected_rows.push(Row::new(new_row_values));
                }

                Ok(QueryResult::Rows(selected_rows))
            }
            Statement::Update {
                table,
                assignments,
                where_clause,
            } => {
                let table_ref = self
                    .catalog
                    .get_table_mut(&table)
                    .ok_or_else(|| DbError::table_not_found(&table))?;
                let schema = table_ref.schema().clone();

                let mut affected = 0;
                let mut new_rows = Vec::with_capacity(table_ref.len());

                for row in table_ref.rows() {
                    let should_update = if let Some(ref wc) = where_clause {
                        matches_where(row, &schema, wc)?
                    } else {
                        true
                    };

                    if should_update {
                        let mut new_values = row.values().to_vec();
                        for (col_name, new_val) in &assignments {
                            let idx = schema
                                .column_index(col_name)
                                .ok_or_else(|| DbError::column_not_found(col_name))?;
                            let col_def = &schema.columns()[idx];
                            if !new_val.is_null() {
                                let type_ok = match col_def.data_type() {
                                    DataType::Null => false,
                                    DataType::Integer => new_val.is_integer(),
                                    DataType::Float => new_val.is_float(),
                                    DataType::Text => new_val.is_text(),
                                    DataType::Blob => new_val.is_blob(),
                                };
                                if !type_ok {
                                    return Err(DbError::type_mismatch(format!(
                                        "column '{}' expects {}",
                                        col_name,
                                        col_def.data_type()
                                    )));
                                }
                            }
                            new_values[idx] = new_val.clone();
                        }
                        schema.validate_values(&new_values)?;
                        new_rows.push(Row::new(new_values));
                        affected += 1;
                    } else {
                        new_rows.push(row.clone());
                    }
                }

                for (idx, col) in schema.columns().iter().enumerate() {
                    if col.is_unique() || col.is_primary_key() {
                        let mut seen_values: Vec<&Value> = Vec::new();
                        for row in &new_rows {
                            let val = row.get(idx).unwrap_or(&Value::Null);
                            if !val.is_null() && seen_values.contains(&val) {
                                return Err(DbError::invalid_operation(format!(
                                    "duplicate value for column '{}'",
                                    col.name()
                                )));
                            }
                            seen_values.push(val);
                        }
                    }
                }

                table_ref.replace_rows(new_rows);
                Ok(QueryResult::AffectedRows(affected))
            }
            Statement::Delete {
                table,
                where_clause,
            } => {
                let table_ref = self
                    .catalog
                    .get_table_mut(&table)
                    .ok_or_else(|| DbError::table_not_found(&table))?;
                let schema = table_ref.schema().clone();

                let mut affected = 0;
                let mut remaining_rows = Vec::with_capacity(table_ref.len());

                for row in table_ref.rows() {
                    let should_delete = if let Some(ref wc) = where_clause {
                        matches_where(row, &schema, wc)?
                    } else {
                        true
                    };

                    if should_delete {
                        affected += 1;
                    } else {
                        remaining_rows.push(row.clone());
                    }
                }

                table_ref.replace_rows(remaining_rows);
                Ok(QueryResult::AffectedRows(affected))
            }
        }
    }
}

fn matches_where(row: &Row, schema: &TableSchema, wc: &WhereClause) -> Result<bool, DbError> {
    let idx = schema
        .column_index(&wc.column)
        .ok_or_else(|| DbError::column_not_found(&wc.column))?;
    let row_val = row.get(idx).unwrap_or(&Value::Null);
    Ok(row_val == &wc.value)
}
