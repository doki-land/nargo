use nargo_resolver::resolver::*;
use std::collections::HashMap;

#[test]
fn test_resolve_options_default() {
    let options = ResolveOptions::default();
    assert!(options.include_dev);
    assert!(options.include_optional);
    assert!(!options.allow_prerelease);
    assert_eq!(options.max_depth, 100);
    assert!(options.parallel_jobs > 0);
}

#[tokio::test]
async fn test_resolver_basic() {
    let mut resolver = Resolver::new();
    let mut deps = HashMap::new();
    deps.insert("lodash".to_string(), nargo_config::Dependency::Version("4.17.21".to_string()));

    let result = resolver.resolve(&deps, &HashMap::new()).await.unwrap();
    assert_eq!(result.stats.total_packages, 1);
    assert!(result.conflicts.is_empty());
}

#[tokio::test]
async fn test_resolver_git_dependency() {
    let mut resolver = Resolver::new();
    let mut deps = HashMap::new();
    deps.insert("my-lib".to_string(), nargo_config::Dependency::Detailed(nargo_config::DependencyDetail { git: Some("https://github.com/user/lib.git".to_string()), tag: Some("v1.0.0".to_string()), ..Default::default() }));

    let result = resolver.resolve(&deps, &HashMap::new()).await.unwrap();
    assert_eq!(result.stats.git_deps, 1);
}

#[tokio::test]
async fn test_resolver_workspace_dependency() {
    let mut resolver = Resolver::new().with_workspace_packages(HashMap::from([("shared".to_string(), "1.0.0".to_string())]));

    let mut deps = HashMap::new();
    deps.insert("shared".to_string(), nargo_config::Dependency::Version("workspace:*".to_string()));

    let result = resolver.resolve(&deps, &HashMap::new()).await.unwrap();
    assert!(result.conflicts.is_empty());
}

#[test]
fn test_topological_sort() {
    let resolver = Resolver::new();
    let mut graph = DependencyGraph::new();

    let app = graph.add_node(DependencyNode::new("app", "1.0.0"));
    let lodash = graph.add_node(DependencyNode::new("lodash", "4.17.21"));
    let axios = graph.add_node(DependencyNode::new("axios", "1.6.0"));

    graph.add_edge(app, lodash, DependencyEdge::new("^4.0.0"));
    graph.add_edge(app, axios, DependencyEdge::new("^1.0.0"));

    let sorted = resolver.topological_sort(&graph).unwrap();
    assert_eq!(sorted.len(), 3);
}
