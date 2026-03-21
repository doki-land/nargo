use nargo_workspace::*;

#[test]
fn test_workspace_member_creation() {
    let member = WorkspaceMember::new("test-pkg", "/path/to/pkg").with_version("1.0.0");

    assert_eq!(member.name, "test-pkg");
    assert_eq!(member.version, "1.0.0");
    assert!(!member.is_root);
}

#[test]
fn test_workspace_member_dependencies() {
    let mut member = WorkspaceMember::new("app", "/path/to/app");
    member.add_dependency("lodash");
    member.add_dev_dependency("typescript");

    assert_eq!(member.dependencies, vec!["lodash"]);
    assert_eq!(member.dev_dependencies, vec!["typescript"]);
}

#[test]
fn test_shared_config() {
    let config = SharedConfig { version: Some("1.0.0".to_string()), authors: vec!["Team".to_string()], license: Some("MIT".to_string()), ..Default::default() };

    assert_eq!(config.version, Some("1.0.0".to_string()));
    assert_eq!(config.authors, vec!["Team".to_string()]);
    assert_eq!(config.license, Some("MIT".to_string()));
}
