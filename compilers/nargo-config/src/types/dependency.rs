use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a dependency which can be either a simple version string
/// or a detailed configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string dependency.
    Version(String),

    /// Detailed dependency configuration.
    Detailed(DependencyDetail),
}

impl Dependency {
    /// Returns the version if this is a simple version dependency.
    pub fn version(&self) -> Option<&str> {
        match self {
            Dependency::Version(v) => Some(v),
            Dependency::Detailed(d) => d.version.as_deref(),
        }
    }

    /// Returns true if this is a git dependency.
    pub fn is_git(&self) -> bool {
        matches!(self, Dependency::Detailed(d) if d.git.is_some())
    }

    /// Returns true if this is a path dependency.
    pub fn is_path(&self) -> bool {
        matches!(self, Dependency::Detailed(d) if d.path.is_some())
    }

    /// Returns true if this is a workspace dependency.
    pub fn is_workspace(&self) -> bool {
        matches!(self, Dependency::Detailed(d) if d.workspace.is_some())
    }

    /// Returns the git URL if this is a git dependency.
    pub fn git_url(&self) -> Option<&str> {
        match self {
            Dependency::Detailed(d) => d.git.as_deref(),
            _ => None,
        }
    }

    /// Returns the path if this is a path dependency.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Dependency::Detailed(d) => d.path.as_deref(),
            _ => None,
        }
    }

    /// Returns the features enabled for this dependency.
    pub fn features(&self) -> &[String] {
        match self {
            Dependency::Detailed(d) => &d.features,
            _ => &[],
        }
    }

    /// Returns whether this is an optional dependency.
    pub fn optional(&self) -> bool {
        match self {
            Dependency::Detailed(d) => d.optional,
            _ => false,
        }
    }
}

/// Detailed dependency configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDetail {
    /// Version requirement (semver).
    #[serde(default)]
    pub version: Option<String>,

    /// Git repository URL.
    #[serde(default)]
    pub git: Option<String>,

    /// Git branch to use.
    #[serde(default)]
    pub branch: Option<String>,

    /// Git tag to use.
    #[serde(default)]
    pub tag: Option<String>,

    /// Git commit hash.
    #[serde(default)]
    pub rev: Option<String>,

    /// Local path to the dependency.
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Workspace dependency reference.
    #[serde(default)]
    pub workspace: Option<bool>,

    /// Registry to fetch the package from.
    #[serde(default)]
    pub registry: Option<String>,

    /// Package name if different from dependency name (alias).
    #[serde(default, rename = "package")]
    pub package_name: Option<String>,

    /// Features to enable for this dependency.
    #[serde(default)]
    pub features: Vec<String>,

    /// Whether this is an optional dependency.
    #[serde(default)]
    pub optional: bool,

    /// Whether to use default features.
    #[serde(default = "default_true", rename = "default-features")]
    pub default_features: bool,

    /// Platform-specific dependency conditions.
    #[serde(default)]
    pub target: Option<String>,
}

/// Returns the default true value.
pub fn default_true() -> bool {
    true
}

impl Default for DependencyDetail {
    fn default() -> Self {
        Self { version: None, git: None, branch: None, tag: None, rev: None, path: None, workspace: None, registry: None, package_name: None, features: Vec::new(), optional: false, default_features: true, target: None }
    }
}
