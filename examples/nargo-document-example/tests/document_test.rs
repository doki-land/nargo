use nargo_document_example::*;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(!config.title.is_empty());
    assert!(!config.description.is_empty());
}

#[test]
fn test_config_update() {
    let mut config = Config::default();
    config.title = "测试文档".to_string();
    config.description = "测试描述".to_string();

    assert_eq!(config.title, "测试文档");
    assert_eq!(config.description, "测试描述");
}

#[test]
fn test_directory_operations() {
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("docs");
    let output_dir = temp_dir.path().join("dist");

    std::fs::create_dir_all(&input_dir).unwrap();
    assert!(input_dir.exists());

    std::fs::write(input_dir.join("index.md"), "# 测试文档\n\n这是测试内容。").unwrap();

    assert!(input_dir.join("index.md").exists());
}
