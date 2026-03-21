use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Script/Task configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScriptConfig {
    /// Simple command string.
    Command(String),
    /// Detailed script configuration.
    Detailed(DetailedScriptConfig),
}

impl ScriptConfig {
    /// Returns the command string.
    pub fn command(&self) -> &str {
        match self {
            ScriptConfig::Command(cmd) => cmd,
            ScriptConfig::Detailed(d) => &d.command,
        }
    }
}

impl Default for ScriptConfig {
    fn default() -> Self {
        ScriptConfig::Command(String::new())
    }
}

/// Detailed script configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedScriptConfig {
    /// The command to run.
    pub command: String,
    /// Current working directory for the command.
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Dependencies on other scripts.
    #[serde(default)]
    pub depends_on: Vec<String>,
}
