#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod client;
pub mod git;
pub mod path;
pub mod semver;
pub mod signature;
pub mod types;

pub use client::{RegistryClient, RegistryError};
pub use git::GitResolver;
pub use path::{PathResolver, PathUtils};
pub use semver::{CompareOp, ConstraintType, NpmVersionConstraint, VersionComparator};
pub use signature::{KeyPair, PublicKey, default_key_dir, default_private_key_path, default_public_key_path, sign_package, verify_package};
pub use types::*;

use nargo_types::{Error, Result};
use std::path::Path;

/// Search for packages in the registry.
pub async fn search(_query: &str, _limit: usize) -> Result<Vec<PackageInfo>> {
    Ok(Vec::new())
}

/// Package information.
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name.
    pub name: String,
    /// Package description.
    pub description: String,
    /// Latest version.
    pub version: String,
}

/// Registry manager for handling multiple registries.
#[derive(Debug)]
pub struct RegistryManager {
    /// Default registry client.
    default_client: RegistryClient,
    /// Scoped registry clients.
    scoped_clients: std::collections::HashMap<String, RegistryClient>,
}

impl RegistryManager {
    /// Creates a new registry manager with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(MultiRegistryConfig::default())
    }

    /// Creates a new registry manager with custom configuration.
    pub fn with_config(config: MultiRegistryConfig) -> Result<Self> {
        let default_client = RegistryClient::with_config(config.default_registry)?;
        let mut scoped_clients = std::collections::HashMap::new();

        for (scope, registry_config) in config.scoped_registries {
            let client = RegistryClient::with_config(registry_config)?;
            scoped_clients.insert(scope, client);
        }

        Ok(Self { default_client, scoped_clients })
    }

    /// Gets the appropriate registry client for a package name.
    pub fn get_client(&self, package_name: &str) -> &RegistryClient {
        if package_name.starts_with('@') {
            if let Some(scope_end) = package_name.find('/') {
                let scope = &package_name[1..scope_end];
                if let Some(client) = self.scoped_clients.get(scope) {
                    return client;
                }
            }
        }
        &self.default_client
    }

    /// Adds a scoped registry.
    pub fn add_scoped_registry(&mut self, scope: String, config: RegistryConfig) -> Result<()> {
        let client = RegistryClient::with_config(config)?;
        self.scoped_clients.insert(scope, client);
        Ok(())
    }

    /// Removes a scoped registry.
    pub fn remove_scoped_registry(&mut self, scope: &str) {
        self.scoped_clients.remove(scope);
    }

    /// Gets the default registry client.
    pub fn default_client(&self) -> &RegistryClient {
        &self.default_client
    }

    /// Gets all scoped registry clients.
    pub fn scoped_clients(&self) -> &std::collections::HashMap<String, RegistryClient> {
        &self.scoped_clients
    }
}

/// Unified dependency resolver that handles all dependency types.
#[derive(Debug)]
pub struct DependencyResolver {
    /// Registry manager for handling multiple registries.
    registry_manager: RegistryManager,
    /// Git resolver for Git dependencies.
    git_resolver: GitResolver,
    /// Path resolver for local dependencies.
    path_resolver: PathResolver,
}

impl DependencyResolver {
    /// Creates a new dependency resolver with default configuration.
    pub fn new() -> Result<Self> {
        let registry_manager = RegistryManager::new()?;
        let git_resolver = GitResolver::with_default_cache();
        let path_resolver = PathResolver::with_current_dir()?;

        Ok(Self { registry_manager, git_resolver, path_resolver })
    }

    /// Creates a new dependency resolver with custom configuration.
    pub fn with_config(config: ResolverConfig) -> Result<Self> {
        let mut multi_config = MultiRegistryConfig::default();
        multi_config.default_registry = config.registry;
        let registry_manager = RegistryManager::with_config(multi_config)?;
        let git_resolver = GitResolver::new(config.cache_dir.join("git"));
        let path_resolver = PathResolver::new(config.root_dir);

        Ok(Self { registry_manager, git_resolver, path_resolver })
    }

    /// Resolves a dependency specification to a concrete dependency.
    ///
    /// # Arguments
    /// * `spec` - The dependency specification.
    ///
    /// # Returns
    /// The resolved dependency with source location.
    pub async fn resolve(&self, spec: &DependencySpec) -> Result<ResolvedDependency> {
        match spec {
            DependencySpec::Npm { name, version } => {
                let client = self.registry_manager.get_client(name);
                client.download_package(name, version).await
            }
            DependencySpec::Git { .. } | DependencySpec::Github { .. } => self.git_resolver.resolve(spec).await,
            DependencySpec::Path { .. } => self.path_resolver.resolve(spec),
        }
    }

    /// Parses a dependency string into a specification.
    ///
    /// Supports:
    /// - NPM: `package@version` or just `package`
    /// - Git: `git+https://...` or `git+ssh://...`
    /// - GitHub: `github:user/repo`
    /// - Path: `file:./path` or `./path`
    pub fn parse_dependency(input: &str) -> Result<DependencySpec> {
        let input = input.trim();

        if input.starts_with("git+") || input.starts_with("git://") {
            return GitResolver::parse_git_url(input);
        }

        if input.starts_with("github:") {
            return GitResolver::parse_git_url(input);
        }

        if PathUtils::is_path_dependency(input) {
            return PathResolver::parse_path_spec(input);
        }

        let (name, version) = if let Some(at_pos) = input.rfind('@') {
            if at_pos > 0 && !input.starts_with('@') {
                let (name, version) = input.split_at(at_pos);
                (name, version[1..].to_string())
            }
            else if input.starts_with('@') {
                if let Some(second_at) = input[1..].find('@') {
                    let name = &input[..second_at + 1];
                    let version = input[second_at + 2..].to_string();
                    (name, version)
                }
                else {
                    (input, "latest".to_string())
                }
            }
            else {
                (input, "latest".to_string())
            }
        }
        else {
            (input, "latest".to_string())
        };

        Ok(DependencySpec::Npm { name: name.to_string(), version })
    }

    /// Gets the registry manager.
    pub fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }

    /// Gets the git resolver.
    pub fn git_resolver(&self) -> &GitResolver {
        &self.git_resolver
    }

    /// Gets the path resolver.
    pub fn path_resolver(&self) -> &PathResolver {
        &self.path_resolver
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default dependency resolver")
    }
}

/// Configuration for the dependency resolver.
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Registry configuration.
    pub registry: RegistryConfig,
    /// Root directory for path resolution.
    pub root_dir: std::path::PathBuf,
    /// Cache directory for downloads.
    pub cache_dir: std::path::PathBuf,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        let root_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cache_dir = root_dir.join(".nargo-cache");

        Self { registry: RegistryConfig { cache_dir: cache_dir.clone(), ..Default::default() }, root_dir, cache_dir }
    }
}
