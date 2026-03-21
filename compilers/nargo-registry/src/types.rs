use nargo_types::NargoValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a package metadata from npm registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// The name of the package.
    pub name: String,
    /// The description of the package.
    #[serde(default)]
    pub description: Option<String>,
    /// All available versions of the package.
    pub versions: HashMap<String, PackageVersion>,
    /// Distribution tags like "latest", "next", etc.
    #[serde(rename = "dist-tags", default)]
    pub dist_tags: HashMap<String, String>,
    /// The time when each version was published.
    #[serde(rename = "time", default)]
    pub time: HashMap<String, String>,
    /// The latest version shortcut.
    #[serde(default)]
    pub latest: Option<String>,
}

/// Represents a specific version of a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    /// The name of the package.
    pub name: String,
    /// The version string.
    pub version: String,
    /// The description of this version.
    #[serde(default)]
    pub description: Option<String>,
    /// The main entry point.
    #[serde(default)]
    pub main: Option<String>,
    /// The module entry point for ESM.
    #[serde(default)]
    pub module: Option<String>,
    /// The types entry point.
    #[serde(default)]
    pub types: Option<String>,
    /// The typings entry point (alternative to types).
    #[serde(default)]
    pub typings: Option<String>,
    /// The exports map.
    #[serde(default)]
    pub exports: Option<NargoValue>,
    /// The dependencies.
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// The dev dependencies.
    #[serde(default)]
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    /// The peer dependencies.
    #[serde(default)]
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    /// The optional dependencies.
    #[serde(default)]
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
    /// The distribution information.
    pub dist: DistributionInfo,
    /// The license.
    #[serde(default)]
    pub license: Option<String>,
    /// The repository information.
    #[serde(default)]
    pub repository: Option<RepositoryInfo>,
    /// The homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,
    /// The bugs URL.
    #[serde(default)]
    pub bugs: Option<BugsInfo>,
    /// The author information.
    #[serde(default)]
    pub author: Option<PersonInfo>,
    /// The contributors.
    #[serde(default)]
    pub contributors: Vec<PersonInfo>,
    /// The maintainers.
    #[serde(default)]
    pub maintainers: Vec<PersonInfo>,
    /// The keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// The engines.
    #[serde(default)]
    pub engines: Option<EnginesInfo>,
    /// Whether the package is deprecated.
    #[serde(default)]
    pub deprecated: Option<String>,
    /// The scripts.
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

/// Distribution information for a package version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionInfo {
    /// The integrity hash (e.g., sha512-...).
    pub integrity: Option<String>,
    /// The shasum hash.
    pub shasum: Option<String>,
    /// The tarball URL.
    pub tarball: String,
    /// The file count.
    #[serde(default)]
    #[serde(rename = "fileCount")]
    pub file_count: Option<u64>,
    /// The unpacked size.
    #[serde(default)]
    #[serde(rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,
    /// The npm signature.
    #[serde(default)]
    #[serde(rename = "npm-signature")]
    pub npm_signature: Option<String>,
}

/// Repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// The repository type (e.g., "git").
    #[serde(rename = "type")]
    pub repo_type: Option<String>,
    /// The repository URL.
    pub url: Option<String>,
    /// The repository directory (for monorepos).
    #[serde(default)]
    pub directory: Option<String>,
}

/// Bugs information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugsInfo {
    /// The bugs URL.
    pub url: Option<String>,
    /// The bugs email.
    pub email: Option<String>,
}

/// Person information (author, contributor, maintainer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonInfo {
    /// The person's name.
    pub name: Option<String>,
    /// The person's email.
    pub email: Option<String>,
    /// The person's URL.
    pub url: Option<String>,
}

/// Engines information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginesInfo {
    /// The required Node.js version.
    pub node: Option<String>,
    /// The required npm version.
    pub npm: Option<String>,
}

/// Search result from npm registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The search results.
    pub objects: Vec<SearchObject>,
    /// The total number of results.
    pub total: u64,
    /// The time taken for the search.
    pub time: String,
}

/// A single search result object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchObject {
    /// The search result score.
    pub score: SearchScore,
    /// The search result package.
    pub package: SearchPackage,
    /// The highlight information.
    #[serde(default)]
    pub highlight: Option<String>,
}

/// Search score information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchScore {
    /// The final score.
    #[serde(rename = "final")]
    pub final_score: f64,
    /// The detail scores.
    pub detail: SearchScoreDetail,
}

/// Detailed search score breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchScoreDetail {
    /// The quality score.
    pub quality: f64,
    /// The popularity score.
    pub popularity: f64,
    /// The maintenance score.
    pub maintenance: f64,
}

/// Package information in search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPackage {
    /// The package name.
    pub name: String,
    /// The package scope.
    #[serde(default)]
    pub scope: Option<String>,
    /// The package version.
    pub version: String,
    /// The package description.
    #[serde(default)]
    pub description: Option<String>,
    /// The package keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// The package date.
    #[serde(default)]
    pub date: Option<String>,
    /// The package author.
    #[serde(default)]
    pub author: Option<PersonInfo>,
    /// The package publisher.
    #[serde(default)]
    pub publisher: Option<PersonInfo>,
    /// The package maintainers.
    #[serde(default)]
    pub maintainers: Vec<PersonInfo>,
    /// The package links.
    #[serde(default)]
    pub links: Option<PackageLinks>,
}

/// Links for a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageLinks {
    /// The npm link.
    pub npm: Option<String>,
    /// The homepage link.
    pub homepage: Option<String>,
    /// The repository link.
    pub repository: Option<String>,
    /// The bugs link.
    pub bugs: Option<String>,
}

/// Represents a dependency specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySpec {
    /// NPM registry dependency with version constraint.
    Npm {
        /// Package name.
        name: String,
        /// Version constraint.
        version: String,
    },
    /// Git dependency.
    Git {
        /// Git URL.
        url: String,
        /// Optional commit reference (branch, tag, or commit hash).
        reference: Option<GitReference>,
    },
    /// Local path dependency.
    Path {
        /// Path to the local package.
        path: std::path::PathBuf,
    },
    /// GitHub shorthand (user/repo).
    Github {
        /// GitHub repository (user/repo).
        repo: String,
        /// Optional commit reference.
        reference: Option<GitReference>,
    },
}

/// Git reference type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitReference {
    /// A branch name.
    Branch(String),
    /// A tag name.
    Tag(String),
    /// A commit hash.
    Commit(String),
}

/// Resolved dependency information.
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// The package name.
    pub name: String,
    /// The resolved version.
    pub version: String,
    /// The resolved URL or path.
    pub resolved: ResolvedSource,
    /// The integrity hash.
    pub integrity: Option<String>,
}

/// Resolved source location.
#[derive(Debug, Clone)]
pub enum ResolvedSource {
    /// NPM registry tarball URL.
    Tarball(String),
    /// Git repository path.
    GitRepo {
        /// Local path to the cloned repository.
        path: std::path::PathBuf,
        /// The resolved commit hash.
        commit: String,
    },
    /// Local path.
    LocalPath(std::path::PathBuf),
}

/// Download progress information.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download.
    pub total: Option<u64>,
    /// Bytes downloaded so far.
    pub downloaded: u64,
}

/// Configuration for a single registry.
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// The npm registry URL.
    pub registry_url: String,
    /// Authentication token.
    pub auth_token: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Cache directory for downloaded packages.
    pub cache_dir: std::path::PathBuf,
    /// Maximum idle connections per host.
    pub max_idle_connections: usize,
    /// Cache TTL in seconds.
    pub cache_ttl_secs: u64,
    /// Maximum retries on failure.
    pub max_retries: u32,
    /// Retry delay in milliseconds.
    pub retry_delay_ms: u64,
    /// Path to the private key for package signing.
    pub signing_key_path: Option<std::path::PathBuf>,
    /// Path to the public key for package verification.
    pub verification_key_path: Option<std::path::PathBuf>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self { registry_url: "https://registry.npmjs.org".to_string(), auth_token: None, timeout_secs: 30, cache_dir: std::path::PathBuf::from(".nargo-cache"), max_idle_connections: 10, cache_ttl_secs: 300, max_retries: 3, retry_delay_ms: 1000, signing_key_path: None, verification_key_path: None }
    }
}

/// Multi-registry configuration.
#[derive(Debug, Clone)]
pub struct MultiRegistryConfig {
    /// Default registry configuration.
    pub default_registry: RegistryConfig,
    /// Scoped registries mapping scope names to registry configurations.
    pub scoped_registries: std::collections::HashMap<String, RegistryConfig>,
}

impl Default for MultiRegistryConfig {
    fn default() -> Self {
        Self { default_registry: RegistryConfig::default(), scoped_registries: std::collections::HashMap::new() }
    }
}
