//! `kiv check` command implementation.

use crate::project::Project;
use kivc::{compile_source, CompileResult, CompilerConfig, StopAfter};
use std::env;
use std::fs;

/// Checks the Kiv project without building
pub fn check() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let project = Project::find_root(&current_dir)?;

    println!(
        "    Checking {} v{} ({})",
        project.manifest.package.name,
        project.manifest.package.version,
        project.root.display()
    );

    // Read source file
    let main_file = project.main_file();
    if !main_file.exists() {
        return Err(format!("Source file not found: {}", main_file.display()).into());
    }

    let source = fs::read_to_string(&main_file)?;

    // Configure compiler to stop after type checking
    let config = CompilerConfig::new().with_stop_after(StopAfter::TypeCheck);

    // Compile
    let (_session, result) =
        compile_source(main_file.to_string_lossy().to_string(), source, config);

    match result {
        CompileResult::StoppedAtHir(_) => {
            println!("    Finished checking in ...");
            Ok(())
        }
        CompileResult::Failed(diagnostics) => {
            // Print diagnostics
            for error in diagnostics.errors() {
                eprintln!("{:?}", miette::Report::new(error.clone()));
            }
            Err("Check failed".into())
        }
        _ => Err("Unexpected compilation result".into()),
    }
}
