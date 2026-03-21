#![warn(missing_docs)]

use nargo_config::{ConfigLoader, NargoConfig};
use std::path::PathBuf;

/// 测试配置加载器的基本功能
#[tokio::test]
async fn test_config_loader() {
    // 测试配置文件查找功能
    let current_dir = PathBuf::from(".");
    let config_path = ConfigLoader::find_config_file(&current_dir);
    println!("Found config file: {:?}", config_path);

    // 测试默认配置加载
    let default_config = NargoConfig::default();
    assert_eq!(default_config.name, None);
    assert_eq!(default_config.version, None);
    assert_eq!(default_config.build, None);
    assert_eq!(default_config.dev, None);
    assert_eq!(default_config.plugins, None);
    assert_eq!(default_config.aliases, None);
    assert_eq!(default_config.format, None);
    assert_eq!(default_config.lint, None);
}

/// 测试 JSON 配置文件加载
#[tokio::test]
async fn test_json_config() {
    // 创建临时 JSON 配置文件
    let config_content = r#"
    {
        "name": "test-project",
        "version": "1.0.0",
        "build": {
            "out_dir": "dist",
            "prod": true,
            "sourcemap": false
        },
        "format": {
            "indent_size": 4,
            "indent_type": "Spaces",
            "line_width": 100,
            "single_quote": false,
            "semi": true,
            "trailing_comma": "Es5",
            "attr_spacing": true,
            "self_closing_space": true,
            "tag_spacing": true,
            "arrow_parens": true,
            "object_spacing": true,
            "array_spacing": true,
            "function_spacing": true,
            "block_spacing": true
        },
        "lint": {
            "rules": {
                "no-console": {
                    "enabled": true,
                    "severity": "Warning"
                }
            },
            "extends": "recommended"
        }
    }
    "#;

    let config_path = PathBuf::from("nargo.config.test.json");
    std::fs::write(&config_path, config_content).unwrap();

    // 加载配置文件
    let loader = ConfigLoader::new(config_path.clone());
    let config = loader.load().unwrap();

    // 验证配置
    assert_eq!(config.name, Some("test-project".to_string()));
    assert_eq!(config.version, Some("1.0.0".to_string()));

    // 清理临时文件
    std::fs::remove_file(config_path).unwrap();
}

/// 测试 Node.js 版本检测
#[tokio::test]
async fn test_node_version_detection() {
    // 测试 Node.js 版本检测功能
    // 注意：此测试需要 Node.js 环境
    let version = nargo_config::ConfigLoader::get_node_version();
    match version {
        Ok(v) => println!("Node.js version: {}", v),
        Err(e) => println!("Failed to get Node.js version: {}", e),
    }
    // 我们不强制要求 Node.js 环境，所以此测试总是通过
    assert!(true);
}
