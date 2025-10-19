//! `kiv new` command implementation.

use crate::project::Project;
use std::path::Path;

/// Creates a new Kiv project
pub fn new(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(name);

    println!("Creating new project '{}'...", name);

    let project = Project::create(path, name)?;

    println!(
        "     Created binary package `{}`",
        project.manifest.package.name
    );
    println!();
    println!("Project created at: {}", project.root.display());

    Ok(())
}
