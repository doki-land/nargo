//! Storage backend implementation.

use nargo_types::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;

use crate::cache::CacheEntry;

/// Storage location configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    /// Root directory for cache storage.
    pub root: PathBuf,
    /// Registry packages directory.
    pub registry_dir: PathBuf,
    /// Git packages directory.
    pub git_dir: PathBuf,
    /// Index file path.
    pub index_file: PathBuf,
}

impl Default for StorageLocation {
    fn default() -> Self {
        let root = Self::default_cache_root();
        Self::from_root(root)
    }
}

impl StorageLocation {
    /// Build layout under a cache root (registry / git / index live directly here).
    pub fn from_root(root: PathBuf) -> Self {
        Self {
            registry_dir: root.join("registry"),
            git_dir: root.join("git"),
            index_file: root.join("index.json"),
            root,
        }
    }

    /// Returns the default cache root: `<cwd>/.cache/nargo`.
    ///
    /// Callers with a known workspace root should prefer
    /// [`nargo_types::cache_dir`] instead so the path is not cwd-relative.
    pub fn default_cache_root() -> PathBuf {
        nargo_types::cache_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

/// Storage backend for cache management.
#[derive(Debug)]
pub struct Storage {
    location: StorageLocation,
}

impl Storage {
    /// Creates a new storage instance with default location.
    pub fn new() -> Result<Self> {
        let location = StorageLocation::default();
        let storage = Self { location };
        storage.ensure_directories()?;
        Ok(storage)
    }

    /// Creates a storage with a custom root directory.
    pub fn with_root(root: PathBuf) -> Result<Self> {
        let location = StorageLocation::from_root(root);
        let storage = Self { location };
        storage.ensure_directories()?;
        Ok(storage)
    }

    /// Ensures all required directories exist.
    fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.location.root)?;
        fs::create_dir_all(&self.location.registry_dir)?;
        fs::create_dir_all(&self.location.git_dir)?;
        Ok(())
    }

    /// Returns the cache root directory.
    pub fn root(&self) -> &Path {
        &self.location.root
    }

    /// Stores a package tarball.
    pub async fn store_package(&self, name: &str, version: &str, tarball: &[u8]) -> Result<()> {
        let dir = self.location.registry_dir.join(name).join(version);
        fs::create_dir_all(&dir)?;

        let tarball_path = dir.join("package.tgz");
        let mut file = tokio::fs::File::create(&tarball_path).await?;
        file.write_all(tarball).await?;

        Ok(())
    }

    /// Loads a package tarball.
    pub async fn load_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let tarball_path = self.location.registry_dir.join(name).join(version).join("package.tgz");
        tokio::fs::read(&tarball_path).await.map_err(|e| Error::external_error("storage".to_string(), format!("Failed to read tarball: {}", e), nargo_types::Span::unknown()))
    }

    /// Stores package metadata.
    pub async fn store_metadata(&self, name: &str, version: &str, entry: &CacheEntry) -> Result<()> {
        let dir = self.location.registry_dir.join(name).join(version);
        fs::create_dir_all(&dir)?;

        let metadata_path = dir.join("metadata.json");
        let content = serde_json::to_string_pretty(entry).map_err(|e| Error::external_error("serde_json".to_string(), format!("Failed to serialize metadata: {}", e), nargo_types::Span::unknown()))?;
        tokio::fs::write(&metadata_path, content).await?;

        Ok(())
    }

    /// Removes a package from storage.
    pub async fn remove_package(&self, name: &str, version: &str) -> Result<()> {
        let dir = self.location.registry_dir.join(name).join(version);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await?;
        }
        Ok(())
    }

    /// Clears all cached data.
    pub async fn clear(&self) -> Result<()> {
        if self.location.registry_dir.exists() {
            tokio::fs::remove_dir_all(&self.location.registry_dir).await?;
        }
        if self.location.git_dir.exists() {
            tokio::fs::remove_dir_all(&self.location.git_dir).await?;
        }
        if self.location.index_file.exists() {
            tokio::fs::remove_file(&self.location.index_file).await?;
        }
        self.ensure_directories()?;
        Ok(())
    }

    /// Loads the cache index.
    pub fn load_index(&self) -> Result<HashMap<String, CacheEntry>> {
        if !self.location.index_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.location.index_file)?;
        let index: HashMap<String, CacheEntry> = serde_json::from_str(&content).map_err(|e| Error::external_error("serde_json".to_string(), format!("Failed to deserialize index: {}", e), nargo_types::Span::unknown()))?;
        Ok(index)
    }

    /// Saves the cache index.
    pub async fn save_index(&self, index: &HashMap<String, CacheEntry>) -> Result<()> {
        let content = serde_json::to_string_pretty(index).map_err(|e| Error::external_error("serde_json".to_string(), format!("Failed to serialize index: {}", e), nargo_types::Span::unknown()))?;
        tokio::fs::write(&self.location.index_file, content).await?;
        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
