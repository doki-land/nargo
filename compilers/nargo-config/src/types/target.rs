use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::dependency::Dependency;

/// Target-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Target-specific dependencies.
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Target-specific dev dependencies.
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,

    /// Target-specific build dependencies.
    #[serde(default, rename = "build-dependencies")]
    pub build_dependencies: HashMap<String, Dependency>,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self { dependencies: HashMap::new(), dev_dependencies: HashMap::new(), build_dependencies: HashMap::new() }
    }
}
