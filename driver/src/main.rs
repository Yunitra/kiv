//! # Kiv Driver
//!
//! This is the command-line interface for the Kiv programming language.
//! It provides a user-friendly interface to the kivc compiler.

mod commands;
mod project;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kiv")]
#[command(version, about = "The Kiv programming language compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new Kiv project
    New {
        /// Name of the project
        name: String,
    },

    /// Initialize a Kiv project in the current directory
    Init,

    /// Build the current project
    Build {
        /// Build in release mode with optimizations
        #[arg(long)]
        release: bool,
    },

    /// Check the current project for errors without building
    Check,

    /// Build and run the current project
    Run {
        /// Build in release mode with optimizations
        #[arg(long)]
        release: bool,
    },

    /// Remove build artifacts
    Clean,

    /// Format Kiv source files
    Fmt {
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,
    },

    /// Run linter on the current project
    Clippy,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::New { name } => commands::new(&name),
        Command::Init => commands::init(),
        Command::Build { release } => commands::build(release),
        Command::Check => commands::check(),
        Command::Run { release } => commands::run(release),
        Command::Clean => commands::clean(),
        Command::Fmt { check } => commands::fmt(check),
        Command::Clippy => {
            eprintln!("kiv clippy is not yet implemented");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
