//! Dependency resolver for Nargo package manager.
//!
//! This crate provides dependency resolution, conflict detection,
//! and topological sorting for package dependencies.

#![warn(missing_docs)]

pub mod conflict;
pub mod graph;
pub mod resolver;

pub use conflict::{Conflict, ConflictDetector, ConflictSolution, ConflictType};
pub use graph::{DependencyEdge, DependencyGraph, DependencyNode, PackageSource};
pub use resolver::{ResolveOptions, ResolveResult, Resolver};

use nargo_types::Result;
use std::path::Path;

/// Print the dependency tree.
pub fn print_tree(_root: &Path) -> Result<()> {
    println!("Dependency tree:");
    Ok(())
}
