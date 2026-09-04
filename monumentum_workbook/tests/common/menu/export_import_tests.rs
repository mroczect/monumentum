use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_workbook::Workbook;
use monumentum_workbook::WorkbookError;
use proptest::prelude::*;
use std::io::Cursor;

// -----------------------------------------------------------------------------

fn create_test_workbook()
-> Result<Workbook<monumentum_db::store::storage::InMemoryStorage>, WorkbookError> {
    let mut wb = Workbook::new_in_memory();

    let cols = vec![
        ColumnDef::new("id", DataType::Integer),
        ColumnDef::new("name", DataType::Text),
        ColumnDef::new("score", DataType::Float),
        ColumnDef::new("active", DataType::Boolean),
        ColumnDef::new("formula", DataType::Text),
    ];
    wb.create_sheet("data", cols)?;

    // Insert the three rows expected by export and roundtrip tests.
    wb.insert_row(
        "data",
        vec![
            Value::from(1_i64),
            Value::from("Alice"),
            Value::try_from(90.5_f64)?,
            Value::Boolean(true),
            Value::from("=SUM(A1:A1)"),
        ],
    )?;

    wb.insert_row(
        "data",
        vec![
            Value::from(2_i64),
            Value::Null,
            Value::try_from(85.0_f64)?,
            Value::Boolean(false),
            Value::from("plain text"),
        ],
    )?;

    wb.insert_row(
        "data",
        vec![
            Value::from(3_i64),
            Value::from("Bob, \"Special\""),
            Value::try_from(75.25_f64)?,
            Value::Boolean(true),
            Value::from("=CONCAT(A1,\",\",B1)"),
        ],
    )?;

    Ok(wb)
}

fn export_csv_to_vec(
    wb: &Workbook<monumentum_db::store::storage::InMemoryStorage>,
    sheet: &str,
) -> Result<Vec<u8>, WorkbookError> {
    let mut buf = Vec::new();
    wb.export_csv(sheet, &mut buf)?;
    Ok(buf)
}

fn export_json_to_vec(
    wb: &Workbook<monumentum_db::store::storage::InMemoryStorage>,
    sheet: &str,
) -> Result<Vec<u8>, WorkbookError> {
    let mut buf = Vec::new();
    wb.export_json(sheet, &mut buf)?;
    Ok(buf)
}

#[test]
fn export_csv_basic_structure() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    let cols = vec![
        ColumnDef::new("id", DataType::Integer),
        ColumnDef::new("name", DataType::Text),
        ColumnDef::new("score", DataType::Float),
        ColumnDef::new("active", DataType::Boolean),
        ColumnDef::new("formula", DataType::Text),
    ];
    wb.create_sheet("data", cols)?;

    wb.insert_row(
        "data",
        vec![
            Value::from(1_i64),
            Value::from("Alice"),
            Value::try_from(90.5_f64)?,
            Value::Boolean(true),
            Value::from("=SUM(A1:A1)"),
        ],
    )?;

    wb.insert_row(
        "data",
        vec![
            Value::from(2_i64),
            Value::Null,
            Value::try_from(85.0_f64)?,
            Value::Boolean(false),
            Value::from("plain text"),
        ],
    )?;

    wb.insert_row(
        "data",
        vec![
            Value::from(3_i64),
            Value::from("Bob, \"Special\""),
            Value::try_from(75.25_f64)?,
            Value::Boolean(true),
            Value::from("=CONCAT(A1,\",\",B1)"),
        ],
    )?;

    let data = export_csv_to_vec(&wb, "data")?;
    let content = String::from_utf8(data).map_err(|e| {
        WorkbookError::Db(DbError::invalid_operation(format!("invalid UTF-8: {e}")))
    })?;

    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 4);

    let first_line = lines.first().ok_or(WorkbookError::InvalidReference)?;
    assert!(first_line.starts_with("id,name,score,active,formula"));

    let second_line = lines.get(1).ok_or(WorkbookError::InvalidReference)?;
    assert!(second_line.contains("1,Alice,90.5,true,"));

    let third_line = lines.get(2).ok_or(WorkbookError::InvalidReference)?;
    assert!(third_line.contains("2,,85,false,"));

    let fourth_line = lines.get(3).ok_or(WorkbookError::InvalidReference)?;
    assert!(fourth_line.contains("3,\"Bob, \"\"Special\"\"\",75.25,true,"));

    Ok(())
}

#[test]
fn export_csv_sheet_not_found() {
    let wb = Workbook::new_in_memory();
    let result = export_csv_to_vec(&wb, "missing");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::TableNotFound(_))));
    }
}

#[test]
fn export_csv_blob_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("blob_sheet", vec![ColumnDef::new("data", DataType::Blob)])?;
    wb.insert_row("blob_sheet", vec![Value::from(vec![1_u8, 2, 3])])?;

    let mut buf = Vec::new();
    let result = wb.export_csv("blob_sheet", &mut buf);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::Unsupported(_))));
    }
    Ok(())
}

#[test]
fn import_csv_valid_data_roundtrip() -> Result<(), WorkbookError> {
    let mut wb = create_test_workbook()?;
    let original_data = wb.get_cell_value("data", 0, 1)?;
    let export_buf = export_csv_to_vec(&wb, "data")?;

    wb.clear_sheet("data")?;
    wb.import_csv("data", Cursor::new(export_buf))?;

    let imported = wb.get_cell_value("data", 0, 1)?;
    assert_eq!(original_data, imported);
    Ok(())
}

#[test]
fn import_csv_header_mismatch_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("sheet", vec![ColumnDef::new("id", DataType::Integer)])?;

    let csv_data = b"wrong_name\n1\n";
    let result = wb.import_csv("sheet", Cursor::new(csv_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::InvalidOperation(_))));
    }
    Ok(())
}

#[test]
fn import_csv_wrong_column_count_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet(
        "sheet",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;

    let csv_data = b"id,name\n1\n";
    let result = wb.import_csv("sheet", Cursor::new(csv_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::InvalidOperation(_))));
    }
    Ok(())
}

#[test]
fn import_csv_empty_input_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("sheet", vec![ColumnDef::new("id", DataType::Integer)])?;

    let csv_data = b"";
    let result = wb.import_csv("sheet", Cursor::new(csv_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::InvalidOperation(_))));
    }
    Ok(())
}

#[test]
fn import_csv_integer_parsing_errors() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("sheet", vec![ColumnDef::new("id", DataType::Integer)])?;

    let csv_data = b"id\nnot_an_int\n";
    let result = wb.import_csv("sheet", Cursor::new(csv_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::TypeMismatch(_))));
    }
    Ok(())
}

#[test]
fn export_json_basic_structure() -> Result<(), WorkbookError> {
    let wb = create_test_workbook()?;
    let data = export_json_to_vec(&wb, "data")?;
    let content = String::from_utf8(data).map_err(|e| {
        WorkbookError::Db(DbError::invalid_operation(format!("invalid UTF-8: {e}")))
    })?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| WorkbookError::Db(DbError::invalid_operation(format!("invalid JSON: {e}"))))?;

    let arr = json.as_array().ok_or(WorkbookError::InvalidArgument)?;
    assert_eq!(arr.len(), 3);

    let first = arr.first().ok_or(WorkbookError::InvalidReference)?;
    let first_obj = first.as_object().ok_or(WorkbookError::InvalidArgument)?;

    assert_eq!(first_obj.get("id"), Some(&serde_json::Value::from(1_i64)));
    assert_eq!(
        first_obj.get("name"),
        Some(&serde_json::Value::String("Alice".to_string()))
    );
    assert_eq!(
        first_obj.get("score"),
        Some(&serde_json::Value::from(90.5_f64))
    );
    assert_eq!(
        first_obj.get("active"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        first_obj.get("formula"),
        Some(&serde_json::Value::String("=SUM(A1:A1)".to_string()))
    );
    Ok(())
}

#[test]
fn export_json_blob_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("blob_sheet", vec![ColumnDef::new("data", DataType::Blob)])?;
    wb.insert_row("blob_sheet", vec![Value::from(vec![1_u8, 2, 3])])?;

    let mut buf = Vec::new();
    let result = wb.export_json("blob_sheet", &mut buf);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::Unsupported(_))));
    }
    Ok(())
}

#[test]
fn import_json_valid_data_roundtrip() -> Result<(), WorkbookError> {
    let mut wb = create_test_workbook()?;
    let original_name = wb.get_cell_value("data", 0, 1)?;
    let export_buf = export_json_to_vec(&wb, "data")?;

    wb.clear_sheet("data")?;
    wb.import_json("data", Cursor::new(export_buf))?;

    let imported_name = wb.get_cell_value("data", 0, 1)?;
    assert_eq!(original_name, imported_name);
    Ok(())
}

#[test]
fn import_json_root_not_array_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("sheet", vec![ColumnDef::new("id", DataType::Integer)])?;

    let json_data = br#"{"id": 1}"#;
    let result = wb.import_json("sheet", Cursor::new(json_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::InvalidOperation(_))));
    }
    Ok(())
}

#[test]
fn import_json_missing_field_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet(
        "sheet",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;

    let json_data = br#"[{"id": 1}]"#;
    let result = wb.import_json("sheet", Cursor::new(json_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::InvalidOperation(_))));
    }
    Ok(())
}

#[test]
fn import_json_type_mismatch_returns_error() -> Result<(), WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet("sheet", vec![ColumnDef::new("id", DataType::Integer)])?;

    let json_data = br#"[{"id": "not_an_int"}]"#;
    let result = wb.import_json("sheet", Cursor::new(json_data));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, WorkbookError::Db(DbError::TypeMismatch(_))));
    }
    Ok(())
}

fn build_workbook(
    values: &[(i64, f64, bool, String)],
) -> Result<Workbook<monumentum_db::store::storage::InMemoryStorage>, WorkbookError> {
    let mut wb = Workbook::new_in_memory();
    wb.create_sheet(
        "test",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("score", DataType::Float),
            ColumnDef::new("active", DataType::Boolean),
            ColumnDef::new("comment", DataType::Text),
        ],
    )?;

    for (id, score, active, comment) in values {
        wb.insert_row(
            "test",
            vec![
                Value::from(*id),
                Value::try_from(*score)?,
                Value::Boolean(*active),
                Value::from(comment.as_str()),
            ],
        )?;
    }
    Ok(wb)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn csv_roundtrip_preserves_values(
        values in proptest::collection::vec(
            (
                any::<i64>(),
                any::<f64>().prop_filter("finite", |f| f.is_finite()),
                any::<bool>(),
                ".*".prop_filter("non-empty and not formula", |s| !s.is_empty() && !s.starts_with('=')),
            ),
            1..10,
        )
    ) {
        let Ok(mut wb) = build_workbook(&values) else {
            prop_assert!(false, "failed to build workbook");
            return Ok(());
        };

        let Ok(export_buf) = export_csv_to_vec(&wb, "test") else {
            prop_assert!(false, "export failed");
            return Ok(());
        };

        if wb.clear_sheet("test").is_err() {
            prop_assert!(false, "clear failed");
            return Ok(());
        }

        if wb.import_csv("test", Cursor::new(export_buf)).is_err() {
            prop_assert!(false, "import failed");
            return Ok(());
        }

        for (i, (id, score, active, comment)) in values.into_iter().enumerate() {
            let Ok(id_val) = wb.get_cell_value("test", i, 0) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(id_val, Value::from(id));

            let Ok(score_val) = wb.get_cell_value("test", i, 1) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            if let Value::Float(f) = score_val {
                let a = f.as_f64();
                let b = score;
                if a == 0.0 || b == 0.0 {
                    prop_assert!((a - b).abs() < 1e-12);
                } else {
                    let rel = (a - b).abs() / a.abs().max(b.abs());
                    prop_assert!(rel < 1e-12, "float mismatch: {} vs {}", a, b);
                }
            } else {
                prop_assert!(false, "expected float");
                return Ok(());
            }

            let Ok(active_val) = wb.get_cell_value("test", i, 2) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(active_val, Value::Boolean(active));

            let Ok(comment_val) = wb.get_cell_value("test", i, 3) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(comment_val, Value::from(comment.as_str()));
        }
    }

    #[test]
    fn json_roundtrip_preserves_values(
        values in proptest::collection::vec(
            (
                any::<i64>(),
                any::<f64>().prop_filter("finite", |f| f.is_finite()),
                any::<bool>(),
                ".*".prop_filter("non-empty and not formula", |s| !s.is_empty() && !s.starts_with('=')),
            ),
            1..10,
        )
    ) {
        let Ok(mut wb) = build_workbook(&values) else {
            prop_assert!(false, "failed to build workbook");
            return Ok(());
        };

        let Ok(export_buf) = export_json_to_vec(&wb, "test") else {
            prop_assert!(false, "export failed");
            return Ok(());
        };

        if wb.clear_sheet("test").is_err() {
            prop_assert!(false, "clear failed");
            return Ok(());
        }

        if wb.import_json("test", Cursor::new(export_buf)).is_err() {
            prop_assert!(false, "import failed");
            return Ok(());
        }

        for (i, (id, score, active, comment)) in values.into_iter().enumerate() {
            let Ok(id_val) = wb.get_cell_value("test", i, 0) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(id_val, Value::from(id));

            let Ok(score_val) = wb.get_cell_value("test", i, 1) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            if let Value::Float(f) = score_val {
                let a = f.as_f64();
                let b = score;
                if a == 0.0 || b == 0.0 {
                    prop_assert!((a - b).abs() < 1e-12);
                } else {
                    let rel = (a - b).abs() / a.abs().max(b.abs());
                    prop_assert!(rel < 1e-12, "float mismatch: {} vs {}", a, b);
                }
            } else {
                prop_assert!(false, "expected float");
                return Ok(());
            }

            let Ok(active_val) = wb.get_cell_value("test", i, 2) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(active_val, Value::Boolean(active));

            let Ok(comment_val) = wb.get_cell_value("test", i, 3) else {
                prop_assert!(false, "cell not found");
                return Ok(());
            };
            prop_assert_eq!(comment_val, Value::from(comment.as_str()));
        }
    }
}
