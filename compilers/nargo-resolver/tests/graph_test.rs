//! Integration tests for dependency graph.

use nargo_resolver::{DependencyEdge, DependencyGraph, DependencyNode};

#[test]
fn test_add_nodes() {
    let mut graph = DependencyGraph::new();
    let node1 = graph.add_node(DependencyNode::new("lodash", "4.17.21"));
    let node2 = graph.add_node(DependencyNode::new("axios", "1.6.0"));

    assert_eq!(graph.node_count(), 2);
    assert!(graph.get_node_by_name("lodash").is_some());
    assert!(graph.get_node_by_name("axios").is_some());
}

#[test]
fn test_add_edges() {
    let mut graph = DependencyGraph::new();
    let n1 = graph.add_node(DependencyNode::new("app", "1.0.0"));
    let n2 = graph.add_node(DependencyNode::new("lodash", "4.17.21"));

    graph.add_edge(n1, n2, DependencyEdge::new("^4.0.0"));

    assert_eq!(graph.edge_count(), 1);
    let deps = graph.dependencies_of(n1);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0.name, "lodash");
}

#[test]
fn test_cycle_detection() {
    let mut graph = DependencyGraph::new();
    let a = graph.add_node(DependencyNode::new("a", "1.0.0"));
    let b = graph.add_node(DependencyNode::new("b", "1.0.0"));
    let c = graph.add_node(DependencyNode::new("c", "1.0.0"));

    graph.add_edge(a, b, DependencyEdge::new("1.0.0"));
    graph.add_edge(b, c, DependencyEdge::new("1.0.0"));
    graph.add_edge(c, a, DependencyEdge::new("1.0.0"));

    assert!(graph.has_cycle());
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_topological_sort() {
    let mut graph = DependencyGraph::new();
    let app = graph.add_node(DependencyNode::new("app", "1.0.0"));
    let lodash = graph.add_node(DependencyNode::new("lodash", "4.17.21"));
    let axios = graph.add_node(DependencyNode::new("axios", "1.6.0"));

    graph.add_edge(app, lodash, DependencyEdge::new("^4.0.0"));
    graph.add_edge(app, axios, DependencyEdge::new("^1.0.0"));

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);

    let names: Vec<&str> = sorted.iter().map(|n| n.name.as_str()).collect();
    let app_pos = names.iter().position(|&n| n == "app");
    let lodash_pos = names.iter().position(|&n| n == "lodash");
    assert!(app_pos.is_some() && lodash_pos.is_some());
    let app_pos = app_pos.unwrap();
    let lodash_pos = lodash_pos.unwrap();
    assert!(app_pos < lodash_pos, "in toposort, dependent (app) comes before dependency (lodash)");
}
