//! `kiv build` command implementation.

use crate::project::Project;
use kivc::{compile_source, CompileResult, CompilerConfig, OptLevel};
use std::env;
use std::fs;
// Linking is disabled in textual IR mode; keep imports minimal

/// Builds the Kiv project
pub fn build(release: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let project = Project::find_root(&current_dir)?;

    println!(
        "   Compiling {} v{} ({})",
        project.manifest.package.name,
        project.manifest.package.version,
        project.root.display()
    );

    // Ensure build directories exist
    project.ensure_build_dirs()?;

    // Read source file
    let main_file = project.main_file();
    if !main_file.exists() {
        return Err(format!("Source file not found: {}", main_file.display()).into());
    }

    let source = fs::read_to_string(&main_file)?;

    // Configure compiler
    let config = CompilerConfig::new()
        .with_opt_level(if release {
            OptLevel::Aggressive
        } else {
            OptLevel::None
        })
        .with_debug_info(!release);

    // Compile to textual IR (or LLVM IR if feature enabled)
    let (_session, result) =
        compile_source(main_file.to_string_lossy().to_string(), source, config);

    match result {
        CompileResult::Success(ir) => {
            // For now, write IR to file and skip linking when LLVM tools are unavailable
            let ll_path = project.output_ll(release);
            fs::write(&ll_path, &ir)?;
            println!("    Wrote IR to {}", ll_path.display());
            Ok(())
        }
        CompileResult::Failed(diagnostics) => {
            // Print diagnostics
            for error in diagnostics.errors() {
                eprintln!("{:?}", miette::Report::new(error.clone()));
            }
            Err("Compilation failed".into())
        }
        _ => Err("Unexpected compilation result".into()),
    }
}
