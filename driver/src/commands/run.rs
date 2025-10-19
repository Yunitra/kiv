//! `kiv run` command implementation.

use crate::commands::build;
use crate::project::Project;
use std::env;
use std::process::Command;

/// Runs the Kiv project
pub fn run(release: bool) -> Result<(), Box<dyn std::error::Error>> {
    // First, build the project
    build::build(release)?;

    let current_dir = env::current_dir()?;
    let project = Project::find_root(&current_dir)?;

    let binary = project.output_binary(release);

    println!();
    println!("     Running `{}`", binary.display());
    println!();

    // Run the binary
    let status = Command::new(&binary).status()?;

    if !status.success() {
        Err(format!("Process exited with status: {}", status).into())
    } else {
        Ok(())
    }
}
