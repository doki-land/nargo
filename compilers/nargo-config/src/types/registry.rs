use serde::{Deserialize, Serialize};

/// Registry configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Registry URL.
    pub url: String,

    /// Authentication token (can use env vars like ${NPM_TOKEN}).
    #[serde(default, rename = "auth-token")]
    pub auth_token: Option<String>,
}

impl Default for RegistryEntry {
    fn default() -> Self {
        Self { url: String::new(), auth_token: None }
    }
}
