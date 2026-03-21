//! Nargo linter module.

#![warn(missing_docs)]

use async_trait::async_trait;
use nargo_ir::IRModule;
use nargo_types::{Error, Result};
use serde::{Deserialize, Serialize};

pub mod rules;

/// Diagnostic severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// Error level.
    Error,
    /// Warning level.
    Warning,
    /// Hint level.
    Hint,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "Error"),
            Severity::Warning => write!(f, "Warning"),
            Severity::Hint => write!(f, "Hint"),
        }
    }
}

/// Diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic code.
    pub code: String,
    /// Diagnostic message.
    pub message: String,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Line number.
    pub line: usize,
    /// Column number.
    pub column: usize,
}

/// Fix action for lint errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixAction {
    /// Start position of the fix.
    pub start: usize,
    /// End position of the fix.
    pub end: usize,
    /// Replacement text.
    pub replacement: String,
    /// Description of the fix.
    pub description: String,
}

/// Lint rule trait.
#[async_trait]
pub trait LintRule: Send + Sync {
    /// Returns the rule name.
    fn name(&self) -> &'static str;
    /// Checks the source code.
    async fn check(&self, source: &str, module: &IRModule) -> Result<Vec<Diagnostic>>;
    /// Fixes the lint errors.
    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>>;
}

/// Rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Rule severity.
    pub severity: Option<Severity>,
    /// Whether auto-fix is enabled for this rule.
    pub fix: Option<bool>,
}

/// Linter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    /// Rules configuration.
    pub rules: std::collections::HashMap<String, RuleConfig>,
    /// Selected rule set.
    pub extends: Option<String>,
}

/// Get predefined rule set.
pub fn get_predefined_rule_set(name: &str) -> Option<LinterConfig> {
    match name {
        "recommended" => Some(LinterConfig { rules: vec![("no-console".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-debugger".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }), ("no-unused-vars".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-unused-imports".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-unreachable-code".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-magic-numbers".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-alert".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Warning), fix: None }), ("no-undeclared-variables".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None })].into_iter().collect(), extends: None }),
        "strict" => Some(LinterConfig {
            rules: vec![
                ("no-console".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-debugger".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-unused-vars".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-unused-imports".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-unreachable-code".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-empty-template".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-deprecated-tags".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-magic-numbers".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-alert".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
                ("no-undeclared-variables".to_string(), RuleConfig { enabled: true, severity: Some(Severity::Error), fix: None }),
            ]
            .into_iter()
            .collect(),
            extends: None,
        }),
        _ => None,
    }
}

/// Nargo linter.
pub struct NargoLinter {
    rules: Vec<Box<dyn LintRule>>,
    config: LinterConfig,
}

impl NargoLinter {
    /// Create a new linter with default configuration.
    pub fn new() -> Self {
        let config = LinterConfig { rules: std::collections::HashMap::new(), extends: None };
        Self { rules: Vec::new(), config }
    }

    /// Create a new linter with custom configuration.
    pub fn with_config(mut config: LinterConfig) -> Self {
        // Handle rule set extension
        if let Some(extends) = &config.extends {
            if let Some(rule_set) = get_predefined_rule_set(extends) {
                // Merge rule set into config
                for (rule_name, rule_config) in rule_set.rules {
                    if !config.rules.contains_key(&rule_name) {
                        config.rules.insert(rule_name, rule_config);
                    }
                }
            }
        }
        Self { rules: Vec::new(), config }
    }

    /// Create a new linter with all built-in rules.
    pub fn with_all_rules(config: LinterConfig) -> Self {
        let mut linter = Self::with_config(config);
        linter.add_builtin_rules();
        linter
    }

    /// Add all built-in rules.
    pub fn add_builtin_rules(&mut self) {
        self.add_rule(Box::new(rules::NoConsole));
        self.add_rule(Box::new(rules::NoDebugger));
        self.add_rule(Box::new(rules::NoDeprecatedTags));
        self.add_rule(Box::new(rules::NoEmptyTemplate));
        self.add_rule(Box::new(rules::NoUnusedVars));
        self.add_rule(Box::new(rules::NoUnusedImports));
        self.add_rule(Box::new(rules::NoUnreachableCode));
        self.add_rule(Box::new(rules::NoMagicNumbers));
        self.add_rule(Box::new(rules::NoAlert));
        self.add_rule(Box::new(rules::NoUndeclaredVariables));
    }

    /// Add a lint rule.
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    /// Run all lint rules.
    pub async fn run(&self, name: &str, source: &str) -> Result<Vec<Diagnostic>> {
        // Create a new parser for each run
        let registry = std::sync::Arc::new(nargo_parser::ParserRegistry::new());
        let mut parser = nargo_parser::Parser::new(name.to_string(), source, registry);
        let module = parser.parse_all()?;
        let mut all_diagnostics = Vec::new();

        println!("Rules count: {}", self.rules.len());
        println!("Module name: {}", module.name);
        println!("Module has script: {:?}", module.script.is_some());
        println!("Module has template: {:?}", module.template.is_some());
        if let Some(script) = &module.script {
            println!("Script body length: {}", script.body.len());
        }
        if let Some(template) = &module.template {
            println!("Template nodes length: {}", template.nodes.len());
        }

        for rule in &self.rules {
            let rule_name = rule.name();
            println!("Running rule: {}", rule_name);

            // Check if the rule is enabled in config
            let is_enabled = if let Some(rule_config) = self.config.rules.get(rule_name) {
                println!("Rule {} is enabled: {}", rule_name, rule_config.enabled);
                rule_config.enabled
            }
            else {
                // Default to enabled if not in config
                println!("Rule {} is not in config, defaulting to enabled", rule_name);
                true
            };
            if is_enabled {
                let diagnostics = rule.check(source, &module).await?;
                println!("Rule {} found {} diagnostics", rule_name, diagnostics.len());
                all_diagnostics.extend(diagnostics);
            }
        }

        println!("Total diagnostics: {}", all_diagnostics.len());
        Ok(all_diagnostics)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LinterConfig {
        &self.config
    }

    /// Run all lint rules and generate fix actions.
    pub async fn run_fix(&self, name: &str, source: &str) -> Result<Vec<FixAction>> {
        // Create a new parser for each run
        let registry = std::sync::Arc::new(nargo_parser::ParserRegistry::new());
        let mut parser = nargo_parser::Parser::new(name.to_string(), source, registry);
        let module = parser.parse_all()?;
        let mut all_fix_actions = Vec::new();

        for rule in &self.rules {
            let rule_name = rule.name();

            // Check if the rule is enabled in config
            let is_enabled = if let Some(rule_config) = self.config.rules.get(rule_name) {
                rule_config.enabled
            }
            else {
                // Default to enabled if not in config
                true
            };

            // Check if fix is enabled for this rule
            let is_fix_enabled = if let Some(rule_config) = self.config.rules.get(rule_name) {
                rule_config.fix.unwrap_or(true)
            }
            else {
                // Default to true if not specified
                true
            };

            if is_enabled && is_fix_enabled {
                let fix_actions = rule.fix(source, &module).await?;
                all_fix_actions.extend(fix_actions);
            }
        }

        Ok(all_fix_actions)
    }

    /// Apply fix actions to the source code.
    pub fn apply_fixes(&self, source: &str, fix_actions: &[FixAction]) -> String {
        // Sort fix actions by start position in reverse order to avoid position shifting
        let mut sorted_actions = fix_actions.to_vec();
        sorted_actions.sort_by(|a, b| b.start.cmp(&a.start));

        let mut result = source.to_string();
        for action in sorted_actions {
            let start = action.start;
            let end = action.end;
            if start <= end && end <= result.len() {
                result = format!("{}{}{}", &result[..start], action.replacement, &result[end..]);
            }
        }

        result
    }
}
