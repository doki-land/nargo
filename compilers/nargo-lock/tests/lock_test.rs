//! Integration tests for lock module.

use nargo_lock::{LockEntry, LockFile};
use tempfile::tempdir;

#[test]
fn test_lock_entry() {
    let entry = LockEntry::new("lodash", "4.17.21").with_source("https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz").with_integrity("sha512-abc123");

    assert_eq!(entry.name, "lodash");
    assert_eq!(entry.version, "4.17.21");
    assert_eq!(entry.key(), "lodash@4.17.21");
}

#[test]
fn test_lock_file_operations() {
    let mut lock = LockFile::new();
    let entry = LockEntry::new("vue", "3.4.0").with_integrity("sha512-xyz789");

    lock.add_package(entry);
    assert!(lock.has_package("vue", "3.4.0"));
    assert_eq!(lock.len(), 1);

    lock.remove_package("vue", "3.4.0");
    assert!(!lock.has_package("vue", "3.4.0"));
}

#[test]
fn test_lock_file_save_load() {
    let dir = tempdir().unwrap();
    let lock_path = dir.path().join("nargo.lock");

    let mut lock = LockFile::new();
    lock.add_package(LockEntry::new("react", "18.2.0").with_integrity("sha512-test123"));

    lock.save(&lock_path).unwrap();
    assert!(lock_path.exists());

    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert!(loaded.has_package("react", "18.2.0"));
}
