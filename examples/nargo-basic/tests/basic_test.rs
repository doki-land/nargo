use nargo_basic::*;
use tempfile::tempdir;

#[test]
fn test_config_default() {
    let config = NargoConfig::default();
    assert!(config.name.is_none());
    assert!(config.version.is_none());
}

#[test]
fn test_config_update() {
    let mut config = NargoConfig::default();
    config.name = Some("test-app".to_string());
    config.version = Some("1.0.0".to_string());

    assert_eq!(config.name, Some("test-app".to_string()));
    assert_eq!(config.version, Some("1.0.0".to_string()));
}

#[test]
fn test_path_operations() {
    let temp_dir = tempdir().unwrap();
    let test_path = temp_dir.path().join("test.txt");

    assert!(!test_path.exists());

    std::fs::write(&test_path, "test content").unwrap();
    assert!(test_path.exists());
}
