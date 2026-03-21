use serde::{Deserialize, Serialize};
use nargo_types::NargoValue;
use std::collections::HashMap;

use super::dependency::default_true;

/// Linting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    /// Enable linting.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Lint rules configuration.
    #[serde(default)]
    pub rules: HashMap<String, LintRuleConfig>,

    /// Files to include.
    #[serde(default)]
    pub include: Vec<String>,

    /// Files to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self { enabled: true, rules: HashMap::new(), include: vec!["**/*.ts".to_string(), "**/*.tsx".to_string(), "**/*.js".to_string()], exclude: vec!["**/node_modules/**".to_string(), "**/target/**".to_string()] }
    }
}

/// Lint rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRuleConfig {
    /// Rule severity.
    pub level: LintLevel,

    /// Rule-specific options.
    #[serde(default)]
    pub options: Option<NargoValue>,
}

/// Lint severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LintLevel {
    /// Off - rule is disabled.
    Off,
    /// Warn - show warning.
    Warn,
    /// Error - fail the build.
    Error,
}
