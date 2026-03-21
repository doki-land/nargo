use nargo_linter::{
    NargoLinter,
    rules::{NoConsole, NoDebugger, NoDeprecatedTags, NoEmptyTemplate, NoMagicNumbers, NoUnreachableCode, NoUnusedVars},
};

#[tokio::test]
async fn test_no_console_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoConsole));

    let source = "<script>\nfunction test() {\n  console.log('hello');\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-console"));
}

#[tokio::test]
async fn test_no_debugger_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoDebugger));

    let source = "<script>\nfunction test() {\n  debugger;\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-debugger"));
}

#[tokio::test]
async fn test_no_deprecated_tags_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoDeprecatedTags));

    let source = "<template>\n  <center>\n    <h1>Hello</h1>\n  </center>\n</template>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-deprecated-tags"));
    assert_eq!(diagnostics[0].line, 2);
}

#[tokio::test]
async fn test_no_empty_template_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoEmptyTemplate));

    let source = "<template>\n</template>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-empty-template"));
}

#[tokio::test]
async fn test_multiple_rules() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoConsole));
    linter.add_rule(Box::new(NoDebugger));

    let source = "<script>\nfunction test() {\n  console.log('hello');\n  debugger;\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-console"));
    assert!(diagnostics.iter().any(|d| d.code == "no-debugger"));
    assert_eq!(diagnostics.len(), 2);
}

#[tokio::test]
async fn test_no_unused_vars_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoUnusedVars));

    let source = "<script>\nfunction test() {\n  const unusedVar = 123;\n  console.log('hello');\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-unused-vars"));
}

#[tokio::test]
async fn test_no_unreachable_code_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoUnreachableCode));

    let source = "<script>\nfunction test() {\n  return 42;\n  console.log('unreachable');\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-unreachable-code"));
}

#[tokio::test]
async fn test_no_magic_numbers_rule() {
    let mut linter = NargoLinter::new();
    linter.add_rule(Box::new(NoMagicNumbers));

    let source = "<script>\nfunction test() {\n  const result = 42;\n  return result;\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-magic-numbers"));
}

#[tokio::test]
async fn test_rule_config_disabled() {
    use nargo_linter::{LinterConfig, RuleConfig, Severity};

    let mut config = LinterConfig { rules: std::collections::HashMap::new(), extends: None };
    config.rules.insert("no-console".to_string(), RuleConfig { enabled: false, severity: Some(Severity::Warning) });

    let mut linter = NargoLinter::with_config(config);
    linter.add_rule(Box::new(NoConsole));

    let source = "<script>\nfunction test() {\n  console.log('hello');\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(!diagnostics.iter().any(|d| d.code == "no-console"));
}

#[tokio::test]
async fn test_rule_set_extends() {
    use nargo_linter::LinterConfig;

    let config = LinterConfig { rules: std::collections::HashMap::new(), extends: Some("recommended".to_string()) };

    let mut linter = NargoLinter::with_config(config);
    linter.add_rule(Box::new(NoConsole));
    linter.add_rule(Box::new(NoDebugger));

    let source = "<script>\nfunction test() {\n  console.log('hello');\n  debugger;\n}\n</script>";
    let diagnostics = linter.run("test.ts", source).await.unwrap();

    assert!(diagnostics.iter().any(|d| d.code == "no-console"));
    assert!(diagnostics.iter().any(|d| d.code == "no-debugger"));
}
