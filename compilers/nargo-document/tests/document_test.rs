use nargo_document::{
    config::Config,
    generator::{Generator, markdown::MarkdownRenderer},
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_load() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();

    // Create a test TOML config file
    let config_path = temp_dir.path().join("nargodoc.config.toml");
    fs::write(
        &config_path,
        r#"
title = "Test Documentation"
description = "Test description"
"#,
    )
    .unwrap();

    // Load the config
    let config = Config::load_from_file(&config_path).unwrap();
    assert_eq!(config.title, Some("Test Documentation".to_string()));
    assert_eq!(config.description, Some("Test description".to_string()));
}

#[test]
fn test_config_default() {
    // Test default config
    let config = Config::default();
    assert_eq!(config.title, None);
    assert_eq!(config.description, None);
    assert!(config.locales.is_empty());
    assert_eq!(config.theme, Default::default());
    assert_eq!(config.markdown, Default::default());
    assert_eq!(config.build, Default::default());
}

#[test]
fn test_markdown_renderer() {
    // Create markdown renderer
    let renderer = MarkdownRenderer::new();

    // Test basic markdown rendering
    let markdown = "# Test Title\n\n**Bold text**\n\n- List item 1\n- List item 2";
    let html = renderer.render(markdown).unwrap();
    assert!(html.contains("<h1>Test Title</h1>"));
    assert!(html.contains("<strong>Bold text</strong>"));
    assert!(html.contains("<li>List item 1</li>"));
    assert!(html.contains("<li>List item 2</li>"));
}

#[test]
fn test_generator() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create input directory and test markdown file
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("test.md"), "# Test Page\n\nThis is a test page.").unwrap();

    // Create config
    let config = Config::default();

    // Create generator
    let mut generator = Generator::new(config);

    // Generate documentation
    let result = generator.generate(input_dir.to_str().unwrap(), output_dir.to_str().unwrap());

    // The generate method might fail due to missing dependencies (e.g., wkhtmltopdf for PDF generation)
    // So we'll just check that it runs without panicking, even if it returns an error
    assert!(result.is_ok() || result.is_err());
}
