//! Lock file verification implementation.

use nargo_types::Result;
use std::{collections::HashSet, path::Path};
use tracing::{info, warn};

use crate::lock::LockFile;

/// Result of lock file verification.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// Whether verification passed.
    pub is_valid: bool,
    /// Missing packages.
    pub missing: Vec<String>,
    /// Packages with integrity mismatches.
    pub integrity_errors: Vec<String>,
    /// Packages with version conflicts.
    pub version_conflicts: Vec<String>,
}

impl VerifyResult {
    /// Creates a new verification result.
    pub fn new() -> Self {
        Self { is_valid: true, missing: Vec::new(), integrity_errors: Vec::new(), version_conflicts: Vec::new() }
    }

    /// Adds a missing package.
    pub fn add_missing(&mut self, package: String) {
        self.missing.push(package);
        self.is_valid = false;
    }

    /// Adds an integrity error.
    pub fn add_integrity_error(&mut self, package: String) {
        self.integrity_errors.push(package);
        self.is_valid = false;
    }

    /// Adds a version conflict.
    pub fn add_version_conflict(&mut self, package: String) {
        self.version_conflicts.push(package);
        self.is_valid = false;
    }

    /// Returns a summary of the verification result.
    pub fn summary(&self) -> String {
        if self.is_valid {
            "Lock file is valid".to_string()
        }
        else {
            let mut issues = Vec::new();
            if !self.missing.is_empty() {
                issues.push(format!("{} missing packages", self.missing.len()));
            }
            if !self.integrity_errors.is_empty() {
                issues.push(format!("{} integrity errors", self.integrity_errors.len()));
            }
            if !self.version_conflicts.is_empty() {
                issues.push(format!("{} version conflicts", self.version_conflicts.len()));
            }
            format!("Lock file has issues: {}", issues.join(", "))
        }
    }
}

impl Default for VerifyResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock file verifier.
#[derive(Debug)]
pub struct Verifier {
    /// Whether to verify integrity hashes.
    pub verify_integrity: bool,
    /// Whether to check for missing dependencies.
    pub check_missing: bool,
}

impl Default for Verifier {
    fn default() -> Self {
        Self { verify_integrity: true, check_missing: true }
    }
}

impl Verifier {
    /// Creates a new verifier with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifies a lock file against the current project state.
    pub fn verify(&self, lock: &LockFile, project_root: &Path) -> Result<VerifyResult> {
        let mut result = VerifyResult::new();

        info!("Verifying lock file for project at {:?}", project_root);

        if self.check_missing {
            self.check_missing_packages(lock, &mut result);
        }

        if self.verify_integrity {
            self.check_integrity(lock, &mut result);
        }

        self.check_dependency_graph(lock, &mut result);

        if result.is_valid {
            info!("Lock file verification passed");
        }
        else {
            warn!("Lock file verification failed: {}", result.summary());
        }

        Ok(result)
    }

    fn check_missing_packages(&self, lock: &LockFile, result: &mut VerifyResult) {
        for entry in lock.entries() {
            if entry.source.is_empty() {
                result.add_missing(format!("{} (no source)", entry.key()));
            }
            if entry.integrity.is_empty() {
                result.add_missing(format!("{} (no integrity)", entry.key()));
            }
        }
    }

    fn check_integrity(&self, lock: &LockFile, result: &mut VerifyResult) {
        for entry in lock.entries() {
            if !entry.integrity.is_empty() {
                if !entry.integrity.starts_with("sha512-") && !entry.integrity.starts_with("sha256-") {
                    result.add_integrity_error(format!("{} (invalid integrity format)", entry.key()));
                }
            }
        }
    }

    fn check_dependency_graph(&self, lock: &LockFile, result: &mut VerifyResult) {
        let mut visited = HashSet::new();
        for entry in lock.entries() {
            for dep in &entry.dependencies {
                if !lock.packages.contains_key(dep) {
                    result.add_missing(format!("{} -> {} (dependency not in lock file)", entry.key(), dep));
                }
            }

            if visited.contains(&entry.name) {
                result.add_version_conflict(format!("{} (multiple versions of same package)", entry.name));
            }
            visited.insert(entry.name.clone());
        }
    }

    /// Compares two lock files and returns the differences.
    pub fn diff(old: &LockFile, new: &LockFile) -> LockDiff {
        let mut diff = LockDiff::new();

        for (key, entry) in &new.packages {
            if !old.packages.contains_key(key) {
                diff.added.push(entry.key());
            }
        }

        for (key, entry) in &old.packages {
            if !new.packages.contains_key(key) {
                diff.removed.push(entry.key());
            }
            else if old.packages.get(key).unwrap().integrity != new.packages.get(key).unwrap().integrity {
                diff.changed.push(entry.key());
            }
        }

        diff
    }
}

/// Represents the difference between two lock files.
#[derive(Debug, Clone, Default)]
pub struct LockDiff {
    /// Packages added in the new lock file.
    pub added: Vec<String>,
    /// Packages removed from the old lock file.
    pub removed: Vec<String>,
    /// Packages with changed integrity.
    pub changed: Vec<String>,
}

impl LockDiff {
    /// Creates a new empty diff.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if there are any changes.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Returns a summary of the diff.
    pub fn summary(&self) -> String {
        if self.is_empty() {
            "No changes".to_string()
        }
        else {
            let mut parts = Vec::new();
            if !self.added.is_empty() {
                parts.push(format!("{} added", self.added.len()));
            }
            if !self.removed.is_empty() {
                parts.push(format!("{} removed", self.removed.len()));
            }
            if !self.changed.is_empty() {
                parts.push(format!("{} changed", self.changed.len()));
            }
            parts.join(", ")
        }
    }
}
