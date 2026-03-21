#![warn(missing_docs)]

use nargo_linter::Severity;
use nargo_types::{NargoContext, Result};
use oak_json;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditIssue {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub category: AuditCategory,
}

impl AuditIssue {
    pub fn category_name(&self) -> &'static str {
        self.category.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditCategory {
    Secret,
    Dependency,
    DangerousPattern,
}

impl AuditCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditCategory::Secret => "Secret",
            AuditCategory::Dependency => "Dependency",
            AuditCategory::DangerousPattern => "DangerousPattern",
        }
    }
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct NargoAudit {
    ctx: Arc<NargoContext>,
    secret_regexes: Vec<(String, Regex)>,
}

impl NargoAudit {
    pub fn new(ctx: Arc<NargoContext>) -> Self {
        let mut secret_regexes = Vec::new();

        // Common secret patterns
        let patterns = vec![("AWS_KEY", r"AKIA[0-9A-Z]{16}"), ("GITHUB_TOKEN", r"ghp_[a-zA-Z0-9]{36}"), ("PRIVATE_KEY", r"-----BEGIN [A-Z ]+ PRIVATE KEY-----"), ("GENERIC_SECRET", r"(?i)secret|password|token|api_key|apikey")];

        for (name, pattern) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                secret_regexes.push((name.to_string(), re));
            }
        }

        Self { ctx, secret_regexes }
    }

    pub async fn run_all(&self) -> Result<Vec<AuditIssue>> {
        let mut issues = Vec::new();

        issues.extend(self.audit_secrets().await?);
        issues.extend(self.audit_dependencies().await?);
        issues.extend(self.audit_dangerous_patterns().await?);

        Ok(issues)
    }

    /// Scan for secrets in the codebase
    pub async fn audit_secrets(&self) -> Result<Vec<AuditIssue>> {
        let mut issues = Vec::new();
        let root = std::env::current_dir()?;

        for entry in WalkDir::new(&root).into_iter().filter_entry(|e| !self.is_ignored(e)).filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(path) {
                    for (name, re) in &self.secret_regexes {
                        for mat in re.find_iter(&content) {
                            let (line, col) = self.get_line_col(&content, mat.start());
                            issues.push(AuditIssue { file: path.to_path_buf(), line, column: col, code: name.clone(), message: format!("Potential secret found: {}", name), severity: Severity::Error, category: AuditCategory::Secret });
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Scan for dangerous dependencies in package.json
    pub async fn audit_dependencies(&self) -> Result<Vec<AuditIssue>> {
        let mut issues = Vec::new();
        let root = std::env::current_dir()?;
        let package_json_path = root.join("package.json");

        if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path)?;
            // 暂时跳过 JSON 解析，因为 oak_json 的 API 与 serde_json 不同
            // 后续可以根据 oak_json 的 API 进行适配
        }

        Ok(issues)
    }

    /// Scan for dangerous code patterns
    pub async fn audit_dangerous_patterns(&self) -> Result<Vec<AuditIssue>> {
        let mut issues = Vec::new();
        let root = std::env::current_dir()?;

        let patterns = vec![("EVAL_USAGE", Regex::new(r"eval\s*\(").unwrap()), ("INNER_HTML", Regex::new(r"dangerouslySetInnerHTML").unwrap()), ("FUNCTION_CTOR", Regex::new(r"new\s+Function\s*\(").unwrap())];

        for entry in WalkDir::new(&root).into_iter().filter_entry(|e| !self.is_ignored(e)).filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(ext, "ts" | "js" | "nargo" | "tsx" | "jsx") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        for (code, re) in &patterns {
                            for mat in re.find_iter(&content) {
                                let (line, col) = self.get_line_col(&content, mat.start());
                                issues.push(AuditIssue { file: path.to_path_buf(), line, column: col, code: code.to_string(), message: format!("Dangerous pattern detected: {}", code), severity: Severity::Warning, category: AuditCategory::DangerousPattern });
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn is_ignored(&self, entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_str().unwrap_or("");
        name == "node_modules" || name == ".git" || name == "target" || name == "dist" || name == "dist-check" || name.ends_with(".html")
    }

    fn get_line_col(&self, content: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in content.char_indices() {
            if i == offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            }
            else {
                col += 1;
            }
        }
        (line, col)
    }
}
