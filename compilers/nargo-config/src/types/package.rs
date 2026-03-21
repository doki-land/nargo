use serde::{Deserialize, Serialize};
use nargo_types::NargoValue;

/// Package metadata configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// The name of the package.
    pub name: String,

    /// The version of the package (semver format).
    pub version: String,

    /// The edition to use (e.g., "2024" for latest TS/JS features).
    #[serde(default = "default_edition")]
    pub edition: String,

    /// List of package authors.
    #[serde(default)]
    pub authors: Vec<String>,

    /// Package description.
    #[serde(default)]
    pub description: Option<String>,

    /// Documentation URL.
    #[serde(default)]
    pub documentation: Option<String>,

    /// Readme file path.
    #[serde(default)]
    pub readme: Option<String>,

    /// Homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Repository URL.
    #[serde(default)]
    pub repository: Option<String>,

    /// License identifier (SPDX format).
    #[serde(default)]
    pub license: Option<String>,

    /// License file path.
    #[serde(default)]
    pub license_file: Option<String>,

    /// Keywords for package discovery.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Categories for package classification.
    #[serde(default)]
    pub categories: Vec<String>,

    /// Whether this package should be published.
    #[serde(default)]
    pub publish: Option<PublishConfig>,

    /// Exclude patterns for packaging.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Include patterns for packaging.
    #[serde(default)]
    pub include: Vec<String>,

    /// Whether this package is part of a workspace.
    #[serde(default)]
    pub workspace: Option<bool>,

    /// Default run target for binaries.
    #[serde(default)]
    pub default_run: Option<String>,

    /// Metdata for custom use.
    #[serde(default)]
    pub metadata: Option<NargoValue>,
}

/// Returns the default edition string.
pub fn default_edition() -> String {
    "2024".to_string()
}

/// Publish configuration - can be boolean or list of registries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublishConfig {
    /// Whether to publish at all.
    Bool(bool),
    /// List of registries to publish to.
    Registries(Vec<String>),
}

impl Default for PublishConfig {
    fn default() -> Self {
        PublishConfig::Bool(true)
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self { name: String::new(), version: String::new(), edition: default_edition(), authors: Vec::new(), description: None, documentation: None, readme: None, homepage: None, repository: None, license: None, license_file: None, keywords: Vec::new(), categories: Vec::new(), publish: None, exclude: Vec::new(), include: Vec::new(), workspace: None, default_run: None, metadata: None }
    }
}
