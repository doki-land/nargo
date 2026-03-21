//! Integration tests for conflict detection.

use nargo_resolver::{ConflictDetector, ConflictType};
use std::collections::HashMap;

#[test]
fn test_version_conflict_detection() {
    let mut detector = ConflictDetector::new();
    let mut deps = HashMap::new();
    deps.insert("lodash".to_string(), vec![("4.17.21".to_string(), "app".to_string()), ("3.10.1".to_string(), "old-lib".to_string())]);

    let conflicts = detector.detect_version_conflicts(&deps);
    assert!(!conflicts.is_empty());
    assert_eq!(conflicts[0].conflict_type, ConflictType::VersionConflict);
}

#[test]
fn test_peer_conflict_detection() {
    let mut detector = ConflictDetector::new();
    let mut peer_deps = HashMap::new();
    peer_deps.insert("react".to_string(), ("18.0.0".to_string(), Some("my-ui-lib".to_string())));

    let mut resolved = HashMap::new();
    resolved.insert("react".to_string(), "17.0.2".to_string());

    let conflicts = detector.detect_peer_conflicts(&peer_deps, &resolved);
    assert!(!conflicts.is_empty());
    assert_eq!(conflicts[0].conflict_type, ConflictType::PeerDependencyConflict);
}

#[test]
fn test_find_solutions() {
    let mut detector = ConflictDetector::new();
    let mut deps = HashMap::new();
    deps.insert("lodash".to_string(), vec![("4.17.21".to_string(), "app".to_string()), ("3.10.1".to_string(), "old-lib".to_string())]);

    detector.detect_version_conflicts(&deps);
    let solutions = detector.find_solutions();
    assert!(!solutions.is_empty());
    assert!(solutions[0].is_automatic);
}
