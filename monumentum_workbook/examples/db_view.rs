use std::env;
use std::path::PathBuf;

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
    let default_path = PathBuf::from("demo.monumentum");
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(default_path);

    let wb = Workbook::<FileStorage>::open(&path)?;

    println!("Workbook : {}", path.display());
    let sheets = wb.sheet_names();
    println!("Sheets   : {}", sheets.join(", "));

    for sheet in sheets {
        print_sheet(&wb, &sheet)?;
    }

    Ok(())
}

fn print_sheet(
    wb: &Workbook<FileStorage>,
    sheet: &str,
) -> Result<(), Box<dyn core::error::Error>> {
    let row_count = wb.row_count(sheet)?;
    let col_count = wb.column_count(sheet)?;
    println!("\nSheet: {sheet}  (rows: {row_count}, columns: {col_count})");

    for row_idx in 0..row_count {
        let mut line = String::from("  [");
        for col_idx in 0..col_count {
            let rendered = match wb.get_cell_value(sheet, row_idx, col_idx) {
                Ok(Value::Formula(_)) | Ok(_) => {
                    match wb.get_cell_value(sheet, row_idx, col_idx) {
                        Ok(v) => format!("{v:?}"),
                        Err(e) => format!("ERR: {e}"),
                    }
                }
                Err(e) => format!("ERR: {e}"),
            };

            if col_idx > 0 {
                line.push_str(", ");
            }
            line.push_str(&rendered);
        }
        line.push(']');
        println!("{line}");
    }

    Ok(())
}
