//! # Kiv Compiler CLI
//!
//! This is the command-line interface for the Kiv compiler (kivc).

use clap::{Parser, Subcommand};
use kivc::{CompileResult, CompilerConfig, OptLevel, compile_source};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kivc")]
#[command(about = "The Kiv Programming Language Compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Kiv source file
    Build {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0-2)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,

        /// Emit debug information
        #[arg(long, default_value = "true")]
        debug: bool,
    },

    /// Check a Kiv source file without generating code
    Check {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Display compiler version
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input,
            output,
            opt_level,
            debug,
        } => {
            let opt = match opt_level {
                0 => OptLevel::None,
                1 => OptLevel::Basic,
                _ => OptLevel::Aggressive,
            };

            let config = CompilerConfig::new()
                .with_opt_level(opt)
                .with_debug_info(debug);

            match std::fs::read_to_string(&input) {
                Ok(source) => {
                    let filename = input.display().to_string();
                    let (_session, result) = compile_source(filename, source, config);

                    match result {
                        CompileResult::Success(llvm_ir) => {
                            // Write LLVM IR to same directory as source
                            let output_path = output.unwrap_or_else(|| {
                                let mut path = input.clone();
                                path.set_extension("ll");
                                path
                            });

                            match std::fs::write(&output_path, llvm_ir) {
                                Ok(_) => {
                                    println!("✓ LLVM IR generated: {}", output_path.display());

                                    // Compile to object file using llc
                                    let obj_path = output_path.with_extension("o");
                                    match std::process::Command::new("llc")
                                        .arg("-filetype=obj")
                                        .arg(&output_path)
                                        .arg("-o")
                                        .arg(&obj_path)
                                        .output()
                                    {
                                        Ok(output) if output.status.success() => {
                                            println!(
                                                "✓ Object file generated: {}",
                                                obj_path.display()
                                            );

                                            // Link to executable with runtime library
                                            let exe_path =
                                                output_path.with_extension(if cfg!(windows) {
                                                    "exe"
                                                } else {
                                                    ""
                                                });

                                            // Use runtime library from target directory
                                            let runtime_o = std::path::PathBuf::from("target")
                                                .join(if cfg!(debug_assertions) {
                                                    "debug"
                                                } else {
                                                    "release"
                                                })
                                                .join("kivc_runtime.o");

                                            if !runtime_o.exists() {
                                                eprintln!(
                                                    "⚠ Runtime library not found. Building kivc-runtime..."
                                                );
                                                let _ = std::process::Command::new("cargo")
                                                    .args(["build", "--package", "kivc-runtime"])
                                                    .output();
                                            }

                                            match std::process::Command::new("clang")
                                                .arg(&obj_path)
                                                .arg(&runtime_o)
                                                .arg("-o")
                                                .arg(&exe_path)
                                                .output()
                                            {
                                                Ok(link_output) if link_output.status.success() => {
                                                    println!(
                                                        "✓ Executable generated: {}",
                                                        exe_path.display()
                                                    );
                                                    println!("\n✅ Compilation successful!");
                                                    std::process::exit(0);
                                                }
                                                Ok(link_output) => {
                                                    eprintln!("⚠ Linking failed:");
                                                    eprintln!(
                                                        "{}",
                                                        String::from_utf8_lossy(
                                                            &link_output.stderr
                                                        )
                                                    );
                                                    println!(
                                                        "\n✓ LLVM IR and object file available"
                                                    );
                                                    std::process::exit(0);
                                                }
                                                Err(_) => {
                                                    eprintln!("⚠ clang not found in PATH");
                                                    println!(
                                                        "  Run: clang {} -o {}",
                                                        obj_path.display(),
                                                        exe_path.display()
                                                    );
                                                    std::process::exit(0);
                                                }
                                            }
                                        }
                                        Ok(llc_output) => {
                                            eprintln!("⚠ llc compilation failed:");
                                            eprintln!(
                                                "{}",
                                                String::from_utf8_lossy(&llc_output.stderr)
                                            );
                                            println!(
                                                "\n✓ LLVM IR available at: {}",
                                                output_path.display()
                                            );
                                            std::process::exit(0);
                                        }
                                        Err(_) => {
                                            eprintln!("⚠ llc not found in PATH");
                                            println!("  Add LLVM\\bin to PATH or run:");
                                            println!(
                                                "    llc -filetype=obj {} -o {}",
                                                output_path.display(),
                                                obj_path.display()
                                            );
                                            println!(
                                                "    clang {} -o {}",
                                                obj_path.display(),
                                                output_path
                                                    .with_extension(if cfg!(windows) {
                                                        "exe"
                                                    } else {
                                                        ""
                                                    })
                                                    .display()
                                            );
                                            std::process::exit(0);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to write output file: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        CompileResult::Failed(diag) => {
                            eprintln!("✗ Compilation failed with {} error(s)", diag.errors().len());
                            for error in diag.errors() {
                                eprintln!("{:?}", error);
                            }
                            std::process::exit(1);
                        }
                        _ => {
                            println!("✓ Compilation stopped at intermediate stage");
                            std::process::exit(0);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Check { input } => {
            let config = CompilerConfig::new();

            match std::fs::read_to_string(&input) {
                Ok(source) => {
                    let filename = input.display().to_string();
                    let (_session, result) = compile_source(filename, source, config);

                    match result {
                        CompileResult::Success(_) | CompileResult::StoppedAtMir(_) => {
                            println!("✓ No errors found");
                            std::process::exit(0);
                        }
                        CompileResult::Failed(diag) => {
                            eprintln!("✗ Found {} error(s)", diag.errors().len());
                            for error in diag.errors() {
                                eprintln!("{:?}", error);
                            }
                            std::process::exit(1);
                        }
                        _ => {
                            println!("✓ Check passed");
                            std::process::exit(0);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Version => {
            println!("kivc {} (Kiv Compiler)", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
    }
}
