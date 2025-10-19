//! # Kiv Formatter
//!
//! This crate provides code formatting functionality for the Kiv programming language.
//! It can format Kiv source code according to a consistent style.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "kiv-fmt")]
#[command(version, about = "Format Kiv source files", long_about = None)]
struct Cli {
    /// Files to format (if not provided, formats all .kiv files in src/)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Check formatting without making changes
    #[arg(long)]
    check: bool,

    /// Format files in place (default)
    #[arg(short, long)]
    write: bool,
}

fn main() {
    let cli = Cli::parse();

    let files = if cli.files.is_empty() {
        // Find all .kiv files in src/
        find_kiv_files("src")
    } else {
        cli.files
    };

    if files.is_empty() {
        eprintln!("No Kiv files found");
        process::exit(1);
    }

    let mut all_formatted = true;

    for file in files {
        match format_file(&file, cli.check, cli.write) {
            Ok(formatted) => {
                if cli.check && !formatted {
                    println!("{}: needs formatting", file.display());
                    all_formatted = false;
                } else if !cli.check {
                    println!("{}: formatted", file.display());
                }
            }
            Err(e) => {
                eprintln!("Error formatting {}: {}", file.display(), e);
                all_formatted = false;
            }
        }
    }

    if cli.check && !all_formatted {
        eprintln!("\nSome files need formatting. Run `kiv fmt` to format them.");
        process::exit(1);
    }
}

/// Formats a single file
/// Returns Ok(true) if file was already formatted, Ok(false) if it needed formatting
fn format_file(
    path: &PathBuf,
    check: bool,
    _write: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Read the file
    let source = fs::read_to_string(path)?;

    // Format using the library function
    let formatted = kiv_formatter::format_code(&source)?;

    // Check if formatting changed anything
    let already_formatted = source == formatted;

    if check {
        return Ok(already_formatted);
    }

    // Write the formatted code back
    fs::write(path, formatted)?;

    Ok(true)
}

/// Finds all .kiv files in a directory recursively
fn find_kiv_files(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                files.extend(find_kiv_files(&path.to_string_lossy()));
            } else if path.extension().and_then(|s| s.to_str()) == Some("kiv") {
                files.push(path);
            }
        }
    }

    files
}
