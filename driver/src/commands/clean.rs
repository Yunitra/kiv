//! `kiv clean` command implementation.

use crate::project::Project;
use std::env;

/// Cleans build artifacts
pub fn clean() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let project = Project::find_root(&current_dir)?;

    println!("   Removing {}", project.target_dir().display());

    project.clean()?;

    println!("    Finished cleaning");

    Ok(())
}
