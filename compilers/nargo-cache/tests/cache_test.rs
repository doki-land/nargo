//! Integration tests for cache module.

use nargo_cache::{Cache, CacheEntry};
use tempfile::tempdir;

#[test]
fn test_cache_entry() {
    let entry = CacheEntry::new("lodash", "4.17.21").with_size(1000).with_source("https://registry.npmjs.org");

    assert_eq!(entry.name, "lodash");
    assert_eq!(entry.version, "4.17.21");
    assert_eq!(entry.size, 1000);
    assert_eq!(entry.key(), "lodash@4.17.21");
}

#[tokio::test]
async fn test_cache_operations() {
    let dir = tempdir().unwrap();
    let mut cache = Cache::with_root(dir.path().to_path_buf()).unwrap();

    let tarball = b"fake tarball content";
    let entry = cache.add("test-pkg", "1.0.0", tarball, "https://registry.npmjs.org").await.unwrap();

    assert!(cache.has("test-pkg", "1.0.0"));
    assert_eq!(entry.size, tarball.len() as u64);

    let retrieved = cache.get_tarball("test-pkg", "1.0.0").await.unwrap();
    assert_eq!(retrieved, tarball);

    cache.remove("test-pkg", "1.0.0").await.unwrap();
    assert!(!cache.has("test-pkg", "1.0.0"));
}

#[test]
fn test_cache_stats() {
    let dir = tempdir().unwrap();
    let cache = Cache::with_root(dir.path().to_path_buf()).unwrap();
    let stats = cache.stats();
    assert_eq!(stats.total_packages, 0);
}
