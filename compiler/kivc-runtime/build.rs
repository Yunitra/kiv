use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Only build the C runtime when the `c-runtime` feature is enabled
    if std::env::var("CARGO_FEATURE_C_RUNTIME").is_err() {
        println!("cargo:rerun-if-changed=build.rs");
        return;
    }
    let runtime_c = "runtime.c";

    // Find target directory (go up from OUT_DIR to find target/debug or target/release)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();

    // Navigate to workspace root and then to target/{profile}
    let mut target_dir = out_dir.clone();
    while target_dir.file_name().is_none_or(|n| n != "target") {
        if !target_dir.pop() {
            // Fallback to OUT_DIR if we can't find target directory
            target_dir = out_dir.clone();
            break;
        }
    }

    if target_dir.file_name().is_some_and(|n| n == "target") {
        target_dir.push(&profile);
    }

    let runtime_o = target_dir.join("kivc_runtime.o");

    eprintln!("Building runtime library: {}", runtime_o.display());

    // Compile runtime.c to runtime.o
    let status = Command::new("clang")
        .arg("-c")
        .arg(runtime_c)
        .arg("-o")
        .arg(&runtime_o)
        .arg("-O2")
        .status()
        .expect("Failed to execute clang. Make sure clang is installed and in PATH.");

    if !status.success() {
        panic!("Failed to compile runtime.c");
    }

    // Copy runtime.o to OUT_DIR for linking
    let out_runtime_o = out_dir.join("kivc_runtime.o");
    fs::copy(&runtime_o, &out_runtime_o).expect("Failed to copy runtime.o to OUT_DIR");

    // Tell cargo about the runtime object file location
    println!("cargo:rustc-env=KIVC_RUNTIME_O={}", runtime_o.display());

    // Rerun if runtime.c changes
    println!("cargo:rerun-if-changed=runtime.c");
}
