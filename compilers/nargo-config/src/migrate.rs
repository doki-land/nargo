//! Migration utilities for converting from package.json to Nargo.toml.

use nargo_types::errors::Result;
use serde::{Deserialize, Serialize};
use oak_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    default_edition, Dependency, DependencyDetail, NargoToml, PackageConfig, WorkspaceConfig,
};

/// Represents a package.json file structure.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PackageJson {
    /// Package name.
    pub name: Option<String>,
    /// Package version.
    pub version: Option<String>,
    /// Package description.
    pub description: Option<String>,
    /// Package author.
    pub author: Option<String>,
    /// Package license.
    pub license: Option<String>,
    /// Repository URL or object.
    pub repository: Option<RepositoryField>,
    /// Main entry point.
    pub main: Option<String>,
    /// Module type (commonjs or module).
    #[serde(rename = "type")]
    pub module_type: Option<String>,
    /// Production dependencies.
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// Development dependencies.
    #[serde(default)]
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    /// Peer dependencies.
    #[serde(default)]
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    /// Optional dependencies.
    #[serde(default)]
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
    /// NPM scripts.
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    /// Engine requirements.
    #[serde(default)]
    pub engines: HashMap<String, String>,
    /// Keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Whether this is a private package.
    #[serde(default)]
    pub private: bool,
}

/// Repository field can be a string or an object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RepositoryField {
    /// Simple URL string.
    Url(String),
    /// Detailed repository object.
    Object {
        /// Repository type (e.g., "git").
        #[serde(rename = "type")]
        repo_type: Option<String>,
        /// Repository URL.
        url: Option<String>,
        /// Directory within repository.
        directory: Option<String>,
    },
}

impl RepositoryField {
    /// Extracts the URL from the repository field.
    pub fn url(&self) -> Option<String> {
        match self {
            RepositoryField::Url(url) => Some(url.clone()),
            RepositoryField::Object { url, .. } => url.clone(),
        }
    }
}

/// Migration options.
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    /// Whether to keep the original package.json.
    pub keep_original: bool,
    /// Whether to include peer dependencies.
    pub include_peers: bool,
    /// Whether to include optional dependencies.
    pub include_optional: bool,
    /// Custom output path for Nargo.toml.
    pub output_path: Option<PathBuf>,
}

impl Default for MigrateOptions {
    fn default() -> Self {
        Self {
            keep_original: true,
            include_peers: true,
            include_optional: true,
            output_path: None,
        }
    }
}

/// Result of a migration operation.
#[derive(Debug, Clone)]
pub struct MigrateResult {
    /// Path to the generated Nargo.toml.
    pub output_path: PathBuf,
    /// Number of dependencies migrated.
    pub dependencies_count: usize,
    /// Number of dev dependencies migrated.
    pub dev_dependencies_count: usize,
    /// Number of scripts converted to tasks.
    pub tasks_count: usize,
    /// Warnings encountered during migration.
    pub warnings: Vec<String>,
}

/// Migrator from package.json to Nargo.toml.
#[derive(Debug, Default)]
pub struct Migrator {
    /// Migration options.
    options: MigrateOptions,
}

impl Migrator {
    /// Creates a new migrator with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a migrator with custom options.
    pub fn with_options(options: MigrateOptions) -> Self {
        Self { options }
    }

    /// Migrates a package.json to Nargo.toml.
    pub fn migrate<P: AsRef<Path>>(&self, project_root: P) -> Result<MigrateResult> {
        let root = project_root.as_ref();
        let package_json_path = root.join("package.json");

        if !package_json_path.exists() {
            return Err(nargo_types::errors::Error::external_error(
                "Migrate".to_string(),
                format!("package.json not found in {:?}", root),
                nargo_types::Span::unknown()
            ));
        }

        let content = fs::read_to_string(&package_json_path)?;
        let pkg: PackageJson = oak_json::from_str(&content)?;

        let nargo_toml = self.convert(&pkg)?;

        let output_path = self
            .options
            .output_path
            .clone()
            .unwrap_or_else(|| root.join("Nargo.toml"));

        let toml_content = serde_json::to_string_pretty(&nargo_toml)?;
        fs::write(&output_path, toml_content)?;

        if !self.options.keep_original {
            fs::remove_file(&package_json_path)?;
        }

        let result = MigrateResult {
            output_path,
            dependencies_count: nargo_toml.dependencies.len(),
            dev_dependencies_count: nargo_toml.dev_dependencies.len(),
            tasks_count: 0,
            warnings: Vec::new(),
        };

        Ok(result)
    }

    /// Converts a PackageJson to NargoToml.
    pub fn convert(&self, pkg: &PackageJson) -> Result<NargoToml> {
        let package = PackageConfig {
            name: pkg.name.clone().unwrap_or_else(|| "unnamed".to_string()),
            version: pkg.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
            edition: default_edition(),
            authors: pkg.author.clone().map(|a| vec![a]).unwrap_or_default(),
            description: pkg.description.clone(),
            documentation: None,
            readme: None,
            homepage: None,
            repository: pkg.repository.as_ref().and_then(|r| r.url()),
            license: pkg.license.clone(),
            license_file: None,
            keywords: pkg.keywords.clone(),
            categories: Vec::new(),
            publish: None,
            exclude: Vec::new(),
            include: Vec::new(),
            workspace: None,
            default_run: None,
            metadata: None,
        };

        let dependencies = self.convert_dependencies(&pkg.dependencies, false, false);
        let dev_dependencies = self.convert_dependencies(&pkg.dev_dependencies, true, false);

        let nargo_toml = NargoToml {
            package,
            dependencies,
            dev_dependencies,
            build_dependencies: HashMap::new(),
            workspace: None,
            features: HashMap::new(),
            profile: Default::default(),
            target: HashMap::new(),
            scripts: HashMap::new(),
            registries: HashMap::new(),
            security: Default::default(),
            lint: Default::default(),
            format: Default::default(),
        };

        Ok(nargo_toml)
    }

    fn convert_dependencies(
        &self,
        deps: &HashMap<String, String>,
        _is_dev: bool,
        _is_optional: bool,
    ) -> HashMap<String, Dependency> {
        deps.iter()
            .map(|(name, version)| {
                let dep = if version.starts_with("git+")
                    || version.starts_with("github:")
                    || version.starts_with("git://")
                {
                    self.parse_git_dependency(version)
                } else if version.starts_with("file:")
                    || version.starts_with("./")
                    || version.starts_with("../")
                {
                    self.parse_path_dependency(version)
                } else {
                    Dependency::Version(version.clone())
                };
                (name.clone(), dep)
            })
            .collect()
    }

    fn parse_git_dependency(&self, version: &str) -> Dependency {
        let url = version.strip_prefix("git+").unwrap_or(version);

        if let Some(hash_pos) = url.rfind('#') {
            let (url_part, ref_part) = url.split_at(hash_pos);
            let reference = ref_part.strip_prefix('#').unwrap_or(ref_part);

            Dependency::Detailed(DependencyDetail {
                version: None,
                git: Some(url_part.to_string()),
                branch: if !reference.starts_with('v')
                    && !reference.chars().all(|c| c.is_ascii_hexdigit())
                {
                    Some(reference.to_string())
                } else {
                    None
                },
                tag: if reference.starts_with('v')
                    || !reference.chars().all(|c| c.is_ascii_hexdigit())
                {
                    Some(reference.to_string())
                } else {
                    None
                },
                rev: if reference.chars().all(|c| c.is_ascii_hexdigit()) && reference.len() >= 7 {
                    Some(reference.to_string())
                } else {
                    None
                },
                path: None,
                workspace: None,
                registry: None,
                package_name: None,
                features: Vec::new(),
                optional: false,
                default_features: true,
                target: None,
            })
        } else {
            Dependency::Detailed(DependencyDetail {
                version: None,
                git: Some(url.to_string()),
                branch: None,
                tag: None,
                rev: None,
                path: None,
                workspace: None,
                registry: None,
                package_name: None,
                features: Vec::new(),
                optional: false,
                default_features: true,
                target: None,
            })
        }
    }

    fn parse_path_dependency(&self, version: &str) -> Dependency {
        let path = version.strip_prefix("file:").unwrap_or(version);
        Dependency::Detailed(DependencyDetail {
            version: None,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: Some(PathBuf::from(path)),
            workspace: None,
            registry: None,
            package_name: None,
            features: Vec::new(),
            optional: false,
            default_features: true,
            target: None,
        })
    }

    /// Detects if a project uses workspaces.
    pub fn detect_workspace(&self, _pkg: &PackageJson) -> Option<WorkspaceConfig> {
        None
    }
}
