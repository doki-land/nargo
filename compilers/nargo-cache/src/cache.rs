//! Cache management implementation.

use nargo_types::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::{info, warn};

use crate::{integrity::Integrity, storage::Storage};

/// Represents a cached package entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Integrity hash.
    pub integrity: Integrity,
    /// Time when cached.
    pub cached_at: u64,
    /// Size in bytes.
    pub size: u64,
    /// Source registry or git URL.
    pub source: String,
}

impl CacheEntry {
    /// Creates a new cache entry.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), integrity: Integrity::default(), cached_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(), size: 0, source: String::new() }
    }

    /// Sets the integrity hash.
    pub fn with_integrity(mut self, integrity: Integrity) -> Self {
        self.integrity = integrity;
        self
    }

    /// Sets the size.
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Sets the source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Returns the cache key for this entry.
    pub fn key(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cached packages.
    pub total_packages: usize,
    /// Total cache size in bytes.
    pub total_size: u64,
    /// Number of registry packages.
    pub registry_packages: usize,
    /// Number of git packages.
    pub git_packages: usize,
}

/// Global cache manager.
#[derive(Debug)]
pub struct Cache {
    /// Storage backend.
    storage: Storage,
    /// In-memory index of cached packages.
    index: HashMap<String, CacheEntry>,
}

impl Cache {
    /// Creates a new cache instance.
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let index = storage.load_index()?;
        Ok(Self { storage, index })
    }

    /// Creates a cache with a custom root directory.
    pub fn with_root(root: PathBuf) -> Result<Self> {
        let storage = Storage::with_root(root)?;
        let index = storage.load_index()?;
        Ok(Self { storage, index })
    }

    /// Gets a cached package.
    pub fn get(&self, name: &str, version: &str) -> Option<&CacheEntry> {
        let key = format!("{}@{}", name, version);
        self.index.get(&key)
    }

    /// Checks if a package is cached.
    pub fn has(&self, name: &str, version: &str) -> bool {
        self.get(name, version).is_some()
    }

    /// Adds a package to the cache.
    pub async fn add(&mut self, name: &str, version: &str, tarball: &[u8], source: &str) -> Result<CacheEntry> {
        let integrity = Integrity::from_bytes(tarball);
        let entry = CacheEntry::new(name, version).with_integrity(integrity.clone()).with_size(tarball.len() as u64).with_source(source);

        self.storage.store_package(name, version, tarball).await?;
        self.storage.store_metadata(name, version, &entry).await?;

        let key = entry.key();
        self.index.insert(key, entry.clone());

        info!("Cached {}@{} ({} bytes)", name, version, tarball.len());
        Ok(entry)
    }

    /// Retrieves a cached package's tarball.
    pub async fn get_tarball(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let entry = self.get(name, version).ok_or_else(|| Error::external_error("cache".to_string(), format!("Package {}@{} not in cache", name, version), nargo_types::Span::unknown()))?;

        let tarball = self.storage.load_package(name, version).await?;

        let computed = Integrity::from_bytes(&tarball);
        if computed != entry.integrity {
            warn!("Integrity mismatch for {}@{}, removing from cache", name, version);
            return Err(Error::external_error("cache".to_string(), "Integrity verification failed".to_string(), nargo_types::Span::unknown()));
        }

        Ok(tarball)
    }

    /// Removes a package from the cache.
    pub async fn remove(&mut self, name: &str, version: &str) -> Result<()> {
        let key = format!("{}@{}", name, version);
        self.index.remove(&key);
        self.storage.remove_package(name, version).await?;
        self.storage.save_index(&self.index).await?;
        info!("Removed {}@{} from cache", name, version);
        Ok(())
    }

    /// Clears the entire cache.
    pub async fn clear(&mut self) -> Result<()> {
        self.index.clear();
        self.storage.clear().await?;
        info!("Cache cleared");
        Ok(())
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();
        for entry in self.index.values() {
            stats.total_packages += 1;
            stats.total_size += entry.size;
            if entry.source.starts_with("http") || entry.source.starts_with("https") {
                stats.registry_packages += 1;
            }
            else if entry.source.starts_with("git") {
                stats.git_packages += 1;
            }
        }
        stats
    }

    /// Returns all cached entries.
    pub fn entries(&self) -> impl Iterator<Item = &CacheEntry> {
        self.index.values()
    }

    /// Returns the cache root directory.
    pub fn root(&self) -> &Path {
        self.storage.root()
    }

    /// Verifies integrity of all cached packages.
    pub async fn verify(&self) -> Result<Vec<String>> {
        let mut failed = Vec::new();
        for entry in self.index.values() {
            if let Ok(tarball) = self.storage.load_package(&entry.name, &entry.version).await {
                let computed = Integrity::from_bytes(&tarball);
                if computed != entry.integrity {
                    failed.push(entry.key());
                }
            }
            else {
                failed.push(entry.key());
            }
        }
        Ok(failed)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize cache")
    }
}
