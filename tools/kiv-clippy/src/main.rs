//! # Kiv Linter
//!
//! Static analysis tool for Kiv code that detects common issues and enforces best practices.

use clap::Parser;
use kivc::{CompileResult, CompilerConfig, StopAfter, compile_source};
use kivc_diagnostics::DiagnosticsCollector;
use lint::all_rules;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "kiv-clippy")]
#[command(version, about = "Lint Kiv source files", long_about = None)]
struct Cli {
    /// Files to lint (if not provided, lints all .kiv files in src/)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Deny all warnings (exit with error if any warnings are found)
    #[arg(short = 'D', long)]
    deny_warnings: bool,
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

    let mut has_warnings = false;

    for file in files {
        match lint_file(&file) {
            Ok(diagnostics) => {
                if diagnostics.has_errors() || !diagnostics.errors().is_empty() {
                    has_warnings = true;
                    for error in diagnostics.errors() {
                        eprintln!("{:?}", miette::Report::new(error.clone()));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error linting {}: {}", file.display(), e);
                has_warnings = true;
            }
        }
    }

    if has_warnings {
        if cli.deny_warnings {
            eprintln!("\nLinting failed with warnings");
            process::exit(1);
        } else {
            eprintln!("\nLinting completed with warnings");
        }
    } else {
        println!("All checks passed!");
    }
}

/// Lints a single file
fn lint_file(path: &PathBuf) -> Result<DiagnosticsCollector, Box<dyn std::error::Error>> {
    // Read the file
    let source = fs::read_to_string(path)?;

    // Compile to HIR
    let config = CompilerConfig::new().with_stop_after(StopAfter::Hir);

    let (_session, result) = compile_source(path.to_string_lossy().to_string(), source, config);

    let hir = match result {
        CompileResult::StoppedAtHir(hir) => hir,
        CompileResult::Failed(diagnostics) => {
            return Ok(diagnostics);
        }
        _ => {
            return Err("Unexpected compilation result".into());
        }
    };

    // Run lint rules
    let mut diagnostics = DiagnosticsCollector::new();
    let rules = all_rules();

    for rule in rules {
        rule.check(&hir, &mut diagnostics);
    }

    Ok(diagnostics)
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
