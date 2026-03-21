//! Integration tests for verify module.

use nargo_lock::{LockEntry, LockFile, Verifier, VerifyResult};
use tempfile::tempdir;

#[test]
fn test_verify_result() {
    let mut result = VerifyResult::new();
    assert!(result.is_valid);

    result.add_missing("test@1.0.0".to_string());
    assert!(!result.is_valid);
    assert_eq!(result.missing.len(), 1);
}

#[test]
fn test_verifier() {
    let verifier = Verifier::new();
    let mut lock = LockFile::new();

    lock.add_package(LockEntry::new("test", "1.0.0").with_source("https://example.com/test.tgz").with_integrity("sha512-abc123"));

    let dir = tempdir().unwrap();
    let result = verifier.verify(&lock, dir.path()).unwrap();
    assert!(result.is_valid);
}

#[test]
fn test_lock_diff() {
    let mut old = LockFile::new();
    old.add_package(LockEntry::new("a", "1.0.0").with_integrity("sha512-aaa"));
    old.add_package(LockEntry::new("b", "1.0.0").with_integrity("sha512-bbb"));

    let mut new = LockFile::new();
    new.add_package(LockEntry::new("a", "1.0.0").with_integrity("sha512-aaa"));
    new.add_package(LockEntry::new("b", "1.0.0").with_integrity("sha512-bbb-changed"));
    new.add_package(LockEntry::new("c", "1.0.0").with_integrity("sha512-ccc"));

    let diff = Verifier::diff(&old, &new);
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.removed.len(), 0);
    assert_eq!(diff.changed.len(), 1);
}
