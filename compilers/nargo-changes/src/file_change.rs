#![warn(missing_docs)]

use filetime::FileTime;
use nargo_types::{Error, Result, Span};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::types::FileChangeType;

/// File change information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// The path to the file.
    pub path: PathBuf,
    /// The type of change.
    pub r#type: FileChangeType,
    /// The previous hash (if applicable).
    pub old_hash: Option<String>,
    /// The new hash (if applicable).
    pub new_hash: Option<String>,
    /// The modification time (seconds since epoch).
    pub modified_time: Option<u64>,
}

/// File change detector.
pub struct FileChangeDetector {
    /// The base directory to scan for changes.
    pub base_dir: PathBuf,
    /// The list of file patterns to include.
    pub include_patterns: Vec<String>,
    /// The list of file patterns to exclude.
    pub exclude_patterns: Vec<String>,
}

impl FileChangeDetector {
    /// Creates a new file change detector.
    pub fn new(base_dir: &Path) -> Self {
        Self { base_dir: base_dir.to_path_buf(), include_patterns: vec!["**/*".to_string()], exclude_patterns: vec!["target/**".to_string(), ".git/**".to_string(), "node_modules/**".to_string()] }
    }

    /// Sets the include patterns.
    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    /// Sets the exclude patterns.
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Computes the hash of a file.
    pub fn compute_file_hash(&self, path: &Path) -> Result<String> {
        let content = read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Scans for file changes compared to a previous state.
    pub fn scan_changes(&self, previous_state: Option<&HashMap<PathBuf, String>>) -> Result<Vec<FileChange>> {
        let mut current_state = HashMap::new();
        let mut changes = Vec::new();

        // Scan all files in the base directory
        for entry in WalkDir::new(&self.base_dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.base_dir).unwrap_or(path);
            let relative_path_str = relative_path.to_str().unwrap_or("");

            // Check if the file should be included
            let should_include = self.include_patterns.iter().any(|pattern| glob::Pattern::new(pattern).unwrap().matches(relative_path_str));

            if !should_include {
                continue;
            }

            // Check if the file should be excluded
            let should_exclude = self.exclude_patterns.iter().any(|pattern| glob::Pattern::new(pattern).unwrap().matches(relative_path_str));

            if should_exclude {
                continue;
            }

            // Compute the current hash
            let current_hash = self.compute_file_hash(path)?;
            current_state.insert(relative_path.to_path_buf(), current_hash.clone());

            // Check if this is a new file or a modified file
            if let Some(prev_state) = previous_state {
                if let Some(old_hash) = prev_state.get(relative_path) {
                    if old_hash != &current_hash {
                        changes.push(FileChange { path: relative_path.to_path_buf(), r#type: FileChangeType::Modified, old_hash: Some(old_hash.clone()), new_hash: Some(current_hash), modified_time: Some(FileTime::from_last_modification_time(&entry.metadata().map_err(|e| Error::external_error("fs".to_string(), e.to_string(), Span::unknown()))?).unix_seconds() as u64) });
                    }
                }
                else {
                    changes.push(FileChange { path: relative_path.to_path_buf(), r#type: FileChangeType::Added, old_hash: None, new_hash: Some(current_hash), modified_time: Some(FileTime::from_last_modification_time(&entry.metadata().map_err(|e| Error::external_error("fs".to_string(), e.to_string(), Span::unknown()))?).unix_seconds() as u64) });
                }
            }
        }

        // Check for deleted files
        if let Some(prev_state) = previous_state {
            for (path, old_hash) in prev_state {
                if !current_state.contains_key(path) {
                    changes.push(FileChange { path: path.clone(), r#type: FileChangeType::Deleted, old_hash: Some(old_hash.clone()), new_hash: None, modified_time: None });
                }
            }
        }

        Ok(changes)
    }

    /// Generates a summary of the changes.
    pub fn generate_change_summary(&self, changes: &[FileChange]) -> String {
        let mut summary = String::new();
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;

        for change in changes {
            match change.r#type {
                FileChangeType::Added => added += 1,
                FileChangeType::Modified => modified += 1,
                FileChangeType::Deleted => deleted += 1,
            }
        }

        summary.push_str(&format!("File changes summary:\n"));
        summary.push_str(&format!("- Added: {}\n", added));
        summary.push_str(&format!("- Modified: {}\n", modified));
        summary.push_str(&format!("- Deleted: {}\n", deleted));

        if !changes.is_empty() {
            summary.push_str("\nDetailed changes:\n");
            for change in changes {
                let change_type_str = match change.r#type {
                    FileChangeType::Added => "Added",
                    FileChangeType::Modified => "Modified",
                    FileChangeType::Deleted => "Deleted",
                };
                summary.push_str(&format!("- {}: {}\n", change_type_str, change.path.display()));
            }
        }

        summary
    }
}
