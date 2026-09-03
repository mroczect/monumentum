use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::FileStorage;
use monumentum_functions as _;
use monumentum_query as _;
use monumentum_query::formula::FormulaError;
use monumentum_workbook::Workbook;
use monumentum_workbook::transaction::Transaction;
use pretty_assertions as _;
use proptest as _;
use tempfile as _;

fn custom_double(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [Value::Integer(i)] => Ok(Value::from(i.as_i64().saturating_mul(2))),
        [Value::Float(f)] => {
            let result = f.as_f64() * 2.0;
            monumentum_db::types::Float::try_new(result)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::WrongArity(
            "DOUBLE expects exactly 1 numeric argument".to_string(),
        )),
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn core::error::Error>> {
    let path = std::path::PathBuf::from("demo.monumentum");

    println!("=== 1. Membuat workbook baru ===");
    let mut wb = Workbook::<FileStorage>::create_new(&path)?;

    let columns = vec![
        ColumnDef::new("Nama", DataType::Text),
        ColumnDef::new("Nilai", DataType::Integer),
        ColumnDef::new("Kota", DataType::Text),
    ];
    wb.create_sheet("Data", columns)?;
    println!("Sheet 'Data' dibuat dengan kolom: Nama, Nilai, Kota");

    let cols2 = vec![
        ColumnDef::new("ID", DataType::Integer),
        ColumnDef::new("Label", DataType::Text),
    ];
    wb.create_sheet("Lain", cols2)?;
    println!("Sheet 'Lain' dibuat dengan kolom: ID, Label\n");

    println!("=== 2. Registrasi custom function ===");
    wb.register_function("DOUBLE", custom_double);
    println!("Fungsi 'DOUBLE' berhasil didaftarkan.\n");

    println!("=== 3. Mengisi data ke sheet 'Data' ===");
    wb.insert_row(
        "Data",
        vec![
            Value::from("Alice"),
            Value::from(90_i64),
            Value::from("Bandung"),
        ],
    )?;
    wb.insert_row(
        "Data",
        vec![
            Value::from("Bob"),
            Value::from(80_i64),
            Value::from("Jakarta"),
        ],
    )?;
    wb.insert_row(
        "Data",
        vec![
            Value::from("Charlie"),
            Value::from(85_i64),
            Value::from("Surabaya"),
        ],
    )?;
    wb.insert_row(
        "Data",
        vec![
            Value::from("Diana"),
            Value::from(90_i64),
            Value::from("Bandung"),
        ],
    )?;
    println!("4 baris data ditambahkan.\n");

    println!("=== 4. Formula di sheet 'Formula' ===");
    let cols3 = vec![
        ColumnDef::new("Deskripsi", DataType::Text),
        ColumnDef::new("Hasil", DataType::Float),
    ];
    wb.create_sheet("Formula", cols3)?;

    let mut row_idx = 0;
    wb.insert_row(
        "Formula",
        vec![Value::from("AVERAGE Data!B1:B4"), Value::Null],
    )?;
    wb.set_formula("Formula", row_idx, 1, "AVERAGE(Data!B1:B4)")?;

    row_idx = 1;
    wb.insert_row("Formula", vec![Value::from("SUM Data!B1:B4"), Value::Null])?;
    wb.set_formula("Formula", row_idx, 1, "SUM(Data!B1:B4)")?;

    row_idx = 2;
    wb.insert_row("Formula", vec![Value::from("MAX Data!B1:B4"), Value::Null])?;
    wb.set_formula("Formula", row_idx, 1, "MAX(Data!B1:B4)")?;

    row_idx = 3;
    wb.insert_row("Formula", vec![Value::from("DOUBLE(21)"), Value::Null])?;
    wb.set_formula("Formula", row_idx, 1, "DOUBLE(21)")?;

    println!("4 baris formula ditambahkan.\n");

    println!("=== 5. Evaluasi formula ===");
    for r in 0..wb.row_count("Formula")? {
        let desc = wb.get_cell_value("Formula", r, 0)?;
        let val = wb.get_cell_value("Formula", r, 1)?;
        println!("  {desc:?} => {val:?}");
    }
    println!();

    println!("=== 6. Sorting dengan Query Builder ===");
    let sorted_asc = wb.query("Data").order_by(1, true).fetch_all()?;
    println!("Data setelah sort ascending (Nilai):");
    for row in &sorted_asc {
        let nama = row.get(0).unwrap_or(&Value::Null);
        let nilai = row.get(1).unwrap_or(&Value::Null);
        println!("  {nama:?}: {nilai:?}");
    }

    let sorted_desc = wb.query("Data").order_by(1, false).fetch_all()?;
    println!("Data setelah sort descending (Nilai):");
    for row in &sorted_desc {
        let nama = row.get(0).unwrap_or(&Value::Null);
        let nilai = row.get(1).unwrap_or(&Value::Null);
        println!("  {nama:?}: {nilai:?}");
    }
    println!();

    println!("=== 7. Filtering dengan Query Builder ===");
    let nilai_target = Value::from(90_i64);
    let filtered = wb
        .query("Data")
        .filter(|row| row.get(1).is_some_and(|v| v == &nilai_target))
        .fetch_all()?;
    println!("Baris dengan Nilai = 90:");
    for row in &filtered {
        println!("  {:?}", row.values());
    }
    println!();

    println!("=== 8. Distinct values (tetap memakai API khusus) ===");
    let distinct = wb.distinct_values("Data", 1)?;
    println!("Nilai unik pada kolom Nilai: {distinct:?}\n");

    println!("=== 9. Replace nilai ===");
    let count = wb.replace_in_sheet("Data", &Value::from(80_i64), &Value::from(82_i64))?;
    println!("Diganti {count} sel. Data setelah replace:");
    for r in 0..wb.row_count("Data")? {
        let nama = wb.get_cell_value("Data", r, 0)?;
        let nilai = wb.get_cell_value("Data", r, 1)?;
        println!("  {nama:?}: {nilai:?}");
    }
    println!();

    println!("=== 10. Manipulasi baris & kolom ===");
    wb.insert_row_at(
        "Data",
        0,
        vec![
            Value::from("Eve"),
            Value::from(77_i64),
            Value::from("Medan"),
        ],
    )?;
    println!("Setelah insert_row_at(0):");
    for r in 0..wb.row_count("Data")? {
        let nama = wb.get_cell_value("Data", r, 0)?;
        let nilai = wb.get_cell_value("Data", r, 1)?;
        println!("  {nama:?}: {nilai:?}");
    }

    wb.delete_row("Data", 1)?;
    println!("Setelah delete_row(1):");
    for r in 0..wb.row_count("Data")? {
        let nama = wb.get_cell_value("Data", r, 0)?;
        let nilai = wb.get_cell_value("Data", r, 1)?;
        println!("  {nama:?}: {nilai:?}");
    }

    let mut col_def = ColumnDef::new("Umur", DataType::Integer);
    col_def.set_default(Some(Value::from(20_i64)));
    wb.insert_column("Data", 2, &col_def)?;
    println!("Setelah insert_column Umur di posisi 2:");
    for r in 0..wb.row_count("Data")? {
        println!("  {:?}", wb.get_cell_value("Data", r, 0)?);
    }
    wb.delete_column("Data", 2)?;
    println!("Setelah delete_column(2):");
    for r in 0..wb.row_count("Data")? {
        println!("  {:?}", wb.get_cell_value("Data", r, 0)?);
    }
    println!();

    println!("=== 11. Rename, drop, create sheet ===");
    wb.rename_sheet("Lain", "LainBaru")?;
    println!(
        "Sheet 'Lain' di-rename menjadi 'LainBaru'. Sheet names: {:?}",
        wb.sheet_names()
    );
    wb.drop_sheet("LainBaru")?;
    println!(
        "Sheet 'LainBaru' di-drop. Sheet names: {:?}",
        wb.sheet_names()
    );
    wb.create_sheet("Temporary", vec![ColumnDef::new("X", DataType::Text)])?;
    println!(
        "Sheet 'Temporary' dibuat. Sheet names: {:?}\n",
        wb.sheet_names()
    );

    println!("=== 12. Clear sheet ===");
    wb.clear_sheet("Temporary")?;
    println!(
        "Sheet 'Temporary' dibersihkan. Baris: {}",
        wb.row_count("Temporary")?
    );
    println!();

    println!("=== 13. Validasi cell ===");
    match wb.set_cell("Data", 0, 1, Value::from("bukan angka")) {
        Ok(()) => println!("Tidak terduga: bisa set nilai text ke kolom Integer"),
        Err(e) => println!("Gagal set_cell seperti diharapkan: {e}"),
    }
    wb.set_cell("Data", 0, 1, Value::from(99_i64))?;
    println!();

    println!("=== 14. Proteksi sheet ===");
    wb.protect_sheet("Data")?;
    match wb.set_cell("Data", 0, 0, Value::from("Zed")) {
        Ok(()) => println!("Tidak terduga: bisa menulis sheet terproteksi"),
        Err(e) => println!("Gagal seperti diharapkan: {e}"),
    }
    wb.unprotect_sheet("Data")?;
    wb.set_cell("Data", 0, 0, Value::from("Zed"))?;
    println!(
        "Setelah unprotect, set_cell berhasil: {:?}\n",
        wb.get_cell_value("Data", 0, 0)?
    );

    println!("=== 15. Transaksi ===");
    {
        let mut tx = Transaction::begin(&mut wb);
        tx.workbook_mut().insert_row(
            "Data",
            vec![
                Value::from("Temp"),
                Value::from(50_i64),
                Value::from("Bandung"),
            ],
        )?;
        println!(
            "Sebelum rollback, row count Data = {}",
            tx.workbook_mut().row_count("Data")?
        );
        tx.rollback();
    }
    println!(
        "Setelah rollback, row count Data = {}\n",
        wb.row_count("Data")?
    );

    {
        let mut tx = Transaction::begin(&mut wb);
        tx.workbook_mut().insert_row(
            "Data",
            vec![
                Value::from("Commited"),
                Value::from(60_i64),
                Value::from("Bandung"),
            ],
        )?;
        tx.commit()?;
    }
    println!(
        "Setelah commit, row count Data = {}\n",
        wb.row_count("Data")?
    );

    println!("=== 16. Error handling ===");
    match wb.get_cell_value("Data", 100, 0) {
        Ok(v) => println!("Tidak terduga: {v:?}"),
        Err(e) => println!("Gagal seperti diharapkan: {e}"),
    }
    println!();

    println!("=== 17. Menyimpan ulang dan menutup ===");
    wb.save()?;
    println!("Workbook tersimpan ulang.");
    wb.close()?;
    println!("Workbook ditutup.\n");

    println!("Contoh selesai! Semua API utama berhasil dijalankan.");
    Ok(())
}
