#![allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::if_same_then_else,
    clippy::map_unwrap_or,
    clippy::std_instead_of_core
)]
use serde_json as _;
use std::fmt::Write as _;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::FileStorage;
use monumentum_workbook::Workbook;

use monumentum_functions as _;
use monumentum_query as _;
use pretty_assertions as _;
use proptest as _;
use tempfile as _;

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("demo.monumentum"));

    let mut wb = if path.exists() {
        Workbook::<FileStorage>::open(&path)?
    } else {
        Workbook::<FileStorage>::create_new(&path)?
    };

    println!("Monumentum CLI v0.1");
    println!("File: {}", path.display());
    println!("Ketik 'help' untuk daftar perintah.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("monumentum> ");
        stdout.flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match execute(&mut wb, line) {
            Ok(Some(output)) => println!("{output}"),
            Ok(None) => {}
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    wb.save()?;
    wb.close()?;
    println!("Disimpan dan ditutup.");

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn execute(
    wb: &mut Workbook<FileStorage>,
    line: &str,
) -> Result<Option<String>, Box<dyn core::error::Error>> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let cmd = parts.first().copied().unwrap_or("");
    let args: Vec<&str> = parts[1..].to_vec();

    match cmd {
        "help" => Ok(Some(help_text())),
        "sheets" | "list" => {
            let names = wb.sheet_names();
            Ok(Some(if names.is_empty() {
                "(belum ada sheet)".to_string()
            } else {
                names.join(", ")
            }))
        }
        "create_sheet" => {
            if args.is_empty() {
                return Err("usage: create_sheet <name> <col:type> [<col:type> ...]".into());
            }
            let name = args[0];
            let mut columns = Vec::new();
            for spec in &args[1..] {
                let (col_name, type_name) = spec.split_once(':').ok_or("format kolom salah")?;
                let data_type = parse_data_type(type_name)?;
                columns.push(ColumnDef::new(col_name, data_type));
            }
            wb.create_sheet(name, columns)?;
            Ok(Some(format!("Sheet '{name}' dibuat")))
        }
        "drop_sheet" => {
            if args.is_empty() {
                return Err("usage: drop_sheet <name>".into());
            }
            wb.drop_sheet(args[0])?;
            Ok(Some("Sheet dihapus".to_string()))
        }
        "rename_sheet" => {
            if args.len() < 2 {
                return Err("usage: rename_sheet <old> <new>".into());
            }
            wb.rename_sheet(args[0], args[1])?;
            Ok(Some("Sheet di-rename".to_string()))
        }
        "row_count" | "rows" => {
            if args.is_empty() {
                return Err("usage: rows <sheet>".into());
            }
            let count = wb.row_count(args[0])?;
            Ok(Some(count.to_string()))
        }
        "col_count" | "cols" => {
            if args.is_empty() {
                return Err("usage: cols <sheet>".into());
            }
            let count = wb.column_count(args[0])?;
            Ok(Some(count.to_string()))
        }
        "insert" | "insert_row" => {
            if args.len() < 2 {
                return Err("usage: insert <sheet> <val1> [val2 ...]".into());
            }
            let values = args[1..]
                .iter()
                .map(|s| parse_value(s))
                .collect::<Result<Vec<Value>, _>>()?;
            wb.insert_row(args[0], values)?;
            Ok(Some("Baris ditambahkan".to_string()))
        }
        "delete_row" => {
            if args.len() < 2 {
                return Err("usage: delete_row <sheet> <index>".into());
            }
            let idx: usize = args[1].parse()?;
            wb.delete_row(args[0], idx)?;
            Ok(Some("Baris dihapus".to_string()))
        }
        "set_cell" | "set" => {
            if args.len() < 4 {
                return Err("usage: set_cell <sheet> <row> <col> <value>".into());
            }
            let row: usize = args[1].parse()?;
            let col: usize = args[2].parse()?;
            let value = parse_value(args[3])?;
            wb.set_cell(args[0], row, col, value)?;
            Ok(Some("Sel diupdate".to_string()))
        }
        "get_cell" | "get" => {
            if args.len() < 3 {
                return Err("usage: get_cell <sheet> <row> <col>".into());
            }
            let row: usize = args[1].parse()?;
            let col: usize = args[2].parse()?;
            let value = wb.get_cell_value(args[0], row, col)?;
            Ok(Some(format!("{value:?}")))
        }
        "set_formula" | "formula" => {
            if args.len() < 4 {
                return Err("usage: set_formula <sheet> <row> <col> <formula>".into());
            }
            let row: usize = args[1].parse()?;
            let col: usize = args[2].parse()?;
            let formula = args[3].trim_start_matches('=');
            wb.set_formula(args[0], row, col, formula)?;
            Ok(Some("Formula dipasang".to_string()))
        }
        "sort" => {
            if args.len() < 3 {
                return Err("usage: sort <sheet> <col> asc|desc".into());
            }
            let col: usize = args[1].parse()?;
            let ascending = args[2].eq_ignore_ascii_case("asc");
            wb.sort_sheet(args[0], col, ascending)?;
            Ok(Some("Sheet diurutkan".to_string()))
        }
        "filter" => {
            if args.len() < 3 {
                return Err("usage: filter <sheet> <col> <value>".into());
            }
            let col: usize = args[1].parse()?;
            let value = parse_value(args[2])?;
            let rows = wb.filter_sheet(args[0], col, &value)?;
            let mut out = String::new();
            for row in rows {
                let _ = writeln!(out, "{:?}", row.values());
            }
            Ok(Some(out))
        }
        "distinct" => {
            if args.len() < 2 {
                return Err("usage: distinct <sheet> <col>".into());
            }
            let col: usize = args[1].parse()?;
            let values = wb.distinct_values(args[0], col)?;
            Ok(Some(format!("{values:?}")))
        }
        "replace" => {
            if args.len() < 3 {
                return Err("usage: replace <sheet> <old> <new>".into());
            }
            let old = parse_value(args[1])?;
            let new = parse_value(args[2])?;
            let count = wb.replace_in_sheet(args[0], &old, &new)?;
            Ok(Some(format!("{count} sel diganti")))
        }
        "protect" => {
            if args.is_empty() {
                return Err("usage: protect <sheet>".into());
            }
            wb.protect_sheet(args[0])?;
            Ok(Some("Sheet diproteksi".to_string()))
        }
        "unprotect" => {
            if args.is_empty() {
                return Err("usage: unprotect <sheet>".into());
            }
            wb.unprotect_sheet(args[0])?;
            Ok(Some("Proteksi dilepas".to_string()))
        }
        "save" => {
            wb.save()?;
            Ok(Some("Workbook disimpan".to_string()))
        }
        "reload" => {
            wb.reload()?;
            Ok(Some("Workbook dimuat ulang dari disk".to_string()))
        }
        "exit" | "quit" => Ok(None),
        _ => Err(format!("Perintah tidak dikenal: {cmd}. Ketik 'help' untuk bantuan.").into()),
    }
}

fn help_text() -> String {
    let mut h = String::new();
    let _ = writeln!(h, "Perintah tersedia:");
    let _ = writeln!(h, "  help");
    let _ = writeln!(h, "  sheets|list");
    let _ = writeln!(h, "  create_sheet <name> <col:Type> [<col:Type> ...]");
    let _ = writeln!(h, "  drop_sheet <name>");
    let _ = writeln!(h, "  rename_sheet <old> <new>");
    let _ = writeln!(h, "  rows <sheet>");
    let _ = writeln!(h, "  cols <sheet>");
    let _ = writeln!(h, "  insert <sheet> <v1> [v2 ...]");
    let _ = writeln!(h, "  delete_row <sheet> <idx>");
    let _ = writeln!(h, "  set_cell <sheet> <row> <col> <value>");
    let _ = writeln!(h, "  get_cell <sheet> <row> <col>");
    let _ = writeln!(h, "  set_formula <sheet> <row> <col> <formula>");
    let _ = writeln!(h, "  sort <sheet> <col> asc|desc");
    let _ = writeln!(h, "  filter <sheet> <col> <value>");
    let _ = writeln!(h, "  distinct <sheet> <col>");
    let _ = writeln!(h, "  replace <sheet> <old> <new>");
    let _ = writeln!(h, "  protect <sheet>");
    let _ = writeln!(h, "  unprotect <sheet>");
    let _ = writeln!(h, "  save");
    let _ = writeln!(h, "  reload");
    let _ = writeln!(h, "  exit|quit");
    h
}

fn parse_data_type(s: &str) -> Result<DataType, Box<dyn core::error::Error>> {
    let dt = if s.eq_ignore_ascii_case("integer") {
        DataType::Integer
    } else if s.eq_ignore_ascii_case("float") {
        DataType::Float
    } else if s.eq_ignore_ascii_case("text") {
        DataType::Text
    } else if s.eq_ignore_ascii_case("blob") {
        DataType::Blob
    } else if s.eq_ignore_ascii_case("null") {
        DataType::Null
    } else {
        return Err(format!("Tipe data tidak dikenal: {s}").into());
    };
    Ok(dt)
}

fn parse_value(s: &str) -> Result<Value, Box<dyn core::error::Error>> {
    let lower = s.to_ascii_lowercase();
    if lower == "null" {
        Ok(Value::Null)
    } else if lower == "true" {
        Ok(Value::Boolean(true))
    } else if lower == "false" {
        Ok(Value::Boolean(false))
    } else if s.starts_with('=') {
        Ok(Value::Formula(s.trim_start_matches('=').to_string()))
    } else if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        Ok(Value::from(s[1..s.len() - 1].to_string()))
    } else if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        Ok(Value::from(s[1..s.len() - 1].to_string()))
    } else if let Ok(i) = s.parse::<i64>() {
        Ok(Value::from(i))
    } else if let Ok(f) = s.parse::<f64>() {
        if f.is_finite() {
            monumentum_db::types::Float::try_new(f)
                .map(Value::Float)
                .map_err(|e| e.to_string().into())
        } else {
            Err("Float harus finite".into())
        }
    } else {
        Ok(Value::from(s.to_string()))
    }
}
