//! Lock file data structures and operations.

use nargo_types::{Error, Result};
use oak_json;
use oak_toml;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

/// Lock file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMetadata {
    /// Lock file format version.
    pub version: u32,
    /// Nargo version that generated this lock file.
    pub nargo_version: String,
    /// Timestamp when the lock file was generated.
    pub generated_at: u64,
}

impl Default for LockMetadata {
    fn default() -> Self {
        Self { version: 1, nargo_version: env!("CARGO_PKG_VERSION").to_string(), generated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }
    }
}

/// Represents a locked dependency entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// Package name.
    pub name: String,
    /// Resolved version.
    pub version: String,
    /// Source URL or path.
    pub source: String,
    /// Integrity hash (SRI format).
    pub integrity: String,
    /// Whether this is a dev dependency.
    #[serde(default)]
    pub is_dev: bool,
    /// Whether this is an optional dependency.
    #[serde(default)]
    pub is_optional: bool,
    /// Enabled features.
    #[serde(default)]
    pub features: Vec<String>,
    /// Dependencies of this package.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl LockEntry {
    /// Creates a new lock entry.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), source: String::new(), integrity: String::new(), is_dev: false, is_optional: false, features: Vec::new(), dependencies: Vec::new() }
    }

    /// Sets the source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Sets the integrity hash.
    pub fn with_integrity(mut self, integrity: impl Into<String>) -> Self {
        self.integrity = integrity.into();
        self
    }

    /// Marks as dev dependency.
    pub fn as_dev(mut self) -> Self {
        self.is_dev = true;
        self
    }

    /// Marks as optional dependency.
    pub fn as_optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// Sets the features.
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    /// Adds a dependency.
    pub fn add_dependency(&mut self, dep: impl Into<String>) {
        self.dependencies.push(dep.into());
    }

    /// Returns the unique key for this entry.
    pub fn key(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// The complete lock file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file metadata.
    pub metadata: LockMetadata,
    /// All locked packages.
    pub packages: HashMap<String, LockEntry>,
}

impl Default for LockFile {
    fn default() -> Self {
        Self { metadata: LockMetadata::default(), packages: HashMap::new() }
    }
}

impl LockFile {
    /// Creates a new empty lock file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a lock file from the given path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(Error::external_error("lock".to_string(), format!("Lock file not found: {:?}", path_ref), nargo_types::Span::unknown()));
        }

        let content = fs::read_to_string(path_ref)?;
        let lock: LockFile = oak_toml::from_str(&content).map_err(|e| Error::external_error("lock".to_string(), format!("Failed to parse lock file: {}", e), nargo_types::Span::unknown()))?;

        info!("Loaded lock file with {} packages", lock.packages.len());
        Ok(lock)
    }

    /// Loads a lock file or returns default if not found.
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_default()
    }

    /// Saves the lock file to the given path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = oak_json::to_string(self).map_err(|e| Error::external_error("lock".to_string(), format!("Failed to serialize lock file: {}", e), nargo_types::Span::unknown()))?;
        fs::write(path_ref, content)?;

        info!("Saved lock file with {} packages", self.packages.len());
        Ok(())
    }

    /// Adds a package to the lock file.
    pub fn add_package(&mut self, entry: LockEntry) {
        let key = entry.key();
        self.packages.insert(key, entry);
    }

    /// Removes a package from the lock file.
    pub fn remove_package(&mut self, name: &str, version: &str) {
        let key = format!("{}@{}", name, version);
        self.packages.remove(&key);
    }

    /// Gets a package by name and version.
    pub fn get_package(&self, name: &str, version: &str) -> Option<&LockEntry> {
        let key = format!("{}@{}", name, version);
        self.packages.get(&key)
    }

    /// Gets all packages with a given name (any version).
    pub fn get_versions(&self, name: &str) -> Vec<&LockEntry> {
        self.packages.values().filter(|p| p.name == name).collect()
    }

    /// Checks if a package is locked.
    pub fn has_package(&self, name: &str, version: &str) -> bool {
        self.get_package(name, version).is_some()
    }

    /// Returns the number of locked packages.
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Checks if the lock file is empty.
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Returns all package entries.
    pub fn entries(&self) -> impl Iterator<Item = &LockEntry> {
        self.packages.values()
    }

    /// Updates the generation timestamp.
    pub fn touch(&mut self) {
        self.metadata.generated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    }

    /// Returns the default lock file path for a project.
    pub fn default_path(project_root: &Path) -> PathBuf {
        project_root.join("nargo.lock")
    }
}
