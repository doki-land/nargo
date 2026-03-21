#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Change set type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum ChangeType {
    /// A breaking change.
    #[serde(rename = "breaking")]
    Breaking,
    /// A new feature.
    #[serde(rename = "feature")]
    Feature,
    /// A bug fix.
    #[serde(rename = "fix")]
    Fix,
    /// A documentation change.
    #[serde(rename = "docs")]
    Docs,
    /// A refactoring change.
    #[serde(rename = "refactor")]
    Refactor,
    /// A performance improvement.
    #[serde(rename = "perf")]
    Perf,
    /// A test change.
    #[serde(rename = "test")]
    Test,
    /// A build change.
    #[serde(rename = "build")]
    Build,
    /// A chore change.
    #[serde(rename = "chore")]
    Chore,
}

impl ChangeType {
    /// Returns the string representation of the change type.
    pub fn as_str(&self) -> &str {
        match self {
            ChangeType::Breaking => "breaking",
            ChangeType::Feature => "feature",
            ChangeType::Fix => "fix",
            ChangeType::Docs => "docs",
            ChangeType::Refactor => "refactor",
            ChangeType::Perf => "perf",
            ChangeType::Test => "test",
            ChangeType::Build => "build",
            ChangeType::Chore => "chore",
        }
    }
}

/// File change type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileChangeType {
    /// A new file was added.
    Added,
    /// An existing file was modified.
    Modified,
    /// A file was deleted.
    Deleted,
}

/// Version control system type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VcsType {
    /// Git version control system.
    Git,
    /// Subversion (SVN) version control system.
    Svn,
    /// Mercurial (Hg) version control system.
    Mercurial,
    /// No version control system detected.
    None,
}
