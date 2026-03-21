//! Dependency graph data structures and algorithms.

use nargo_types::{Error, Result};
use petgraph::{
    Direction,
    algo::{is_cyclic_directed, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents the source of a package.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageSource {
    /// Package from npm registry.
    Registry {
        /// Registry URL (default: https://registry.npmjs.org).
        registry: String,
    },
    /// Package from Git repository.
    Git {
        /// Git repository URL.
        url: String,
        /// Git reference (branch, tag, or commit).
        reference: Option<String>,
    },
    /// Package from local path.
    Path {
        /// Local file system path.
        path: String,
    },
    /// Package from GitHub shorthand.
    Github {
        /// GitHub repository (owner/repo).
        repo: String,
        /// Git reference (branch, tag, or commit).
        reference: Option<String>,
    },
    /// Package from workspace.
    Workspace,
}

impl Default for PackageSource {
    fn default() -> Self {
        PackageSource::Registry { registry: "https://registry.npmjs.org".to_string() }
    }
}

/// Represents a node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Package name.
    pub name: String,
    /// Resolved version.
    pub version: String,
    /// Package source.
    pub source: PackageSource,
    /// Whether this is a dev dependency.
    pub is_dev: bool,
    /// Whether this is an optional dependency.
    pub is_optional: bool,
    /// Features enabled for this package.
    pub features: Vec<String>,
}

impl DependencyNode {
    /// Creates a new dependency node.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), source: PackageSource::default(), is_dev: false, is_optional: false, features: Vec::new() }
    }

    /// Sets the package source.
    pub fn with_source(mut self, source: PackageSource) -> Self {
        self.source = source;
        self
    }

    /// Marks this as a dev dependency.
    pub fn as_dev(mut self, is_dev: bool) -> Self {
        self.is_dev = is_dev;
        self
    }

    /// Marks this as a dev dependency (sets to true).
    pub fn set_dev(mut self) -> Self {
        self.is_dev = true;
        self
    }

    /// Marks this as an optional dependency.
    pub fn as_optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// Sets the enabled features.
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    /// Returns a unique key for this node.
    pub fn key(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// Represents an edge in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Version constraint from the dependent.
    pub constraint: String,
    /// Whether this is a peer dependency relationship.
    pub is_peer: bool,
}

impl DependencyEdge {
    /// Creates a new dependency edge.
    pub fn new(constraint: impl Into<String>) -> Self {
        Self { constraint: constraint.into(), is_peer: false }
    }

    /// Marks this as a peer dependency edge.
    pub fn as_peer(mut self) -> Self {
        self.is_peer = true;
        self
    }
}

/// The dependency graph structure.
#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    /// The underlying directed graph.
    graph: DiGraph<DependencyNode, DependencyEdge>,
    /// Map from package name to node index.
    name_to_index: HashMap<String, NodeIndex>,
    /// Map from (name, version) to node index.
    key_to_index: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    pub fn new() -> Self {
        Self { graph: DiGraph::new(), name_to_index: HashMap::new(), key_to_index: HashMap::new() }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: DependencyNode) -> NodeIndex {
        let key = node.key();
        let name = node.name.clone();
        let idx = self.graph.add_node(node);
        self.name_to_index.insert(name, idx);
        self.key_to_index.insert(key, idx);
        idx
    }

    /// Adds an edge between two nodes.
    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, edge: DependencyEdge) {
        self.graph.add_edge(from, to, edge);
    }

    /// Gets a node by name.
    pub fn get_node_by_name(&self, name: &str) -> Option<&DependencyNode> {
        self.name_to_index.get(name).map(|idx| &self.graph[*idx])
    }

    /// Gets a node by key (name@version).
    pub fn get_node_by_key(&self, key: &str) -> Option<&DependencyNode> {
        self.key_to_index.get(key).map(|idx| &self.graph[*idx])
    }

    /// Gets a node by its index.
    pub fn get_node(&self, idx: NodeIndex) -> Option<&DependencyNode> {
        self.graph.node_weight(idx)
    }

    /// Returns all nodes in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = &DependencyNode> {
        self.graph.node_weights()
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Checks if the graph has cycles.
    pub fn has_cycle(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Detects cycles and returns the cycle path if found.
    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        if !self.has_cycle() {
            return None;
        }

        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut result = None;

        for node_idx in self.graph.node_indices() {
            if result.is_some() {
                break;
            }
            self.dfs_cycle(node_idx, &mut visited, &mut path, &mut result);
        }

        result
    }

    fn dfs_cycle(&self, node: NodeIndex, visited: &mut HashSet<NodeIndex>, path: &mut Vec<NodeIndex>, result: &mut Option<Vec<String>>) {
        if result.is_some() {
            return;
        }

        if path.contains(&node) {
            let cycle_start = path.iter().position(|&n| n == node).unwrap();
            let cycle: Vec<String> = path[cycle_start..].iter().chain(std::iter::once(&node)).map(|&idx| self.graph[idx].key()).collect();
            *result = Some(cycle);
            return;
        }

        if visited.contains(&node) {
            return;
        }

        visited.insert(node);
        path.push(node);

        for neighbor in self.graph.neighbors_directed(node, Direction::Outgoing) {
            self.dfs_cycle(neighbor, visited, path, result);
        }

        path.pop();
    }

    /// Performs topological sort and returns nodes in order.
    pub fn topological_sort(&self) -> Result<Vec<DependencyNode>> {
        if self.has_cycle() {
            let cycle = self.detect_cycle();
            let msg = if let Some(cycle) = cycle { format!("Circular dependency detected: {}", cycle.join(" -> ")) } else { "Circular dependency detected".to_string() };
            return Err(Error::external_error("resolver".to_string(), msg, nargo_types::Span::unknown()));
        }

        let sorted: Vec<NodeIndex> = toposort(&self.graph, None).map_err(|e| Error::external_error("resolver".to_string(), format!("Topological sort failed: {:?}", e), nargo_types::Span::unknown()))?;

        Ok(sorted.into_iter().filter_map(|idx| self.graph.node_weight(idx).cloned()).collect())
    }

    /// Returns all dependencies of a node.
    pub fn dependencies_of(&self, node_idx: NodeIndex) -> Vec<(&DependencyNode, &DependencyEdge)> {
        self.graph
            .edges_directed(node_idx, Direction::Outgoing)
            .filter_map(|edge| {
                let target = edge.target();
                self.graph.node_weight(target).map(|node| (node, edge.weight()))
            })
            .collect()
    }

    /// Returns all dependents of a node (reverse dependencies).
    pub fn dependents_of(&self, node_idx: NodeIndex) -> Vec<(&DependencyNode, &DependencyEdge)> {
        self.graph
            .edges_directed(node_idx, Direction::Incoming)
            .filter_map(|edge| {
                let source = edge.source();
                self.graph.node_weight(source).map(|node| (node, edge.weight()))
            })
            .collect()
    }

    /// Returns the node index for a given name.
    pub fn node_index(&self, name: &str) -> Option<NodeIndex> {
        self.name_to_index.get(name).copied()
    }
}
