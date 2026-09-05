use core::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

use fs2 as _;
use proptest as _;

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::traits::StorageEngine;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path_str = args.get(1).ok_or("Usage: db_view <database_path>")?;
    let path = PathBuf::from(path_str);

    if !path.exists() {
        eprintln!("Database file not found: {}", path.display());
        return Err("database file not found".into());
    }

    let mut storage = FileStorage::open(&path, 10)?;
    run_repl(&mut storage)?;

    drop(storage);
    Ok(())
}

fn prompt(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{prompt} ");
    io::stdout().flush()?;
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn list_tables(storage: &FileStorage) {
    println!("\nAvailable tables:");
    let mut count = 0usize;
    for (name, table) in storage.get_catalog().tables() {
        println!("  - {name} ({} columns)", table.schema().columns().len());
        count = count.saturating_add(1);
    }
    if count == 0 {
        println!("  (no tables found)");
    }
}

fn view_schema(storage: &FileStorage, table_name: &str) -> Result<(), Box<dyn Error>> {
    let table = storage
        .get_table(table_name)
        .ok_or_else(|| format!("Table '{table_name}' not found"))?;
    println!("\nSchema for table '{table_name}':");
    for (i, col) in table.schema().columns().iter().enumerate() {
        println!(
            "  {}: {} ({}) pk={} nullable={}",
            i,
            col.name(),
            col.data_type().as_str(),
            col.is_primary_key(),
            col.is_nullable()
        );
    }
    Ok(())
}

fn print_row(row: &monumentum_handler::core::row::Row) {
    let values = row
        .values()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    println!("    [ {values} ]");
}

fn view_rows(
    storage: &mut FileStorage,
    table_name: &str,
    start: usize,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    if limit == 0 {
        println!("  (limit is zero)");
        return Ok(());
    }
    let mut found_any = false;
    for idx in start..start.saturating_add(limit) {
        match storage.get_row(table_name, idx)? {
            Some(row) => {
                print_row(&row);
                found_any = true;
            }
            None => break,
        }
    }
    if !found_any {
        println!("  (no rows in the requested range)");
    }
    Ok(())
}

fn get_row_by_index(
    storage: &mut FileStorage,
    table_name: &str,
    idx: usize,
) -> Result<(), Box<dyn Error>> {
    match storage.get_row(table_name, idx)? {
        Some(row) => print_row(&row),
        None => println!("  (row {idx} does not exist)"),
    }
    Ok(())
}

fn handle_command(storage: &mut FileStorage, cmd: &str) -> Result<bool, Box<dyn Error>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts.first().ok_or("Empty command")?;
    if command.is_empty() {
        return Ok(true);
    }

    match *command {
        "tables" | "list" => {
            list_tables(storage);
        }
        "schema" => {
            let table_name = parts.get(1).ok_or("Usage: schema <table>")?;
            view_schema(storage, table_name)?;
        }
        "rows" => {
            let table_name = parts.get(1).ok_or("Usage: rows <table> [start] [limit]")?;
            let start = parts
                .get(2)
                .map(|s| s.parse::<usize>())
                .transpose()?
                .unwrap_or(0);
            let limit = parts
                .get(3)
                .map(|s| s.parse::<usize>())
                .transpose()?
                .unwrap_or(10);
            view_rows(storage, table_name, start, limit)?;
        }
        "get" => {
            let table_name = parts.get(1).ok_or("Usage: get <table> <index>")?;
            let idx = parts.get(2).ok_or("Missing row index")?.parse::<usize>()?;
            get_row_by_index(storage, table_name, idx)?;
        }
        "help" | "?" => {
            print_help();
        }
        "quit" | "exit" => {
            println!("Goodbye.");
            return Ok(false);
        }
        _ => {
            println!("Unknown command. Type 'help' for available commands.");
        }
    }
    Ok(true)
}

fn print_help() {
    println!("\nAvailable commands (read-only):");
    println!("  tables | list                 List all tables");
    println!("  schema <table>               Show table schema");
    println!("  rows <table> [start] [limit] Show rows (default: start=0, limit=10)");
    println!("  get <table> <index>          Show a single row by index");
    println!("  help | ?                     Show this help");
    println!("  quit | exit                  Exit browser");
}

fn run_repl(storage: &mut FileStorage) -> Result<(), Box<dyn Error>> {
    println!("Monumentum DB Browser (read-only)");
    println!("Type 'help' for available commands.\n");

    loop {
        let input = prompt("db> ")?;
        let should_continue = handle_command(storage, &input)?;
        if !should_continue {
            break;
        }
    }
    Ok(())
}
