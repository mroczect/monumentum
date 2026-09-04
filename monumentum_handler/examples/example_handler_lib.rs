use core::error::Error;
use core::fmt::Write as _;

use monumentum_handler::{
    ColumnDef, DataType, DbError, Row, TableSchema, Value,
    core::schema::column::{CheckConstraint, ComparisonOp},
};

fn text_value(s: &str) -> Result<Value, DbError> {
    let text = monumentum_handler::Text::try_new(s.to_string())?;
    Ok(Value::from(text))
}

fn blob_value(data: &[u8]) -> Result<Value, DbError> {
    let blob = monumentum_handler::Blob::try_new(data.to_vec())?;
    Ok(Value::from(blob))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);

    let mut name_col = ColumnDef::new("name", DataType::Text);
    name_col.set_nullable(false);

    let mut age_col = ColumnDef::new("age", DataType::Integer);
    age_col.set_check(Some(CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    }));

    let salary_col = ColumnDef::new("salary", DataType::Float);

    let mut active_col = ColumnDef::new("active", DataType::Boolean);
    active_col.set_default(Some(Value::from(true)));

    let bio_col = ColumnDef::new("bio", DataType::Blob);

    let schema = TableSchema::try_new(
        "employees",
        vec![id_col, name_col, age_col, salary_col, active_col, bio_col],
    )?;

    let employees = [
        (
            1_i64,
            "Alice",
            30_i64,
            Some(7500.50_f64),
            true,
            Some(b"hello".as_slice()),
        ),
        (2_i64, "Bob", 25_i64, Some(6200.0_f64), false, None),
        (
            3_i64,
            "Charlie",
            35_i64,
            None,
            true,
            Some(b"world".as_slice()),
        ),
    ];

    let mut rows: Vec<Row> = Vec::new();

    for (id, name, age, salary, active, bio) in employees {
        let mut values = Vec::with_capacity(6);

        values.push(Value::from(id));

        let name_val = text_value(name)?;
        values.push(name_val);

        values.push(Value::from(age));

        match salary {
            Some(s) => values.push(Value::try_from(s)?),
            None => values.push(Value::Null),
        }

        values.push(Value::from(active));

        match bio {
            Some(b) => values.push(blob_value(b)?),
            None => values.push(Value::Null),
        }

        schema.validate_values(&values)?;

        rows.push(Row::new(values));
    }

    let mut output = String::new();
    writeln!(&mut output, "Laporan Karyawan")?;
    writeln!(
        &mut output,
        "{:<5} {:<15} {:<5} {:<10} {:<6} {:<10}",
        "ID", "Nama", "Umur", "Gaji", "Aktif", "Bio"
    )?;
    writeln!(&mut output, "{}", "-".repeat(60))?;

    let mut total_salary = 0.0_f64;
    for row in &rows {
        let id = row.get(&0).and_then(Value::as_i64).unwrap_or(0);
        let name = row.get(&1).and_then(Value::as_str).unwrap_or("?");
        let age = row.get(&2).and_then(Value::as_i64).unwrap_or(0);
        let salary = row.get(&3).and_then(Value::as_f64).unwrap_or(0.0);
        let active = row.get(&4).and_then(Value::as_bool).unwrap_or(false);
        let bio = row
            .get(&5)
            .and_then(Value::as_blob)
            .map_or_else(|| "NULL".to_string(), |b| format!("{} bytes", b.len()));

        writeln!(
            &mut output,
            "{:<5} {:<15} {:<5} {:<10.2} {:<6} {:<10}",
            id, name, age, salary, active, bio
        )?;
        total_salary += salary;
    }

    writeln!(&mut output, "{}", "-".repeat(60))?;
    writeln!(&mut output, "Total gaji: {:.2}", total_salary)?;
    writeln!(&mut output, "Jumlah karyawan: {}", rows.len())?;

    print!("{output}");

    let bad_values = vec![
        Value::from(4_i64),
        text_value("David")?,
        Value::from(-5_i64),
        Value::try_from(5000.0_f64)?,
        Value::from(true),
        Value::Null,
    ];
    if let Err(e) = schema.validate_values(&bad_values) {
        eprintln!("Validasi gagal seperti yang diharapkan: {e}");
    } else {
        eprintln!("Error: validasi seharusnya gagal untuk umur negatif");
    }

    Ok(())
}
