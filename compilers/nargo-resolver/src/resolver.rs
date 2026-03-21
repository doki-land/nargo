//! Main dependency resolver implementation.

use nargo_types::{Error, Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{
    conflict::{Conflict, ConflictDetector, ConflictSolution},
    graph::{DependencyEdge, DependencyGraph, DependencyNode, PackageSource},
};

/// Options for dependency resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveOptions {
    /// Whether to include dev dependencies.
    pub include_dev: bool,
    /// Whether to include optional dependencies.
    pub include_optional: bool,
    /// Whether to allow prerelease versions.
    pub allow_prerelease: bool,
    /// Maximum depth for dependency resolution.
    pub max_depth: usize,
    /// Registry URL to use.
    pub registry: String,
    /// Number of parallel resolution tasks.
    pub parallel_jobs: usize,
    /// Whether to resolve workspace dependencies.
    pub resolve_workspace: bool,
    /// Whether to use lock file for resolution.
    pub use_lock_file: bool,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self { include_dev: true, include_optional: true, allow_prerelease: false, max_depth: 100, registry: "https://registry.npmjs.org".to_string(), parallel_jobs: num_cpus::get(), resolve_workspace: true, use_lock_file: true }
    }
}

/// Result of dependency resolution.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    /// The resolved dependency graph.
    pub graph: DependencyGraph,
    /// Detected conflicts.
    pub conflicts: Vec<Conflict>,
    /// Suggested solutions for conflicts.
    pub solutions: Vec<ConflictSolution>,
    /// Resolution statistics.
    pub stats: ResolveStats,
}

/// Statistics about the resolution process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolveStats {
    /// Total number of packages resolved.
    pub total_packages: usize,
    /// Number of production dependencies.
    pub production_deps: usize,
    /// Number of dev dependencies.
    pub dev_deps: usize,
    /// Number of peer dependencies.
    pub peer_deps: usize,
    /// Number of optional dependencies.
    pub optional_deps: usize,
    /// Number of conflicts detected.
    pub conflicts: usize,
    /// Number of packages from registry.
    pub registry_deps: usize,
    /// Number of git dependencies.
    pub git_deps: usize,
    /// Number of path dependencies.
    pub path_deps: usize,
    /// Number of workspace dependencies.
    pub workspace_deps: usize,
    /// Resolution time in milliseconds.
    pub resolution_time_ms: u64,
}

/// Main dependency resolver.
#[derive(Debug)]
pub struct Resolver {
    /// Resolution options.
    options: ResolveOptions,
    /// Conflict detector.
    conflict_detector: ConflictDetector,
    /// Cache of resolved packages.
    resolved_cache: HashMap<String, DependencyNode>,
    /// Workspace packages cache.
    workspace_packages: HashMap<String, String>,
}

impl Resolver {
    /// Creates a new resolver with default options.
    pub fn new() -> Self {
        Self::with_options(ResolveOptions::default())
    }

    /// Creates a new resolver with custom options.
    pub fn with_options(options: ResolveOptions) -> Self {
        Self { options, conflict_detector: ConflictDetector::new(), resolved_cache: HashMap::new(), workspace_packages: HashMap::new() }
    }

    /// Sets the workspace packages for resolution.
    pub fn with_workspace_packages(mut self, packages: HashMap<String, String>) -> Self {
        self.workspace_packages = packages;
        self
    }

    /// Resolves dependencies from a Nargo.toml configuration.
    pub async fn resolve(&mut self, dependencies: &HashMap<String, nargo_config::Dependency>, dev_dependencies: &HashMap<String, nargo_config::Dependency>) -> Result<ResolveResult> {
        let start = std::time::Instant::now();
        let mut graph = DependencyGraph::new();

        info!("Starting dependency resolution with {} parallel jobs...", self.options.parallel_jobs);

        let mut version_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut peer_deps: HashMap<String, (String, Option<String>)> = HashMap::new();
        let mut resolved_versions: HashMap<String, String> = HashMap::new();

        self.resolve_deps_recursive(dependencies, false, &mut graph, &mut version_map, &mut resolved_versions, 0).await?;

        if self.options.include_dev {
            self.resolve_deps_recursive(dev_dependencies, true, &mut graph, &mut version_map, &mut resolved_versions, 0).await?;
        }

        self.conflict_detector.detect_version_conflicts(&version_map);
        self.conflict_detector.detect_peer_conflicts(&peer_deps, &resolved_versions);

        let conflicts: Vec<Conflict> = self.conflict_detector.conflicts().to_vec();
        let solutions = self.conflict_detector.find_solutions();

        let stats = self.calculate_stats(&graph, &conflicts, start.elapsed().as_millis() as u64);

        info!("Resolution complete: {} packages, {} conflicts in {}ms", stats.total_packages, stats.conflicts, stats.resolution_time_ms);

        Ok(ResolveResult { graph, conflicts, solutions, stats })
    }

    /// Resolves a single dependency.
    pub async fn resolve_single(&mut self, name: &str, dependency: &nargo_config::Dependency) -> Result<DependencyNode> {
        let (version, source) = self.parse_dependency(dependency)?;

        let mut node = DependencyNode::new(name, &version).with_source(source);

        if let nargo_config::Dependency::Detailed(detail) = dependency {
            if !detail.features.is_empty() {
                node = node.with_features(detail.features.clone());
            }
            if detail.optional {
                node = node.as_optional();
            }
        }

        Ok(node)
    }

    /// Performs topological sort on resolved dependencies.
    pub fn topological_sort(&self, graph: &DependencyGraph) -> Result<Vec<DependencyNode>> {
        graph.topological_sort()
    }

    /// Detects circular dependencies.
    pub fn detect_cycles(&self, graph: &DependencyGraph) -> Option<Vec<String>> {
        graph.detect_cycle()
    }

    #[allow(clippy::too_many_arguments)]
    async fn resolve_deps_recursive(&mut self, deps: &HashMap<String, nargo_config::Dependency>, is_dev: bool, graph: &mut DependencyGraph, version_map: &mut HashMap<String, Vec<(String, String)>>, resolved_versions: &mut HashMap<String, String>, depth: usize) -> Result<()> {
        if depth >= self.options.max_depth {
            warn!("Max depth {} reached, stopping resolution", depth);
            return Ok(());
        }

        for (name, dep) in deps {
            if self.resolved_cache.contains_key(name) {
                debug!("Using cached resolution for {}", name);
                continue;
            }

            let (version, source) = self.parse_dependency(dep)?;

            if self.options.resolve_workspace && self.is_workspace_dependency(dep) {
                if let Some(ws_version) = self.workspace_packages.get(name) {
                    version_map.entry(name.clone()).or_default().push((ws_version.clone(), "workspace".to_string()));
                    resolved_versions.insert(name.clone(), ws_version.clone());
                    continue;
                }
            }

            version_map.entry(name.clone()).or_default().push((version.clone(), "root".to_string()));

            resolved_versions.insert(name.clone(), version.clone());

            let mut node = DependencyNode::new(name, &version).with_source(source).as_dev(is_dev);

            if let nargo_config::Dependency::Detailed(detail) = dep {
                if !detail.features.is_empty() {
                    node = node.with_features(detail.features.clone());
                }
                if detail.optional {
                    node = node.as_optional();
                }
            }

            let node_idx = graph.add_node(node.clone());
            self.resolved_cache.insert(name.clone(), node);

            if depth + 1 < self.options.max_depth {
                self.resolve_transitive_deps(name, &version, node_idx, is_dev, graph, version_map, resolved_versions, depth + 1).await?;
            }
        }

        Ok(())
    }

    async fn resolve_transitive_deps(&mut self, _parent_name: &str, _parent_version: &str, _parent_idx: petgraph::graph::NodeIndex, _is_dev: bool, _graph: &mut DependencyGraph, _version_map: &mut HashMap<String, Vec<(String, String)>>, _resolved_versions: &mut HashMap<String, String>, _depth: usize) -> Result<()> {
        Ok(())
    }

    fn parse_dependency(&self, dep: &nargo_config::Dependency) -> Result<(String, PackageSource)> {
        match dep {
            nargo_config::Dependency::Version(v) => {
                if v.starts_with("workspace:") {
                    Ok((v.clone(), PackageSource::Workspace))
                }
                else if v.starts_with("file:") || v.starts_with("./") || v.starts_with("../") {
                    let path = v.strip_prefix("file:").unwrap_or(v);
                    Ok(("path".to_string(), PackageSource::Path { path: path.to_string() }))
                }
                else if v.starts_with("git+") || v.starts_with("git://") {
                    let url = v.strip_prefix("git+").unwrap_or(v);
                    Ok(("git".to_string(), PackageSource::Git { url: url.to_string(), reference: None }))
                }
                else if v.starts_with("github:") {
                    let repo = v.strip_prefix("github:").unwrap_or(v);
                    Ok(("github".to_string(), PackageSource::Github { repo: repo.to_string(), reference: None }))
                }
                else {
                    Ok((v.clone(), PackageSource::Registry { registry: self.options.registry.clone() }))
                }
            }
            nargo_config::Dependency::Detailed(detail) => {
                if detail.workspace.unwrap_or(false) {
                    Ok(("workspace:*".to_string(), PackageSource::Workspace))
                }
                else if let Some(ref version) = detail.version {
                    Ok((version.clone(), PackageSource::Registry { registry: detail.registry.clone().unwrap_or_else(|| self.options.registry.clone()) }))
                }
                else if let Some(ref git) = detail.git {
                    let reference = detail.tag.clone().or_else(|| detail.branch.clone()).or_else(|| detail.rev.clone());
                    Ok(("git".to_string(), PackageSource::Git { url: git.clone(), reference }))
                }
                else if let Some(ref path) = detail.path {
                    Ok(("path".to_string(), PackageSource::Path { path: path.to_string_lossy().to_string() }))
                }
                else {
                    Err(Error::external_error("resolver".to_string(), "Invalid dependency specification".to_string(), Span::unknown()))
                }
            }
        }
    }

    fn is_workspace_dependency(&self, dep: &nargo_config::Dependency) -> bool {
        match dep {
            nargo_config::Dependency::Version(v) => v.starts_with("workspace:"),
            nargo_config::Dependency::Detailed(d) => d.workspace.unwrap_or(false),
        }
    }

    fn calculate_stats(&self, graph: &DependencyGraph, conflicts: &[Conflict], resolution_time_ms: u64) -> ResolveStats {
        let nodes: Vec<&DependencyNode> = graph.nodes().collect();

        ResolveStats { total_packages: nodes.len(), production_deps: nodes.iter().filter(|n| !n.is_dev && !n.is_optional).count(), dev_deps: nodes.iter().filter(|n| n.is_dev).count(), peer_deps: 0, optional_deps: nodes.iter().filter(|n| n.is_optional).count(), conflicts: conflicts.len(), registry_deps: nodes.iter().filter(|n| matches!(n.source, PackageSource::Registry { .. })).count(), git_deps: nodes.iter().filter(|n| matches!(n.source, PackageSource::Git { .. })).count(), path_deps: nodes.iter().filter(|n| matches!(n.source, PackageSource::Path { .. })).count(), workspace_deps: nodes.iter().filter(|n| matches!(n.source, PackageSource::Workspace)).count(), resolution_time_ms }
    }

    /// Clears the resolver cache.
    pub fn clear_cache(&mut self) {
        self.resolved_cache.clear();
        self.conflict_detector.clear();
    }

    /// Returns the resolved cache.
    pub fn cache(&self) -> &HashMap<String, DependencyNode> {
        &self.resolved_cache
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
