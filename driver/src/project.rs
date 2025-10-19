//! Project management for Kiv projects.
//!
//! This module handles Kiv.toml parsing, project initialization,
//! and build directory management.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Parse errors
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse Kiv.toml: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to serialize Kiv.toml: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Project already exists at {0}")]
    AlreadyExists(PathBuf),

    #[error("Not a Kiv project (Kiv.toml not found)")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, ProjectError>;

/// Kiv project manifest (Kiv.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
}

impl Manifest {
    /// Creates a new manifest with the given package name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            package: Package {
                name: name.into(),
                version: "0.1.0".to_string(),
            },
        }
    }

    /// Loads a manifest from the given path
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Saves the manifest to the given path
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

/// Represents a Kiv project
#[derive(Debug)]
pub struct Project {
    /// Root directory of the project
    pub root: PathBuf,

    /// Project manifest
    pub manifest: Manifest,
}

impl Project {
    /// Creates a new project at the given path
    pub fn create(path: &Path, name: &str) -> Result<Self> {
        // Check if directory already exists
        if path.exists() {
            return Err(ProjectError::AlreadyExists(path.to_path_buf()));
        }

        // Create project directory
        fs::create_dir_all(path)?;

        // Create manifest
        let manifest = Manifest::new(name);
        let manifest_path = path.join("Kiv.toml");
        manifest.save(&manifest_path)?;

        // Create src directory
        let src_dir = path.join("src");
        fs::create_dir(&src_dir)?;

        // Create main.kiv with a simple hello world
        let main_content = r#"fun main() {
    print("Hello, Kiv!")
}
"#;
        fs::write(src_dir.join("main.kiv"), main_content)?;

        // Create .gitignore
        let gitignore_content = "target/\n*.exe\n*.dll\n*.so\n*.dylib\n*.ll\n*.bc\n";
        fs::write(path.join(".gitignore"), gitignore_content)?;

        Ok(Self {
            root: path.to_path_buf(),
            manifest,
        })
    }

    /// Initializes a project in the current directory
    pub fn init(path: &Path, name: &str) -> Result<Self> {
        // Check if Kiv.toml already exists
        let manifest_path = path.join("Kiv.toml");
        if manifest_path.exists() {
            return Err(ProjectError::AlreadyExists(manifest_path));
        }

        // Create manifest
        let manifest = Manifest::new(name);
        manifest.save(&manifest_path)?;

        // Create src directory if it doesn't exist
        let src_dir = path.join("src");
        if !src_dir.exists() {
            fs::create_dir(&src_dir)?;

            // Create main.kiv if it doesn't exist
            let main_file = src_dir.join("main.kiv");
            if !main_file.exists() {
                let main_content = r#"fun main() {
    print("Hello, Kiv!")
}
"#;
                fs::write(main_file, main_content)?;
            }
        }

        // Create .gitignore if it doesn't exist
        let gitignore_path = path.join(".gitignore");
        if !gitignore_path.exists() {
            let gitignore_content = "target/\n*.exe\n*.dll\n*.so\n*.dylib\n*.ll\n*.bc\n";
            fs::write(gitignore_path, gitignore_content)?;
        }

        Ok(Self {
            root: path.to_path_buf(),
            manifest,
        })
    }

    /// Loads a project from the given directory
    pub fn load(path: &Path) -> Result<Self> {
        let manifest_path = path.join("Kiv.toml");
        if !manifest_path.exists() {
            return Err(ProjectError::NotFound);
        }

        let manifest = Manifest::load(&manifest_path)?;

        Ok(Self {
            root: path.to_path_buf(),
            manifest,
        })
    }

    /// Finds the project root by walking up the directory tree
    pub fn find_root(start_path: &Path) -> Result<Self> {
        let mut current = start_path;

        loop {
            let manifest_path = current.join("Kiv.toml");
            if manifest_path.exists() {
                return Self::load(current);
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => return Err(ProjectError::NotFound),
            }
        }
    }

    /// Gets the source directory
    pub fn src_dir(&self) -> PathBuf {
        self.root.join("src")
    }

    /// Gets the main source file
    pub fn main_file(&self) -> PathBuf {
        self.src_dir().join("main.kiv")
    }

    /// Gets the target directory
    pub fn target_dir(&self) -> PathBuf {
        self.root.join("target")
    }

    /// Gets the debug build directory
    pub fn debug_dir(&self) -> PathBuf {
        self.target_dir().join("debug")
    }

    /// Gets the release build directory
    pub fn release_dir(&self) -> PathBuf {
        self.target_dir().join("release")
    }

    /// Ensures build directories exist
    pub fn ensure_build_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.debug_dir())?;
        fs::create_dir_all(self.release_dir())?;
        Ok(())
    }

    /// Cleans the build artifacts
    pub fn clean(&self) -> Result<()> {
        let target_dir = self.target_dir();
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        Ok(())
    }

    /// Gets the output binary path for the given build mode
    pub fn output_binary(&self, release: bool) -> PathBuf {
        let dir = if release {
            self.release_dir()
        } else {
            self.debug_dir()
        };

        let binary_name = if cfg!(target_os = "windows") {
            format!("{}.exe", self.manifest.package.name)
        } else {
            self.manifest.package.name.clone()
        };

        dir.join(binary_name)
    }

    /// Gets the LLVM IR output path
    pub fn output_ll(&self, release: bool) -> PathBuf {
        let dir = if release {
            self.release_dir()
        } else {
            self.debug_dir()
        };

        dir.join(format!("{}.ll", self.manifest.package.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest::new("test-project");
        let toml = toml::to_string(&manifest).unwrap();
        assert!(toml.contains("test-project"));
        assert!(toml.contains("0.1.0"));
    }

    #[test]
    fn test_project_creation() {
        let temp_dir = env::temp_dir().join("kiv-test-project");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }

        let project = Project::create(&temp_dir, "test-project").unwrap();
        assert_eq!(project.manifest.package.name, "test-project");
        assert!(project.main_file().exists());
        assert!(temp_dir.join("Kiv.toml").exists());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_project_init() {
        let temp_dir = env::temp_dir().join("kiv-test-init");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }
        fs::create_dir(&temp_dir).unwrap();

        let project = Project::init(&temp_dir, "test-init").unwrap();
        assert_eq!(project.manifest.package.name, "test-init");
        assert!(project.main_file().exists());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}
