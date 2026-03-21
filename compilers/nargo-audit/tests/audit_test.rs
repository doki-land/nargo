use nargo_audit::NargoAudit;
use nargo_types::{config::NargoConfig, context::NargoContext};
use std::{fs, sync::Arc};
use tempfile::tempdir;

#[tokio::test]
async fn test_audit_secrets() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a file with a secret
    let secret_file = root.join("secret.txt");
    fs::write(&secret_file, "My AWS key is AKIA1234567890ABCDEF").unwrap();

    let mut config = NargoConfig::default();
    config.root = root.to_path_buf();
    let ctx = Arc::new(NargoContext::new(config));
    let auditor = NargoAudit::new(ctx);

    let issues = auditor.audit_secrets().await.unwrap();
    assert!(!issues.is_empty());
    assert_eq!(issues[0].code, "AWS_KEY");
}

#[tokio::test]
async fn test_audit_dependencies() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a package.json with dangerous dep
    let pkg_json = root.join("package.json");
    fs::write(&pkg_json, r#"{"dependencies": {"request": "2.88.2"}}"#).unwrap();

    let mut config = NargoConfig::default();
    config.root = root.to_path_buf();
    let ctx = Arc::new(NargoContext::new(config));
    let auditor = NargoAudit::new(ctx);

    let issues = auditor.audit_dependencies().await.unwrap();
    assert!(!issues.is_empty());
    assert_eq!(issues[0].code, "DEPRECATED_DEP");
}

#[tokio::test]
async fn test_audit_dangerous_patterns() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a file with dangerous pattern
    let code_file = root.join("app.ts");
    fs::write(&code_file, "const x = eval(\"1 + 1\");").unwrap();

    let mut config = NargoConfig::default();
    config.root = root.to_path_buf();
    let ctx = Arc::new(NargoContext::new(config));
    let auditor = NargoAudit::new(ctx);

    let issues = auditor.audit_dangerous_patterns().await.unwrap();
    assert!(!issues.is_empty());
    assert_eq!(issues[0].code, "EVAL_USAGE");
}
