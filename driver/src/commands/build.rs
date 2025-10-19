//! `kiv build` command implementation.

use crate::project::Project;
use kivc::{CompileResult, CompilerConfig, OptLevel, compile_source};
use std::env;
use std::fs;
use std::process::Command;

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

    // Compile to LLVM IR
    let (_session, result) =
        compile_source(main_file.to_string_lossy().to_string(), source, config);

    match result {
        CompileResult::Success(llvm_ir) => {
            // Write LLVM IR to file
            let ll_path = project.output_ll(release);
            fs::write(&ll_path, &llvm_ir)?;

            // Use LLVM tools to generate object file and link
            let obj_path = ll_path.with_extension("o");
            let output_path = project.output_binary(release);

            // Compile LLVM IR to object file using llc
            let llc_status = Command::new("llc")
                .arg(&ll_path)
                .arg("-filetype=obj")
                .arg("-o")
                .arg(&obj_path)
                .status();

            match llc_status {
                Ok(status) if status.success() => {
                    // Link object file with runtime library
                    let runtime_lib = if cfg!(target_os = "windows") {
                        "kivc_runtime.lib"
                    } else {
                        "libkivc_runtime.a"
                    };

                    let link_status = if cfg!(target_os = "windows") {
                        Command::new("link")
                            .arg(&obj_path)
                            .arg(format!("target/debug/{}", runtime_lib))
                            .arg(format!("/OUT:{}", output_path.display()))
                            .status()
                    } else {
                        Command::new("clang")
                            .arg(&obj_path)
                            .arg(format!("target/debug/{}", runtime_lib))
                            .arg("-o")
                            .arg(&output_path)
                            .status()
                    };

                    match link_status {
                        Ok(status) if status.success() => {
                            println!(
                                "    Finished {} [{}] target(s) in ...",
                                if release { "release" } else { "debug" },
                                if release { "optimized" } else { "unoptimized" }
                            );
                            println!();
                            println!("Binary: {}", output_path.display());
                            Ok(())
                        }
                        Ok(status) => Err(format!("Linking failed with status: {}", status).into()),
                        Err(e) => Err(format!(
                            "Failed to run linker: {}. Make sure clang/link is installed.",
                            e
                        )
                        .into()),
                    }
                }
                Ok(status) => Err(format!("llc failed with status: {}", status).into()),
                Err(e) => {
                    Err(format!("Failed to run llc: {}. Make sure LLVM is installed.", e).into())
                }
            }
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
