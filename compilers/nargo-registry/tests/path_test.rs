use nargo_registry::{DependencySpec, PathResolver, PathUtils};
use std::path::PathBuf;

#[test]
fn test_parse_path_spec_file_prefix() {
    let spec = PathResolver::parse_path_spec("file:./packages/my-package").unwrap();
    match spec {
        DependencySpec::Path { path } => {
            assert_eq!(path, PathBuf::from("./packages/my-package"));
        }
        _ => panic!("Expected Path dependency"),
    }
}

#[test]
fn test_parse_path_spec_relative() {
    let spec = PathResolver::parse_path_spec("../shared/utils").unwrap();
    match spec {
        DependencySpec::Path { path } => {
            assert_eq!(path, PathBuf::from("../shared/utils"));
        }
        _ => panic!("Expected Path dependency"),
    }
}

#[test]
fn test_is_path_dependency() {
    assert!(PathUtils::is_path_dependency("file:./local"));
    assert!(PathUtils::is_path_dependency("./local"));
    assert!(PathUtils::is_path_dependency("../local"));
    assert!(!PathUtils::is_path_dependency("lodash"));
    assert!(!PathUtils::is_path_dependency("git+https://github.com/user/repo"));
}

#[test]
fn test_normalize_path() {
    assert_eq!(PathUtils::normalize_path("file:.\\packages\\my-package"), "./packages/my-package");
    assert_eq!(PathUtils::normalize_path("./packages/my-package"), "./packages/my-package");
}
