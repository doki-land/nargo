use nargo_document::{Config, Generator};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_generate_document() {
    // 创建临时目录
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // 创建输入目录
    fs::create_dir_all(&input_dir).expect("Failed to create input dir");

    // 创建测试 Markdown 文件
    let test_content = r#"
# Test Document

This is a test document for HXO Document generator.

## Features

- Markdown support
- HTML generation
- PDF generation
- Template system

```rust
fn main() {
    println!("Hello, World!");
}
```

```mermaid
graph TD
    A[Markdown] --> B[HTML]
    A --> C[PDF]
    B --> D[Template]
```
"#;

    fs::write(input_dir.join("test.md"), test_content).expect("Failed to write test.md");

    // 创建配置
    let config = Config::default();

    // 创建生成器
    let mut generator = Generator::new(config);

    // 生成文档
    generator.generate(input_dir.to_str().unwrap(), output_dir.to_str().unwrap()).expect("Failed to generate document");

    // 检查生成的文件
    let html_path = output_dir.join("test.html");
    let index_html_path = output_dir.join("index.html");

    assert!(html_path.exists(), "HTML file not generated");
    assert!(index_html_path.exists(), "Index HTML file not generated");

    // 检查 HTML 内容
    let html_content = fs::read_to_string(html_path).expect("Failed to read HTML file");
    assert!(html_content.contains("Test Document"), "HTML content not correct");
    assert!(html_content.contains("Hello, World!"), "Code block not rendered");
}
