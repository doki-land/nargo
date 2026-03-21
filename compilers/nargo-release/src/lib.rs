//! Nargo release management.

#![warn(missing_docs)]

use nargo_types::Result;
use std::path::{Path, PathBuf};

/// Release manager.
pub struct ReleaseManager {
    _root: PathBuf,
}

impl ReleaseManager {
    /// Create a new release manager.
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self { _root: root.to_path_buf() })
    }
}

/// Publish a package to the registry.
pub async fn publish(_root: &Path, _registry: Option<&str>) -> Result<()> {
    println!("Publishing package...");
    Ok(())
}

/// Print the dependency tree.
pub fn print_tree(_root: &Path) -> Result<()> {
    println!("Dependency tree:");
    Ok(())
}
