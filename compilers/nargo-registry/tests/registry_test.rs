use nargo_registry::{DependencyResolver, DependencySpec};

#[test]
fn test_parse_npm_dependency() {
    let spec = DependencyResolver::parse_dependency("lodash@4.17.21").unwrap();
    match spec {
        DependencySpec::Npm { name, version } => {
            assert_eq!(name, "lodash");
            assert_eq!(version, "4.17.21");
        }
        _ => panic!("Expected Npm dependency"),
    }
}

#[test]
fn test_parse_npm_dependency_latest() {
    let spec = DependencyResolver::parse_dependency("lodash").unwrap();
    match spec {
        DependencySpec::Npm { name, version } => {
            assert_eq!(name, "lodash");
            assert_eq!(version, "latest");
        }
        _ => panic!("Expected Npm dependency"),
    }
}

#[test]
fn test_parse_scoped_npm_dependency() {
    let spec = DependencyResolver::parse_dependency("@types/node@18.0.0").unwrap();
    match spec {
        DependencySpec::Npm { name, version } => {
            assert_eq!(name, "@types/node");
            assert_eq!(version, "18.0.0");
        }
        _ => panic!("Expected Npm dependency"),
    }
}

#[test]
fn test_parse_scoped_npm_dependency_no_version() {
    let spec = DependencyResolver::parse_dependency("@types/node").unwrap();
    match spec {
        DependencySpec::Npm { name, version } => {
            assert_eq!(name, "@types/node");
            assert_eq!(version, "latest");
        }
        _ => panic!("Expected Npm dependency"),
    }
}

#[test]
fn test_parse_git_dependency() {
    let spec = DependencyResolver::parse_dependency("git+https://github.com/user/repo.git").unwrap();
    match spec {
        DependencySpec::Git { url, .. } => {
            assert_eq!(url, "https://github.com/user/repo.git");
        }
        _ => panic!("Expected Git dependency"),
    }
}

#[test]
fn test_parse_github_dependency() {
    let spec = DependencyResolver::parse_dependency("github:user/repo").unwrap();
    match spec {
        DependencySpec::Github { repo, .. } => {
            assert_eq!(repo, "user/repo");
        }
        _ => panic!("Expected Github dependency"),
    }
}

#[test]
fn test_parse_path_dependency() {
    let spec = DependencyResolver::parse_dependency("file:./local-package").unwrap();
    match spec {
        DependencySpec::Path { path } => {
            assert_eq!(path, std::path::PathBuf::from("./local-package"));
        }
        _ => panic!("Expected Path dependency"),
    }
}
