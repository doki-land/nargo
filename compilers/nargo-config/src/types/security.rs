use serde::{Deserialize, Serialize};

use super::dependency::default_true;

/// Security configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Allow running package scripts (postinstall, etc.).
    #[serde(default, rename = "allow-scripts")]
    pub allow_scripts: AllowScriptsConfig,

    /// Verify package integrity.
    #[serde(default = "default_true", rename = "verify-integrity")]
    pub verify_integrity: bool,

    /// Audit level for vulnerability checks.
    #[serde(default, rename = "audit-level")]
    pub audit_level: AuditLevel,

    /// Allow running binaries from dependencies.
    #[serde(default = "default_true")]
    pub allow_binaries: bool,

    /// Trust specific packages.
    #[serde(default)]
    pub trust: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self { allow_scripts: AllowScriptsConfig::Bool(false), verify_integrity: true, audit_level: AuditLevel::Moderate, allow_binaries: true, trust: Vec::new() }
    }
}

/// Allow scripts configuration - can be boolean or list of packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllowScriptsConfig {
    /// Boolean to allow/disallow all scripts.
    Bool(bool),
    /// List of packages allowed to run scripts.
    Packages(Vec<String>),
}

impl Default for AllowScriptsConfig {
    fn default() -> Self {
        AllowScriptsConfig::Bool(false)
    }
}

/// Audit severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditLevel {
    /// Low severity and above.
    Low,
    /// Moderate severity and above.
    Moderate,
    /// High severity and above.
    High,
    /// Critical severity only.
    Critical,
    /// Disable audit.
    None,
}

impl Default for AuditLevel {
    fn default() -> Self {
        AuditLevel::Moderate
    }
}
