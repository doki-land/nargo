//! Bundler module for Nargo.

use crate::errors::Result;
use std::path::Path;

/// Nargo bundler for building projects.
pub struct NargoBundler;

impl NargoBundler {
    /// Create a new bundler instance.
    pub fn new() -> Self {
        Self
    }

    /// Build the project.
    pub fn build(&self, root: &Path, out_dir: &Path) -> Result<BuildResult> {
        Ok(BuildResult {
            total_size: 0,
            files: vec![],
        })
    }
}

impl Default for NargoBundler {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a build operation.
pub struct BuildResult {
    /// Total size of built artifacts.
    pub total_size: u64,
    /// List of built files.
    pub files: Vec<BuiltFile>,
}

/// Information about a built file.
pub struct BuiltFile {
    /// File path.
    pub path: String,
    /// File size.
    pub size: u64,
}
