use serde::{Deserialize, Serialize};
use nargo_types::NargoValue;
use std::collections::HashMap;

use super::{dependency::Dependency, package::PublishConfig};

/// Workspace configuration for multi-package projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// List of member packages in the workspace.
    #[serde(default)]
    pub members: Vec<String>,

    /// Members to exclude from the workspace.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Workspace resolver version.
    #[serde(default = "default_resolver")]
    pub resolver: String,

    /// Shared package metadata for workspace members.
    #[serde(default)]
    pub package: Option<WorkspacePackage>,

    /// Shared dependencies for all workspace members.
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Shared dev-dependencies for all workspace members.
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,

    /// Default members to operate on.
    #[serde(default)]
    pub default_members: Vec<String>,

    /// Workspace-level metadata.
    #[serde(default)]
    pub metadata: Option<NargoValue>,
}

/// Returns the default resolver string.
pub fn default_resolver() -> String {
    "2".to_string()
}

/// Shared package metadata that workspace members can inherit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// Shared version for workspace members.
    #[serde(default)]
    pub version: Option<String>,

    /// Shared authors list.
    #[serde(default)]
    pub authors: Vec<String>,

    /// Shared edition.
    #[serde(default)]
    pub edition: Option<String>,

    /// Shared description.
    #[serde(default)]
    pub description: Option<String>,

    /// Shared documentation URL.
    #[serde(default)]
    pub documentation: Option<String>,

    /// Shared homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Shared repository URL.
    #[serde(default)]
    pub repository: Option<String>,

    /// Shared license.
    #[serde(default)]
    pub license: Option<String>,

    /// Shared license file.
    #[serde(default)]
    pub license_file: Option<String>,

    /// Shared readme.
    #[serde(default)]
    pub readme: Option<String>,

    /// Shared keywords.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Shared categories.
    #[serde(default)]
    pub categories: Vec<String>,

    /// Shared publish configuration.
    #[serde(default)]
    pub publish: Option<PublishConfig>,

    /// Shared exclude patterns.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Shared include patterns.
    #[serde(default)]
    pub include: Vec<String>,
}
