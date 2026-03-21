#![warn(missing_docs)]

use crate::{
    change_set::ChangeSet,
    file_change::{FileChange, FileChangeDetector},
};
use std::path::Path;

/// Change preview.
pub struct ChangePreview {
    /// The file changes.
    pub file_changes: Vec<FileChange>,
    /// The change sets.
    pub change_sets: Vec<ChangeSet>,
}

impl ChangePreview {
    /// Creates a new change preview.
    pub fn new(file_changes: Vec<FileChange>, change_sets: Vec<ChangeSet>) -> Self {
        Self { file_changes, change_sets }
    }

    /// Generates a preview of the changes.
    pub fn generate_preview(&self) -> String {
        let mut preview = String::new();

        // Add file changes summary
        let detector = FileChangeDetector::new(Path::new("."));
        preview.push_str(&detector.generate_change_summary(&self.file_changes));

        // Add change sets summary
        if !self.change_sets.is_empty() {
            preview.push_str("\nChange sets:\n");
            for change_set in &self.change_sets {
                preview.push_str(&format!("- [{}] {}\n", change_set.r#type.as_str(), change_set.summary));
            }
        }

        preview
    }
}
