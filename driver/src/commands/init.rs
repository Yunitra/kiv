//! `kiv init` command implementation.

use crate::project::Project;
use std::env;

/// Initializes a Kiv project in the current directory
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("kiv-project");

    println!("Initializing project in current directory...");

    let project = Project::init(&current_dir, name)?;

    println!(
        "     Created binary package `{}`",
        project.manifest.package.name
    );
    println!();
    println!("Project initialized!");

    Ok(())
}
