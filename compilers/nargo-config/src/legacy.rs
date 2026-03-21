use nargo_types::errors::Result;
use serde::{Deserialize, Serialize};
use oak_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Legacy task configuration for build scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// The command to execute for this task.
    pub command: String,

    /// List of tasks that must complete before this task runs.
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Whether this task can run in parallel with other tasks.
    #[serde(default)]
    pub parallel: bool,
}

/// Legacy Nargo configuration for project settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NargoConfig {
    /// The root directory of the project.
    #[serde(default = "default_root")]
    pub root: PathBuf,

    /// Output directory for build artifacts.
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,

    /// Development server configuration.
    #[serde(default)]
    pub dev_server: DevServerConfig,

    /// Mock server configuration.
    #[serde(default)]
    pub mock: MockConfig,

    /// Test runner configuration.
    #[serde(default)]
    pub test: TestConfig,

    /// List of plugin names to load.
    #[serde(default)]
    pub plugins: Vec<String>,

    /// Named task definitions.
    #[serde(default)]
    pub tasks: HashMap<String, TaskConfig>,

    /// Hook scripts for lifecycle events.
    #[serde(default)]
    pub hooks: HashMap<String, String>,
}

fn default_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("dist")
}

/// Development server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerConfig {
    /// Port number for the development server.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Host address to bind the server to.
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

/// Mock server configuration for API simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Whether the mock server is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Port number for the mock server.
    #[serde(default = "default_mock_port")]
    pub port: u16,

    /// Directory containing mock data files.
    #[serde(default = "default_mock_dir")]
    pub dir: PathBuf,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_mock_port(),
            dir: default_mock_dir(),
        }
    }
}

fn default_mock_port() -> u16 {
    4000
}

fn default_mock_dir() -> PathBuf {
    PathBuf::from("mocks")
}

/// Test runner configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Whether to collect code coverage.
    #[serde(default)]
    pub coverage: bool,

    /// Glob patterns for test file discovery.
    #[serde(default = "default_test_include")]
    pub include: Vec<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            coverage: false,
            include: default_test_include(),
        }
    }
}

fn default_test_include() -> Vec<String> {
    vec!["**/*.test.ts".to_string(), "**/*.spec.ts".to_string()]
}

impl Default for NargoConfig {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            out_dir: PathBuf::from("dist"),
            dev_server: DevServerConfig::default(),
            mock: MockConfig::default(),
            test: TestConfig::default(),
            plugins: Vec::new(),
            tasks: HashMap::new(),
            hooks: HashMap::new(),
        }
    }
}

impl NargoConfig {
    /// Loads a NargoConfig from the specified file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)?;

        let config: NargoConfig = if path_ref.extension().map_or(false, |ext| ext == "toml") {
            oak_toml::from_str(&content)?
        } else {
            oak_json::from_str(&content)?
        };
        Ok(config)
    }

    /// Loads a NargoConfig or returns the default if loading fails.
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_default()
    }
}
