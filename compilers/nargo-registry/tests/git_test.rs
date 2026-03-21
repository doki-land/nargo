use nargo_registry::{DependencySpec, GitReference, GitResolver};

#[test]
fn test_parse_git_url_basic() {
    let spec = GitResolver::parse_git_url("git+https://github.com/user/repo.git").unwrap();
    match spec {
        DependencySpec::Git { url, reference } => {
            assert_eq!(url, "https://github.com/user/repo.git");
            assert!(reference.is_none());
        }
        _ => panic!("Expected Git dependency"),
    }
}

#[test]
fn test_parse_git_url_with_tag() {
    let spec = GitResolver::parse_git_url("git+https://github.com/user/repo.git#v1.0.0").unwrap();
    match spec {
        DependencySpec::Git { url, reference } => {
            assert_eq!(url, "https://github.com/user/repo.git");
            assert!(matches!(reference, Some(GitReference::Tag(t)) if t == "v1.0.0"));
        }
        _ => panic!("Expected Git dependency"),
    }
}

#[test]
fn test_parse_git_url_with_branch() {
    let spec = GitResolver::parse_git_url("git+https://github.com/user/repo.git#develop").unwrap();
    match spec {
        DependencySpec::Git { url, reference } => {
            assert_eq!(url, "https://github.com/user/repo.git");
            assert!(matches!(reference, Some(GitReference::Branch(b)) if b == "develop"));
        }
        _ => panic!("Expected Git dependency"),
    }
}

#[test]
fn test_parse_github_shorthand() {
    let spec = GitResolver::parse_git_url("github:user/repo").unwrap();
    match spec {
        DependencySpec::Github { repo, reference } => {
            assert_eq!(repo, "user/repo");
            assert!(reference.is_none());
        }
        _ => panic!("Expected Github dependency"),
    }
}

#[test]
fn test_parse_github_shorthand_with_ref() {
    let spec = GitResolver::parse_git_url("github:user/repo#v2.0.0").unwrap();
    match spec {
        DependencySpec::Github { repo, reference } => {
            assert_eq!(repo, "user/repo");
            assert!(matches!(reference, Some(GitReference::Tag(t)) if t == "v2.0.0"));
        }
        _ => panic!("Expected Github dependency"),
    }
}
