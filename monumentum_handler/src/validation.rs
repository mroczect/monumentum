use crate::constants::MAX_NAME_LENGTH;
use crate::error::DbError;

pub fn validate_name(name: &str) -> Result<(), DbError> {
    if name.is_empty() {
        return Err(DbError::invalid_operation("name cannot be empty"));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(DbError::invalid_operation(format!(
            "name too long: {} bytes (max {})",
            name.len(),
            MAX_NAME_LENGTH
        )));
    }
    if name.chars().any(char::is_control) {
        return Err(DbError::invalid_operation(
            "name contains control characters",
        ));
    }
    Ok(())
}

pub fn validate_column_name(name: &str) -> Result<(), DbError> {
    validate_name(name).map_err(|e| DbError::invalid_operation(format!("invalid column name: {e}")))
}

pub fn validate_table_name(name: &str) -> Result<(), DbError> {
    validate_name(name).map_err(|e| DbError::invalid_operation(format!("invalid table name: {e}")))
}
