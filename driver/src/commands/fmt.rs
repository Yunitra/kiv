//! `kiv fmt` command implementation.

use std::process::Command;

/// Formats Kiv code
pub fn fmt(check: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Formatting Kiv code...");

    let mut cmd = Command::new("kiv-fmt");

    if check {
        cmd.arg("--check");
    }

    let status = cmd.status();

    match status {
        Ok(status) if status.success() => {
            println!("    Finished formatting");
            Ok(())
        }
        Ok(status) => Err(format!("Formatter exited with status: {}", status).into()),
        Err(e) => Err(format!("Failed to run kiv-fmt: {}. Make sure it's installed.", e).into()),
    }
}
