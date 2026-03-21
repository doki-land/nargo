//! Dependency conflict detection and resolution.

use nargo_types::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of dependency conflicts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Version conflict: multiple versions of the same package required.
    VersionConflict,
    /// Peer dependency conflict: peer dependency not satisfied.
    PeerDependencyConflict,
    /// Missing dependency: required dependency not found.
    MissingDependency,
    /// Circular dependency detected.
    CircularDependency,
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::VersionConflict => write!(f, "Version Conflict"),
            ConflictType::PeerDependencyConflict => write!(f, "Peer Dependency Conflict"),
            ConflictType::MissingDependency => write!(f, "Missing Dependency"),
            ConflictType::CircularDependency => write!(f, "Circular Dependency"),
        }
    }
}

/// Represents a dependency conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Type of conflict.
    pub conflict_type: ConflictType,
    /// Package name involved in the conflict.
    pub package: String,
    /// Description of the conflict.
    pub description: String,
    /// Conflicting versions (if applicable).
    pub versions: Vec<String>,
    /// Packages that require conflicting versions.
    pub required_by: Vec<String>,
}

impl Conflict {
    /// Creates a new conflict.
    pub fn new(conflict_type: ConflictType, package: impl Into<String>, description: impl Into<String>) -> Self {
        Self { conflict_type, package: package.into(), description: description.into(), versions: Vec::new(), required_by: Vec::new() }
    }

    /// Adds a conflicting version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.versions.push(version.into());
        self
    }

    /// Adds a package that requires this dependency.
    pub fn required_by(mut self, package: impl Into<String>) -> Self {
        self.required_by.push(package.into());
        self
    }
}

/// Represents a solution to a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSolution {
    /// The conflict being solved.
    pub conflict: Conflict,
    /// The recommended version to use.
    pub recommended_version: Option<String>,
    /// Description of the solution.
    pub description: String,
    /// Whether the solution is automatic or requires user action.
    pub is_automatic: bool,
}

impl ConflictSolution {
    /// Creates a new conflict solution.
    pub fn new(conflict: Conflict, description: impl Into<String>) -> Self {
        Self { conflict, recommended_version: None, description: description.into(), is_automatic: false }
    }

    /// Sets the recommended version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.recommended_version = Some(version.into());
        self
    }

    /// Marks this solution as automatic.
    pub fn automatic(mut self) -> Self {
        self.is_automatic = true;
        self
    }
}

/// Conflict detector for dependency resolution.
#[derive(Debug, Default)]
pub struct ConflictDetector {
    /// Detected conflicts.
    conflicts: Vec<Conflict>,
}

impl ConflictDetector {
    /// Creates a new conflict detector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Detects version conflicts between dependencies.
    pub fn detect_version_conflicts(&mut self, dependencies: &HashMap<String, Vec<(String, String)>>) -> &Vec<Conflict> {
        for (package, version_sources) in dependencies {
            if version_sources.len() > 1 {
                let versions: Vec<String> = version_sources.iter().map(|(v, _)| v.clone()).collect();
                let unique_versions: Vec<String> = versions.iter().filter(|v| !versions.iter().all(|u| u == *v)).cloned().collect();

                if unique_versions.len() > 1 {
                    let sources: Vec<String> = version_sources.iter().map(|(_, s)| s.clone()).collect();

                    let conflict = Conflict::new(ConflictType::VersionConflict, package.clone(), format!("Multiple incompatible versions of '{}' required: {}", package, versions.join(", "))).with_version(versions.join(", "));

                    let conflict = sources.iter().fold(conflict, |c, s| c.required_by(s));
                    self.conflicts.push(conflict);
                }
            }
        }
        &self.conflicts
    }

    /// Detects peer dependency conflicts.
    pub fn detect_peer_conflicts(&mut self, peer_deps: &HashMap<String, (String, Option<String>)>, resolved: &HashMap<String, String>) -> &Vec<Conflict> {
        for (package, (required, peer_of)) in peer_deps {
            if let Some(resolved_version) = resolved.get(package) {
                if resolved_version != required {
                    let conflict = Conflict::new(ConflictType::PeerDependencyConflict, package.clone(), format!("Peer dependency '{}' requires version {}, but {} is installed", package, required, resolved_version)).with_version(required.clone()).with_version(resolved_version.clone());

                    let conflict = if let Some(parent) = peer_of { conflict.required_by(parent) } else { conflict };
                    self.conflicts.push(conflict);
                }
            }
            else {
                let conflict = Conflict::new(ConflictType::MissingDependency, package.clone(), format!("Peer dependency '{}' is required but not installed", package)).with_version(required.clone());

                let conflict = if let Some(parent) = peer_of { conflict.required_by(parent) } else { conflict };
                self.conflicts.push(conflict);
            }
        }
        &self.conflicts
    }

    /// Returns all detected conflicts.
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Checks if there are any conflicts.
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Clears all detected conflicts.
    pub fn clear(&mut self) {
        self.conflicts.clear();
    }

    /// Attempts to find solutions for all conflicts.
    pub fn find_solutions(&self) -> Vec<ConflictSolution> {
        self.conflicts.iter().map(|conflict| self.solve_conflict(conflict)).collect()
    }

    fn solve_conflict(&self, conflict: &Conflict) -> ConflictSolution {
        match conflict.conflict_type {
            ConflictType::VersionConflict => {
                let highest = conflict
                    .versions
                    .iter()
                    .max_by(|a, b| {
                        let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
                        let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
                        a_parts.cmp(&b_parts)
                    })
                    .cloned();

                ConflictSolution::new(conflict.clone(), "Use the highest compatible version".to_string()).with_version(highest.unwrap_or_default()).automatic()
            }
            ConflictType::PeerDependencyConflict => ConflictSolution::new(conflict.clone(), "Install the required peer dependency version".to_string()).with_version(conflict.versions.first().cloned().unwrap_or_default()),
            ConflictType::MissingDependency => ConflictSolution::new(conflict.clone(), "Install the missing peer dependency".to_string()).with_version(conflict.versions.first().cloned().unwrap_or_default()),
            ConflictType::CircularDependency => ConflictSolution::new(conflict.clone(), "Refactor to remove the circular dependency".to_string()),
        }
    }
}
