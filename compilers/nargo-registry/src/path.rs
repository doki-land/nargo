use nargo_types::{Error, NargoValue, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::types::{DependencySpec, ResolvedDependency, ResolvedSource};

/// Local path dependency resolver.
#[derive(Debug)]
pub struct PathResolver {
    /// The root directory for resolving relative paths.
    root_dir: PathBuf,
}

impl PathResolver {
    /// Creates a new path resolver with the specified root directory.
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Creates a new path resolver using the current working directory.
    pub fn with_current_dir() -> Result<Self> {
        let root_dir = std::env::current_dir().map_err(|e| Error::external_error("path".to_string(), format!("Failed to get current directory: {}", e), nargo_types::Span::unknown()))?;
        Ok(Self { root_dir })
    }

    /// Parses a path dependency specification.
    ///
    /// Supports formats:
    /// - `file:./relative/path`
    /// - `file:../relative/path`
    /// - `file:/absolute/path`
    /// - `./relative/path`
    /// - `../relative/path`
    pub fn parse_path_spec(input: &str) -> Result<DependencySpec> {
        let path_str = if input.starts_with("file:") {
            &input[5..]
        }
        else if input.starts_with("./") || input.starts_with("../") || input.starts_with('/') {
            input
        }
        else {
            return Err(Error::external_error("path".to_string(), format!("Invalid path dependency format: {}", input), nargo_types::Span::unknown()));
        };

        Ok(DependencySpec::Path { path: PathBuf::from(path_str) })
    }

    /// Resolves a path dependency.
    ///
    /// # Arguments
    /// * `spec` - The path dependency specification.
    ///
    /// # Returns
    /// The resolved dependency with absolute path.
    pub fn resolve(&self, spec: &DependencySpec) -> Result<ResolvedDependency> {
        match spec {
            DependencySpec::Path { path } => self.resolve_path(path),
            _ => Err(Error::external_error("path".to_string(), "Not a path dependency".to_string(), nargo_types::Span::unknown())),
        }
    }

    /// Resolves a path to an absolute path and validates it.
    fn resolve_path(&self, path: &Path) -> Result<ResolvedDependency> {
        let absolute_path = if path.is_absolute() { path.to_path_buf() } else { self.root_dir.join(path) };

        let canonical_path = absolute_path.canonicalize().map_err(|_| Error::external_error("path".to_string(), format!("Path does not exist: {}", absolute_path.display()), nargo_types::Span::unknown()))?;

        debug!("Resolved path: {} -> {}", path.display(), canonical_path.display());

        if !canonical_path.is_dir() {
            return Err(Error::external_error("path".to_string(), format!("Not a valid package directory: {}", canonical_path.display()), nargo_types::Span::unknown()));
        }

        let package_info = self.read_package_info(&canonical_path)?;

        Ok(ResolvedDependency { name: package_info.name, version: package_info.version, resolved: ResolvedSource::LocalPath(canonical_path), integrity: None })
    }

    /// Reads package information from a directory.
    fn read_package_info(&self, dir: &Path) -> Result<PackageInfo> {
        let package_json_path = dir.join("package.json");

        if !package_json_path.exists() {
            return Err(Error::external_error("path".to_string(), format!("No package.json found in {}", dir.display()), nargo_types::Span::unknown()));
        }

        let content = std::fs::read_to_string(&package_json_path).map_err(|e| Error::external_error("path".to_string(), format!("Failed to read {}: {}", package_json_path.display(), e), nargo_types::Span::unknown()))?;

        let json: NargoValue = serde_json::from_str(&content).map_err(|e| Error::external_error("path".to_string(), format!("Failed to parse {}: {}", package_json_path.display(), e), nargo_types::Span::unknown()))?;

        let name = json.get("name").and_then(|n| n.as_str()).map(String::from).unwrap_or_else(|| dir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string());

        let version = json.get("version").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| "0.0.0".to_string());

        info!("Found package: {}@{} at {}", name, version, dir.display());

        Ok(PackageInfo { name, version })
    }

    /// Checks if a path is a valid local package.
    pub fn is_valid_package(&self, path: &Path) -> bool {
        let absolute_path = if path.is_absolute() { path.to_path_buf() } else { self.root_dir.join(path) };

        if let Ok(canonical_path) = absolute_path.canonicalize() { canonical_path.is_dir() && canonical_path.join("package.json").exists() } else { false }
    }

    /// Gets the root directory for this resolver.
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
}

/// Package information extracted from a local package.
#[derive(Debug, Clone)]
struct PackageInfo {
    /// The package name.
    name: String,
    /// The package version.
    version: String,
}

/// Utility functions for working with path dependencies.
pub struct PathUtils;

impl PathUtils {
    /// Normalizes a path dependency string.
    pub fn normalize_path(input: &str) -> String {
        let path = if input.starts_with("file:") { &input[5..] } else { input };

        path.replace('\\', "/")
    }

    /// Checks if a string looks like a path dependency.
    pub fn is_path_dependency(input: &str) -> bool {
        input.starts_with("file:") || input.starts_with("./") || input.starts_with("../") || (input.starts_with('/') && !input.contains("://"))
    }

    /// Converts a relative path to a file: URL.
    pub fn to_file_url(path: &Path) -> Result<String> {
        let absolute = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().map_err(|e| Error::external_error("path".to_string(), format!("Failed to get current directory: {}", e), nargo_types::Span::unknown()))?.join(path) };

        let url = url::Url::from_file_path(&absolute).map_err(|_| Error::external_error("path".to_string(), format!("Failed to convert path to URL: {}", absolute.display()), nargo_types::Span::unknown()))?;

        Ok(url.to_string())
    }
}
