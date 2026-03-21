use nargo_registry::client::*;
use std::collections::HashMap;

#[test]
fn test_registry_config_default() {
    let config = RegistryConfig::default();
    assert_eq!(config.registry_url, "https://registry.npmjs.org");
    assert_eq!(config.timeout_secs, 30);
}

#[test]
fn test_client_creation() {
    let client = RegistryClient::new();
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_metadata_cache() {
    let mut cache = MetadataCache::new(60);
    let meta = PackageMetadata { name: "test".to_string(), description: None, versions: HashMap::new(), dist_tags: HashMap::new(), time: HashMap::new(), latest: None };

    cache.insert("test-key".to_string(), meta.clone());
    assert!(cache.get("test-key").is_some());
}
