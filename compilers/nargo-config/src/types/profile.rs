use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Build profile configurations for different scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Debug profile settings.
    #[serde(default)]
    pub dev: Option<ProfileSettings>,

    /// Release profile settings.
    #[serde(default)]
    pub release: Option<ProfileSettings>,

    /// Test profile settings.
    #[serde(default)]
    pub test: Option<ProfileSettings>,

    /// Benchmark profile settings.
    #[serde(default)]
    pub bench: Option<ProfileSettings>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self { dev: Some(ProfileSettings::default_dev()), release: Some(ProfileSettings::default_release()), test: None, bench: None }
    }
}

/// Individual profile settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    /// Optimization level (0-3, 's', 'z').
    #[serde(default, rename = "opt-level")]
    pub opt_level: Option<OptLevel>,

    /// Number of codegen units.
    #[serde(default, rename = "codegen-units")]
    pub codegen_units: Option<u32>,

    /// Debug information level.
    #[serde(default)]
    pub debug: Option<DebugLevel>,

    /// Strip symbols from binary.
    #[serde(default)]
    pub strip: Option<StripLevel>,

    /// Enable link-time optimization.
    #[serde(default)]
    pub lto: Option<LtoLevel>,

    /// Enable overflow checks.
    #[serde(default, rename = "overflow-checks")]
    pub overflow_checks: Option<bool>,

    /// Panic strategy.
    #[serde(default)]
    pub panic: Option<PanicStrategy>,

    /// Incremental compilation.
    #[serde(default)]
    pub incremental: Option<bool>,

    /// Generate source maps.
    #[serde(default, rename = "source-map")]
    pub source_map: Option<bool>,

    /// Minify output.
    #[serde(default)]
    pub minify: Option<bool>,

    /// Target directory override.
    #[serde(default)]
    pub dir: Option<PathBuf>,

    /// Inherits from another profile.
    #[serde(default)]
    pub inherits: Option<String>,
}

/// Optimization level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OptLevel {
    /// Numeric level.
    Number(u8),
    /// String level ('s' or 'z').
    String(String),
}

impl OptLevel {
    /// Returns the default dev optimization level.
    pub fn default_dev() -> Self {
        OptLevel::Number(0)
    }

    /// Returns the default release optimization level.
    pub fn default_release() -> Self {
        OptLevel::Number(3)
    }
}

impl ProfileSettings {
    /// Creates default debug profile settings.
    pub fn default_dev() -> Self {
        Self { opt_level: Some(OptLevel::default_dev()), codegen_units: Some(256), debug: Some(DebugLevel::Full), strip: None, lto: None, overflow_checks: Some(true), panic: Some(PanicStrategy::Unwind), incremental: Some(true), source_map: Some(true), minify: Some(false), dir: None, inherits: None }
    }

    /// Creates default release profile settings.
    pub fn default_release() -> Self {
        Self { opt_level: Some(OptLevel::default_release()), codegen_units: Some(16), debug: Some(DebugLevel::None), strip: Some(StripLevel::Symbols), lto: Some(LtoLevel::Thin), overflow_checks: Some(false), panic: Some(PanicStrategy::Abort), incremental: Some(false), source_map: Some(false), minify: Some(true), dir: None, inherits: None }
    }
}

/// Debug information level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DebugLevel {
    /// No debug information.
    None,
    /// Line tables only.
    LineTablesOnly,
    /// Limited debug information.
    Limited,
    /// Full debug information.
    Full,
}

/// Symbol stripping level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StripLevel {
    /// No stripping.
    None,
    /// Strip debug info.
    DebugInfo,
    /// Strip all symbols.
    Symbols,
}

/// Link-time optimization level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LtoLevel {
    /// No LTO.
    None,
    /// Thin LTO.
    Thin,
    /// Fat LTO.
    Fat,
}

/// Panic strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PanicStrategy {
    /// Unwind the stack.
    Unwind,
    /// Abort the process.
    Abort,
}
