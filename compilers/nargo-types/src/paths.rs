//! Canonical filesystem paths for Nargo tooling.

use std::path::{Path, PathBuf};

/// Relative cache directory under the workspace (or project) root.
///
/// There is exactly one cache for the whole workspace — never per-package.
pub const CACHE_DIR_REL: &str = ".cache/nargo";

/// Resolve the single Nargo cache directory for a workspace/project root.
///
/// Example: `/repo/.cache/nargo` even when packages live under `runtimes/*`.
pub fn cache_dir(workspace_root: impl AsRef<Path>) -> PathBuf {
    workspace_root.as_ref().join(".cache").join("nargo")
}
