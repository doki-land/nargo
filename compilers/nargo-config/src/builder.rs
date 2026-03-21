use nargo_types::errors::Result;
use std::collections::HashMap;

use crate::types::{
    default_edition, Dependency, DependencyDetail, FormatConfig, LintConfig, PackageConfig,
    ProfileConfig, RegistryEntry, ScriptConfig, SecurityConfig, WorkspaceConfig,
};
use crate::NargoToml;

/// Builder for creating NargoToml configurations programmatically.
#[derive(Debug, Default)]
pub struct NargoTomlBuilder {
    package: Option<PackageConfig>,
    dependencies: HashMap<String, Dependency>,
    dev_dependencies: HashMap<String, Dependency>,
    build_dependencies: HashMap<String, Dependency>,
    workspace: Option<WorkspaceConfig>,
    features: HashMap<String, Vec<String>>,
    profile: ProfileConfig,
    scripts: HashMap<String, ScriptConfig>,
    registries: HashMap<String, RegistryEntry>,
    security: SecurityConfig,
    lint: LintConfig,
    format: FormatConfig,
}

impl NargoTomlBuilder {
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the package configuration.
    pub fn package(mut self, package: PackageConfig) -> Self {
        self.package = Some(package);
        self
    }

    /// Sets the package name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let pkg = self.package.get_or_insert_with(|| PackageConfig {
            name: String::new(),
            version: "0.1.0".to_string(),
            edition: default_edition(),
            authors: Vec::new(),
            description: None,
            documentation: None,
            readme: None,
            homepage: None,
            repository: None,
            license: None,
            license_file: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            publish: None,
            exclude: Vec::new(),
            include: Vec::new(),
            workspace: None,
            default_run: None,
            metadata: None,
        });
        pkg.name = name.into();
        self
    }

    /// Sets the package version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        let pkg = self.package.get_or_insert_with(|| PackageConfig {
            name: String::new(),
            version: String::new(),
            edition: default_edition(),
            authors: Vec::new(),
            description: None,
            documentation: None,
            readme: None,
            homepage: None,
            repository: None,
            license: None,
            license_file: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            publish: None,
            exclude: Vec::new(),
            include: Vec::new(),
            workspace: None,
            default_run: None,
            metadata: None,
        });
        pkg.version = version.into();
        self
    }

    /// Sets the package description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        if let Some(pkg) = &mut self.package {
            pkg.description = Some(description.into());
        }
        self
    }

    /// Sets the package license.
    pub fn license(mut self, license: impl Into<String>) -> Self {
        if let Some(pkg) = &mut self.package {
            pkg.license = Some(license.into());
        }
        self
    }

    /// Adds an author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        if let Some(pkg) = &mut self.package {
            pkg.authors.push(author.into());
        }
        self
    }

    /// Adds a dependency.
    pub fn dependency(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies
            .insert(name.into(), Dependency::Version(version.into()));
        self
    }

    /// Adds a detailed dependency.
    pub fn dependency_detail(mut self, name: impl Into<String>, detail: DependencyDetail) -> Self {
        self.dependencies
            .insert(name.into(), Dependency::Detailed(detail));
        self
    }

    /// Adds a development dependency.
    pub fn dev_dependency(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.dev_dependencies
            .insert(name.into(), Dependency::Version(version.into()));
        self
    }

    /// Adds a build dependency.
    pub fn build_dependency(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.build_dependencies
            .insert(name.into(), Dependency::Version(version.into()));
        self
    }

    /// Sets the workspace configuration.
    pub fn workspace(mut self, workspace: WorkspaceConfig) -> Self {
        self.workspace = Some(workspace);
        self
    }

    /// Adds a feature definition.
    pub fn feature(mut self, name: impl Into<String>, deps: Vec<String>) -> Self {
        self.features.insert(name.into(), deps);
        self
    }

    /// Sets the profile configuration.
    pub fn profile(mut self, profile: ProfileConfig) -> Self {
        self.profile = profile;
        self
    }

    /// Adds a script.
    pub fn script(mut self, name: impl Into<String>, command: impl Into<String>) -> Self {
        self.scripts
            .insert(name.into(), ScriptConfig::Command(command.into()));
        self
    }

    /// Adds a registry.
    pub fn registry(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.registries.insert(
            name.into(),
            RegistryEntry {
                url: url.into(),
                auth_token: None,
            },
        );
        self
    }

    /// Sets the security configuration.
    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.security = security;
        self
    }

    /// Sets the lint configuration.
    pub fn lint(mut self, lint: LintConfig) -> Self {
        self.lint = lint;
        self
    }

    /// Sets the format configuration.
    pub fn format(mut self, format: FormatConfig) -> Self {
        self.format = format;
        self
    }

    /// Builds the final NargoToml configuration.
    pub fn build(self) -> Result<NargoToml> {
        let package = self
            .package
            .ok_or_else(|| nargo_types::errors::Error::external_error(
                "NargoTomlBuilder".to_string(),
                "Package configuration is required".to_string(),
                nargo_types::Span::unknown()
            ))?;

        let config = NargoToml {
            package,
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
            build_dependencies: self.build_dependencies,
            workspace: self.workspace,
            features: self.features,
            profile: self.profile,
            target: HashMap::new(),
            scripts: self.scripts,
            registries: self.registries,
            security: self.security,
            lint: self.lint,
            format: self.format,
        };

        config.validate()?;
        Ok(config)
    }
}
