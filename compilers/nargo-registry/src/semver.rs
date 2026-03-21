use nargo_types::{Error, Result};
use semver::{Comparator, Op, Version, VersionReq};
use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

/// Represents a version constraint for npm-style version matching.
#[derive(Debug, Clone, PartialEq)]
pub struct NpmVersionConstraint {
    /// The raw constraint string.
    pub raw: String,
    /// The parsed comparators.
    comparators: Vec<VersionComparator>,
    /// The constraint type.
    pub constraint_type: ConstraintType,
}

/// Type of version constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintType {
    /// Standard npm semver constraint.
    Semver,
    /// Tag constraint (latest, next, beta, etc.).
    Tag(String),
    /// Workspace protocol.
    Workspace,
    /// File/path protocol.
    Path(PathBuf),
    /// Git URL.
    Git(String),
    /// GitHub shorthand.
    Github(String),
    /// Any version.
    Any,
}

/// A single version comparator.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionComparator {
    /// The operator.
    pub op: CompareOp,
    /// The major version.
    pub major: u64,
    /// The minor version (None means any).
    pub minor: Option<u64>,
    /// The patch version (None means any).
    pub patch: Option<u64>,
    /// Pre-release identifiers.
    pub pre: Vec<String>,
}

/// Comparison operator for version constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Exact match: "1.2.3"
    Exact,
    /// Greater than: ">1.2.3"
    Greater,
    /// Greater than or equal: ">=1.2.3"
    GreaterEq,
    /// Less than: "<1.2.3"
    Less,
    /// Less than or equal: "<=1.2.3"
    LessEq,
    /// Compatible with: "^1.2.3"
    Caret,
    /// Approximately: "~1.2.3"
    Tilde,
    /// Any version: "*"
    Any,
}

impl NpmVersionConstraint {
    /// Parses a version constraint string.
    ///
    /// Supports:
    /// - Exact version: "1.2.3"
    /// - Caret range: "^1.2.3"
    /// - Tilde range: "~1.2.3"
    /// - Comparison: ">=1.0.0 <2.0.0"
    /// - X-ranges: "1.x", "1.2.x"
    /// - Hyphen range: "1.0.0 - 2.0.0"
    /// - Tags: "latest", "next"
    /// - Workspace: "workspace:*", "workspace:^"
    /// - Path: "file:./path", "./path", "../path"
    /// - Git: "git+https://...", "github:user/repo"
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();
        let raw = input.to_string();

        if input.starts_with("workspace:") {
            return Self::parse_workspace(input);
        }

        if input.starts_with("file:") || input.starts_with("./") || input.starts_with("../") {
            return Self::parse_path(input);
        }

        if input.starts_with("git+") || input.starts_with("git://") {
            return Self::parse_git(input);
        }

        if input.starts_with("github:") {
            return Self::parse_github(input);
        }

        if input == "*" || input.is_empty() {
            return Ok(Self { raw, comparators: vec![VersionComparator { op: CompareOp::Any, major: 0, minor: None, patch: None, pre: vec![] }], constraint_type: ConstraintType::Any });
        }

        if input == "latest" || input == "next" || input == "beta" || input == "alpha" || input == "canary" {
            return Ok(Self { raw, comparators: vec![VersionComparator { op: CompareOp::Any, major: 0, minor: None, patch: None, pre: vec![] }], constraint_type: ConstraintType::Tag(input.to_string()) });
        }

        let comparators = Self::parse_comparators(input)?;
        Ok(Self { raw, comparators, constraint_type: ConstraintType::Semver })
    }

    /// Parses a workspace protocol constraint.
    fn parse_workspace(input: &str) -> Result<Self> {
        let raw = input.to_string();
        let version_part = input.strip_prefix("workspace:").unwrap_or("*");

        let comparators = if version_part == "*" || version_part == "^" || version_part == "~" { vec![VersionComparator { op: CompareOp::Any, major: 0, minor: None, patch: None, pre: vec![] }] } else { Self::parse_comparators(version_part)? };

        Ok(Self { raw, comparators, constraint_type: ConstraintType::Workspace })
    }

    /// Parses a path protocol constraint.
    fn parse_path(input: &str) -> Result<Self> {
        let raw = input.to_string();
        let path = if input.starts_with("file:") { PathBuf::from(input.strip_prefix("file:").unwrap()) } else { PathBuf::from(input) };

        Ok(Self { raw, comparators: vec![], constraint_type: ConstraintType::Path(path) })
    }

    /// Parses a git URL constraint.
    fn parse_git(input: &str) -> Result<Self> {
        let raw = input.to_string();
        let url = input.strip_prefix("git+").unwrap_or(input).to_string();

        Ok(Self { raw, comparators: vec![], constraint_type: ConstraintType::Git(url) })
    }

    /// Parses a GitHub shorthand constraint.
    fn parse_github(input: &str) -> Result<Self> {
        let raw = input.to_string();
        let repo = input.strip_prefix("github:").unwrap_or(input).to_string();

        Ok(Self { raw, comparators: vec![], constraint_type: ConstraintType::Github(repo) })
    }

    /// Returns true if this is a workspace constraint.
    pub fn is_workspace(&self) -> bool {
        matches!(self.constraint_type, ConstraintType::Workspace)
    }

    /// Returns true if this is a path constraint.
    pub fn is_path(&self) -> bool {
        matches!(self.constraint_type, ConstraintType::Path(_))
    }

    /// Returns true if this is a git constraint.
    pub fn is_git(&self) -> bool {
        matches!(self.constraint_type, ConstraintType::Git(_))
    }

    /// Returns true if this is a GitHub constraint.
    pub fn is_github(&self) -> bool {
        matches!(self.constraint_type, ConstraintType::Github(_))
    }

    /// Returns true if this is a tag constraint.
    pub fn is_tag(&self) -> bool {
        matches!(self.constraint_type, ConstraintType::Tag(_))
    }

    /// Returns the path if this is a path constraint.
    pub fn get_path(&self) -> Option<&PathBuf> {
        match &self.constraint_type {
            ConstraintType::Path(path) => Some(path),
            _ => None,
        }
    }

    /// Returns the git URL if this is a git constraint.
    pub fn get_git_url(&self) -> Option<&str> {
        match &self.constraint_type {
            ConstraintType::Git(url) => Some(url),
            _ => None,
        }
    }

    /// Returns the GitHub repo if this is a GitHub constraint.
    pub fn get_github_repo(&self) -> Option<&str> {
        match &self.constraint_type {
            ConstraintType::Github(repo) => Some(repo),
            _ => None,
        }
    }

    /// Returns the tag if this is a tag constraint.
    pub fn get_tag(&self) -> Option<&str> {
        match &self.constraint_type {
            ConstraintType::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    fn parse_comparators(input: &str) -> Result<Vec<VersionComparator>> {
        let input = input.trim();

        if input.contains(" - ") {
            return Self::parse_hyphen_range(input);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let mut comparators = Vec::new();

        for part in parts {
            if part.contains("||") {
                return Err(Error::external_error("semver".to_string(), format!("OR conditions not yet supported: {}", input), nargo_types::Span::unknown()));
            }

            let comparator = Self::parse_single_comparator(part)?;
            comparators.push(comparator);
        }

        if comparators.is_empty() {
            return Err(Error::external_error("semver".to_string(), "Empty version constraint".to_string(), nargo_types::Span::unknown()));
        }

        Ok(comparators)
    }

    fn parse_hyphen_range(input: &str) -> Result<Vec<VersionComparator>> {
        let parts: Vec<&str> = input.split(" - ").collect();
        if parts.len() != 2 {
            return Err(Error::external_error("semver".to_string(), format!("Invalid hyphen range: {}", input), nargo_types::Span::unknown()));
        }

        let start = Self::parse_single_comparator(parts[0])?;
        let end = Self::parse_single_comparator(parts[1])?;

        Ok(vec![VersionComparator { op: CompareOp::GreaterEq, major: start.major, minor: start.minor, patch: start.patch, pre: start.pre }, VersionComparator { op: CompareOp::LessEq, major: end.major, minor: end.minor, patch: end.patch, pre: end.pre }])
    }

    fn parse_single_comparator(input: &str) -> Result<VersionComparator> {
        let input = input.trim();

        if input == "*" || input == "x" || input == "X" {
            return Ok(VersionComparator { op: CompareOp::Any, major: 0, minor: None, patch: None, pre: vec![] });
        }

        let (op, version_str) = if input.starts_with("^") {
            (CompareOp::Caret, &input[1..])
        }
        else if input.starts_with("~") {
            (CompareOp::Tilde, &input[1..])
        }
        else if input.starts_with(">=") {
            (CompareOp::GreaterEq, &input[2..])
        }
        else if input.starts_with("<=") {
            (CompareOp::LessEq, &input[2..])
        }
        else if input.starts_with(">") {
            (CompareOp::Greater, &input[1..])
        }
        else if input.starts_with("<") {
            (CompareOp::Less, &input[1..])
        }
        else if input.starts_with("=") {
            (CompareOp::Exact, &input[1..])
        }
        else {
            (CompareOp::Exact, input)
        };

        let version_str = version_str.trim();
        Self::parse_version_parts(op, version_str)
    }

    fn parse_version_parts(op: CompareOp, version_str: &str) -> Result<VersionComparator> {
        let version_str = version_str.trim_end_matches(".x").trim_end_matches(".X");
        let version_str = version_str.trim_end_matches("-x").trim_end_matches("-X");

        let parts: Vec<&str> = version_str.split('.').collect();
        let mut pre = Vec::new();

        let major_str = parts.get(0).unwrap_or(&"0");
        let major = if *major_str == "x" || *major_str == "X" || *major_str == "*" {
            return Ok(VersionComparator { op: CompareOp::Any, major: 0, minor: None, patch: None, pre: vec![] });
        }
        else {
            major_str.parse::<u64>().map_err(|e| Error::external_error("semver".to_string(), e.to_string(), nargo_types::Span::unknown()))?
        };

        let minor = if let Some(minor_str) = parts.get(1) {
            if *minor_str == "x" || *minor_str == "X" || *minor_str == "*" {
                None
            }
            else if minor_str.contains('-') {
                let parts: Vec<&str> = minor_str.split('-').collect();
                pre.push(parts[1..].join("-"));
                Some(parts[0].parse::<u64>().map_err(|e| Error::external_error("semver".to_string(), e.to_string(), nargo_types::Span::unknown()))?)
            }
            else {
                Some(minor_str.parse::<u64>().map_err(|e| Error::external_error("semver".to_string(), e.to_string(), nargo_types::Span::unknown()))?)
            }
        }
        else {
            None
        };

        let patch = if let Some(patch_str) = parts.get(2) {
            if *patch_str == "x" || *patch_str == "X" || *patch_str == "*" {
                None
            }
            else if patch_str.contains('-') {
                let parts: Vec<&str> = patch_str.splitn(2, '-').collect();
                pre.extend(parts[1].split('.').map(String::from));
                Some(parts[0].parse::<u64>().map_err(|e| Error::external_error("semver".to_string(), e.to_string(), nargo_types::Span::unknown()))?)
            }
            else {
                Some(patch_str.parse::<u64>().map_err(|e| Error::external_error("semver".to_string(), e.to_string(), nargo_types::Span::unknown()))?)
            }
        }
        else {
            None
        };

        if parts.len() > 3 {
            for extra in parts.iter().skip(3) {
                pre.push((*extra).to_string());
            }
        }

        Ok(VersionComparator { op, major, minor, patch, pre })
    }

    /// Checks if a version satisfies this constraint.
    pub fn satisfies(&self, version: &str) -> bool {
        if let Ok(v) = Version::parse(version) { self.satisfies_version(&v) } else { false }
    }

    /// Checks if a parsed version satisfies this constraint.
    pub fn satisfies_version(&self, version: &Version) -> bool {
        for comparator in &self.comparators {
            if !comparator.matches(version) {
                return false;
            }
        }
        true
    }

    /// Finds the best matching version from a list of available versions.
    pub fn find_best_match<'a>(&self, versions: impl IntoIterator<Item = &'a str>, tags: &HashMap<String, String>) -> Option<String> {
        if self.raw == "latest" {
            return tags.get("latest").cloned();
        }
        if self.raw == "next" {
            return tags.get("next").cloned();
        }

        let mut matching: Vec<Version> = versions.into_iter().filter_map(|v| Version::parse(v).ok()).filter(|v| self.satisfies_version(v)).collect();

        matching.sort_by(|a, b| b.cmp(a));

        matching.first().map(|v| v.to_string())
    }
}

impl VersionComparator {
    /// Checks if this comparator matches a given version.
    pub fn matches(&self, version: &Version) -> bool {
        match self.op {
            CompareOp::Any => true,
            CompareOp::Exact => self.matches_exact(version),
            CompareOp::Greater => self.compare(version) == Ordering::Greater,
            CompareOp::GreaterEq => self.compare(version) != Ordering::Less,
            CompareOp::Less => self.compare(version) == Ordering::Less,
            CompareOp::LessEq => self.compare(version) != Ordering::Greater,
            CompareOp::Caret => self.matches_caret(version),
            CompareOp::Tilde => self.matches_tilde(version),
        }
    }

    fn matches_exact(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }
        if let Some(minor) = self.minor {
            if version.minor != minor {
                return false;
            }
        }
        if let Some(patch) = self.patch {
            if version.patch != patch {
                return false;
            }
        }
        if !self.pre.is_empty() {
            let pre_str = version.pre.to_string();
            let pre_parts: Vec<String> = if pre_str.is_empty() { vec![] } else { pre_str.split('.').map(String::from).collect() };
            if pre_parts != self.pre {
                return false;
            }
        }
        else if !version.pre.is_empty() {
            return false;
        }
        true
    }

    fn matches_caret(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        if let Some(minor) = self.minor {
            if version.minor < minor {
                return false;
            }
            if self.major == 0 {
                if version.minor > minor {
                    return false;
                }
                if let Some(patch) = self.patch {
                    if version.patch < patch {
                        return false;
                    }
                }
                else if version.patch > 0 && version.minor == minor {
                    return true;
                }
            }
        }

        if let Some(patch) = self.patch {
            if version.patch < patch && self.minor.map_or(false, |m| version.minor == m) {
                return false;
            }
        }

        self.compare_pre(version)
    }

    fn matches_tilde(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        if let Some(minor) = self.minor {
            if version.minor != minor {
                return false;
            }
            if let Some(patch) = self.patch {
                if version.patch < patch {
                    return false;
                }
            }
        }

        self.compare_pre(version)
    }

    fn compare(&self, version: &Version) -> Ordering {
        let cmp = version.major.cmp(&self.major);
        if cmp != Ordering::Equal {
            return cmp;
        }

        match self.minor {
            Some(minor) => {
                let cmp = version.minor.cmp(&minor);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            None => return Ordering::Equal,
        }

        match self.patch {
            Some(patch) => version.patch.cmp(&patch),
            None => Ordering::Equal,
        }
    }

    fn compare_pre(&self, version: &Version) -> bool {
        if self.pre.is_empty() && !version.pre.is_empty() {
            return false;
        }
        true
    }
}

/// Resolves a version constraint against available versions.
pub fn resolve_version(constraint: &str, versions: &[String], tags: &HashMap<String, String>) -> Result<String> {
    let parsed = NpmVersionConstraint::parse(constraint)?;
    parsed.find_best_match(versions.iter().map(String::as_str), tags).ok_or_else(|| Error::external_error("semver".to_string(), format!("No matching version found for constraint: {}", constraint), nargo_types::Span::unknown()))
}

/// Checks if a version satisfies a constraint.
pub fn satisfies(constraint: &str, version: &str) -> bool {
    NpmVersionConstraint::parse(constraint).map(|c| c.satisfies(version)).unwrap_or(false)
}
