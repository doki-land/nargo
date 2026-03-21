use nargo_registry::{ConstraintType, NpmVersionConstraint};
use std::{collections::HashMap, path::PathBuf};

#[test]
fn test_exact_version() {
    let constraint = NpmVersionConstraint::parse("1.2.3").unwrap();
    assert!(constraint.satisfies("1.2.3"));
    assert!(!constraint.satisfies("1.2.4"));
    assert!(!constraint.satisfies("2.0.0"));
    assert_eq!(constraint.constraint_type, ConstraintType::Semver);
}

#[test]
fn test_caret_range() {
    let constraint = NpmVersionConstraint::parse("^1.2.3").unwrap();
    assert!(constraint.satisfies("1.2.3"));
    assert!(constraint.satisfies("1.2.4"));
    assert!(constraint.satisfies("1.3.0"));
    assert!(!constraint.satisfies("2.0.0"));
    assert!(!constraint.satisfies("1.2.2"));
}

#[test]
fn test_tilde_range() {
    let constraint = NpmVersionConstraint::parse("~1.2.3").unwrap();
    assert!(constraint.satisfies("1.2.3"));
    assert!(constraint.satisfies("1.2.4"));
    assert!(constraint.satisfies("1.2.9"));
    assert!(!constraint.satisfies("1.3.0"));
    assert!(!constraint.satisfies("2.0.0"));
}

#[test]
fn test_range() {
    let constraint = NpmVersionConstraint::parse(">=1.0.0 <2.0.0").unwrap();
    assert!(constraint.satisfies("1.0.0"));
    assert!(constraint.satisfies("1.5.0"));
    assert!(constraint.satisfies("1.9.9"));
    assert!(!constraint.satisfies("2.0.0"));
    assert!(!constraint.satisfies("0.9.9"));
}

#[test]
fn test_x_range() {
    let constraint = NpmVersionConstraint::parse("1.x").unwrap();
    assert!(constraint.satisfies("1.0.0"));
    assert!(constraint.satisfies("1.5.0"));
    assert!(constraint.satisfies("1.9.9"));
    assert!(!constraint.satisfies("2.0.0"));
}

#[test]
fn test_any_version() {
    let constraint = NpmVersionConstraint::parse("*").unwrap();
    assert!(constraint.satisfies("1.0.0"));
    assert!(constraint.satisfies("2.0.0"));
    assert_eq!(constraint.constraint_type, ConstraintType::Any);
}

#[test]
fn test_find_best_match() {
    let constraint = NpmVersionConstraint::parse("^1.0.0").unwrap();
    let versions = vec!["1.0.0".to_string(), "1.1.0".to_string(), "1.2.0".to_string(), "2.0.0".to_string()];
    let tags = HashMap::new();
    let best = constraint.find_best_match(versions.iter().map(String::as_str), &tags);
    assert_eq!(best, Some("1.2.0".to_string()));
}

#[test]
fn test_workspace_constraint() {
    let constraint = NpmVersionConstraint::parse("workspace:*").unwrap();
    assert!(constraint.is_workspace());
    assert_eq!(constraint.constraint_type, ConstraintType::Workspace);

    let constraint = NpmVersionConstraint::parse("workspace:^").unwrap();
    assert!(constraint.is_workspace());

    let constraint = NpmVersionConstraint::parse("workspace:1.2.3").unwrap();
    assert!(constraint.is_workspace());
    assert!(constraint.satisfies("1.2.3"));
}

#[test]
fn test_path_constraint() {
    let constraint = NpmVersionConstraint::parse("file:../local-pkg").unwrap();
    assert!(constraint.is_path());
    assert_eq!(constraint.get_path(), Some(&PathBuf::from("../local-pkg")));

    let constraint = NpmVersionConstraint::parse("./local-pkg").unwrap();
    assert!(constraint.is_path());
    assert_eq!(constraint.get_path(), Some(&PathBuf::from("./local-pkg")));

    let constraint = NpmVersionConstraint::parse("../sibling-pkg").unwrap();
    assert!(constraint.is_path());
}

#[test]
fn test_git_constraint() {
    let constraint = NpmVersionConstraint::parse("git+https://github.com/user/repo.git").unwrap();
    assert!(constraint.is_git());
    assert_eq!(constraint.get_git_url(), Some("https://github.com/user/repo.git"));

    let constraint = NpmVersionConstraint::parse("git://github.com/user/repo.git").unwrap();
    assert!(constraint.is_git());
}

#[test]
fn test_github_constraint() {
    let constraint = NpmVersionConstraint::parse("github:user/repo").unwrap();
    assert!(constraint.is_github());
    assert_eq!(constraint.get_github_repo(), Some("user/repo"));
}

#[test]
fn test_tag_constraint() {
    let constraint = NpmVersionConstraint::parse("latest").unwrap();
    assert!(constraint.is_tag());
    assert_eq!(constraint.get_tag(), Some("latest"));

    let constraint = NpmVersionConstraint::parse("next").unwrap();
    assert!(constraint.is_tag());
    assert_eq!(constraint.get_tag(), Some("next"));

    let constraint = NpmVersionConstraint::parse("beta").unwrap();
    assert!(constraint.is_tag());
    assert_eq!(constraint.get_tag(), Some("beta"));
}

#[test]
fn test_hyphen_range() {
    let constraint = NpmVersionConstraint::parse("1.0.0 - 2.0.0").unwrap();
    assert!(constraint.satisfies("1.0.0"));
    assert!(constraint.satisfies("1.5.0"));
    assert!(constraint.satisfies("2.0.0"));
    assert!(!constraint.satisfies("0.9.9"));
    assert!(!constraint.satisfies("2.0.1"));
}

#[test]
fn test_zero_version_caret() {
    let constraint = NpmVersionConstraint::parse("^0.2.3").unwrap();
    assert!(constraint.satisfies("0.2.3"));
    assert!(constraint.satisfies("0.2.4"));
    assert!(!constraint.satisfies("0.3.0"));
    assert!(!constraint.satisfies("1.0.0"));
}
