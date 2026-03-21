//! Integration tests for migration utilities.

use nargo_config::*;
use std::collections::HashMap;

#[test]
fn test_parse_package_json() {
    let json = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "description": "A test package",
            "author": "Test Author",
            "license": "MIT",
            "dependencies": {
                "lodash": "^4.17.21",
                "axios": "1.6.0"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            },
            "scripts": {
                "build": "tsc",
                "test": "jest"
            }
        }"#;

    let pkg: PackageJson = serde_json::from_str(json).unwrap();
    assert_eq!(pkg.name, Some("test-package".to_string()));
    assert_eq!(pkg.version, Some("1.0.0".to_string()));
    assert_eq!(pkg.dependencies.len(), 2);
    assert_eq!(pkg.dev_dependencies.len(), 1);
}

#[test]
fn test_convert_to_nargo_toml() {
    let migrator = Migrator::new();
    let pkg = PackageJson {
        name: Some("my-app".to_string()),
        version: Some("2.0.0".to_string()),
        description: Some("My application".to_string()),
        author: Some("Developer".to_string()),
        license: Some("MIT".to_string()),
        repository: Some(RepositoryField::Url("https://github.com/user/repo".to_string())),
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("vue".to_string(), "^3.4.0".to_string());
            deps
        },
        dev_dependencies: {
            let mut deps = HashMap::new();
            deps.insert("vite".to_string(), "^5.0.0".to_string());
            deps
        },
        ..Default::default()
    };

    let nargo = migrator.convert(&pkg).unwrap();
    assert_eq!(nargo.package.name, "my-app");
    assert_eq!(nargo.package.version, "2.0.0");
    assert!(nargo.dependencies.contains_key("vue"));
    assert!(nargo.dev_dependencies.contains_key("vite"));
}

#[test]
fn test_git_dependency_conversion() {
    let migrator = Migrator::new();
    let pkg = PackageJson {
        name: Some("test".to_string()),
        version: Some("1.0.0".to_string()),
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("my-lib".to_string(), "git+https://github.com/user/lib.git#v1.0.0".to_string());
            deps
        },
        ..Default::default()
    };

    let nargo = migrator.convert(&pkg).unwrap();
    let dep = &nargo.dependencies["my-lib"];
    assert!(matches!(dep, Dependency::Detailed(d) if d.git.is_some()));
}
