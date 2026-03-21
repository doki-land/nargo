//! Linter enhancement tests.

use async_trait::async_trait;
use nargo_linter::{Diagnostic, LintRule, LinterConfig, NargoLinter, Severity};
use nargo_types::Result;

/// Test custom rule.
struct TestCustomRule;

#[async_trait]
impl LintRule for TestCustomRule {
    fn name(&self) -> &'static str {
        "test-custom-rule"
    }

    async fn check(&self, _source: &str, _module: &nargo_ir::IRModule) -> Result<Vec<Diagnostic>> {
        Ok(vec![Diagnostic { code: self.name().to_string(), message: "Test custom rule triggered".to_string(), severity: Severity::Warning, line: 1, column: 1 }])
    }

    async fn fix(&self, _source: &str, _module: &nargo_ir::IRModule) -> Result<Vec<nargo_linter::FixAction>> {
        Ok(Vec::new())
    }
}

#[tokio::test]
async fn test_no_alert_rule() {
    let config = LinterConfig { rules: std::collections::HashMap::new(), extends: Some("recommended".to_string()) };

    let mut linter = NargoLinter::with_config(config);
    linter.add_builtin_rules();

    let source = r#"
    <template>
        <div>Hello</div>
    </template>
    <script>
        alert('Hello');
        console.log('World');
    </script>
    "#;

    let diagnostics = linter.run("test.tsx", source).await.unwrap();
    assert!(!diagnostics.is_empty());

    let alert_diagnostics: Vec<_> = diagnostics.iter().filter(|d| d.code == "no-alert").collect();
    assert!(!alert_diagnostics.is_empty());
}

#[tokio::test]
async fn test_custom_rule_support() {
    let config = LinterConfig { rules: std::collections::HashMap::new(), extends: None };

    let mut linter = NargoLinter::with_config(config);
    linter.add_rule(Box::new(TestCustomRule));

    let source = r#"
    <template>
        <div>Hello</div>
    </template>
    "#;

    let diagnostics = linter.run("test.tsx", source).await.unwrap();
    assert!(!diagnostics.is_empty());

    let custom_diagnostics: Vec<_> = diagnostics.iter().filter(|d| d.code == "test-custom-rule").collect();
    assert!(!custom_diagnostics.is_empty());
}

#[tokio::test]
async fn test_with_all_rules() {
    let config = LinterConfig { rules: std::collections::HashMap::new(), extends: Some("recommended".to_string()) };

    let linter = NargoLinter::with_all_rules(config);
    assert!(!linter.rules.is_empty());

    let source = r#"
    <template>
        <div>Hello</div>
    </template>
    <script>
        alert('Hello');
        console.log('World');
    </script>
    "#;

    let diagnostics = linter.run("test.tsx", source).await.unwrap();
    assert!(!diagnostics.is_empty());
}
