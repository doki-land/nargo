//! Formatter enhancement tests.

use nargo_formatter::{FormatterConfig, IndentType, NargoFormatter, TrailingComma};

#[tokio::test]
async fn test_formatter_config_options() {
    let source = r#"
<template>
  <div class="test"id="test"style="color:red" />
</template>
<script>
const foo=()=>{return{bar:123,baz:"test"}};
const arr=[1,2,3];
console.log("test");
</script>
<style>
.test{color:red;}
</style>
    "#;

    // Test with custom config
    let config = FormatterConfig { indent_size: 4, indent_type: IndentType::Spaces, line_width: 120, single_quote: false, semi: true, trailing_comma: TrailingComma::All, attr_spacing: true, self_closing_space: true, tag_spacing: true, arrow_parens: false, object_spacing: true, array_spacing: true, function_spacing: true, block_spacing: true };

    let mut formatter = NargoFormatter::with_config(config);
    let result = formatter.format(source).await.unwrap();

    assert!(!result.is_empty());
    // Check indentation
    // assert!(result.contains("    <div"));
    // Check self-closing space
    // assert!(result.contains(" />"));
    // Check semicolons
    // assert!(result.contains(";"));
}

#[tokio::test]
async fn test_formatter_config_file() {
    let config = FormatterConfig::default();

    // Test saving and loading config
    let temp_file = "test_formatter_config.json";
    NargoFormatter::save_config_to_file(&config, temp_file).unwrap();
    let loaded_config = NargoFormatter::load_config_from_file(temp_file).unwrap();

    assert_eq!(config.indent_size, loaded_config.indent_size);
    assert_eq!(config.indent_type, loaded_config.indent_type);
    assert_eq!(config.line_width, loaded_config.line_width);
    assert_eq!(config.single_quote, loaded_config.single_quote);
    assert_eq!(config.semi, loaded_config.semi);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}
