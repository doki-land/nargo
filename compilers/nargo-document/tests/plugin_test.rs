//! 插件测试
//!
//! 包含所有文档插件的测试用例

use nargo_document::plugin::{ContainerPlugin, DocumentPlugin, KaTeXPlugin, MermaidPlugin, PluginContext, PrismPlugin, ShikiPlugin};

/// 测试 Container 插件
#[test]
fn test_container_plugin() {
    let plugin = ContainerPlugin::new();
    let input = r#"Some text
:::tip
This is a tip message
:::
More text"#;
    let output = plugin.process_containers(input);
    assert!(output.contains("<div class=\"container-tip\">"));
    assert!(output.contains("<div class=\"container-content\">This is a tip message</div>"));
}

#[test]
fn test_container_plugin_multiple() {
    let plugin = ContainerPlugin::new();
    let input = r#":::warning
Warning message
:::

:::danger
Danger message
:::"#;
    let output = plugin.process_containers(input);
    assert!(output.contains("<div class=\"container-warning\">"));
    assert!(output.contains("<div class=\"container-danger\">"));
}

#[test]
fn test_container_plugin_before_render() {
    let plugin = ContainerPlugin::new();
    let context = PluginContext::from_content(
        r#":::info
Info message
:::"#
            .to_string(),
        "test.md".to_string(),
    );
    let result = plugin.before_render(context);
    assert!(result.content.contains("<div class=\"container-info\">"));
}

/// 测试 KaTeX 插件
#[test]
fn test_katex_plugin() {
    let plugin = KaTeXPlugin::new();
    let input = r"Some text
\(E = mc^2\)
More text";
    let output = plugin.process_katex(input);
    assert!(output.contains("<span class=\"katex\">"));
}

#[test]
fn test_katex_plugin_block() {
    let plugin = KaTeXPlugin::new();
    let input = r"Some text
\[\int_{a}^{b} f(x) dx\]
More text";
    let output = plugin.process_katex(input);
    assert!(output.contains("<div class=\"katex-display\">"));
}

#[test]
fn test_katex_plugin_before_render() {
    let plugin = KaTeXPlugin::new();
    let context = PluginContext::from_content(r"\(E = mc^2\)".to_string(), "test.md".to_string());
    let result = plugin.before_render(context);
    assert!(result.content.contains("<span class=\"katex\">"));
}

/// 测试 Mermaid 插件
#[test]
fn test_mermaid_plugin() {
    let plugin = MermaidPlugin::new();
    let input = r"Some text
```mermaid
graph TD
    A --> B
    B --> C
```
More text";
    let output = plugin.process_mermaid(input);
    assert!(output.contains("<div class=\"mermaid\">"));
    assert!(output.contains("graph TD"));
}

#[test]
fn test_mermaid_plugin_before_render() {
    let plugin = MermaidPlugin::new();
    let context = PluginContext::from_content(
        r"```mermaid
graph TD
    A --> B
```"
        .to_string(),
        "test.md".to_string(),
    );
    let result = plugin.before_render(context);
    assert!(result.content.contains("<div class=\"mermaid\">"));
}

/// 测试 Prism 插件
#[test]
fn test_prism_plugin() {
    let plugin = PrismPlugin::new();
    let input = r#"Some text
```rust
fn main() {
    println!("Hello, world!");
}
```
More text"#;
    let output = plugin.process_code_blocks(input);
    assert!(output.contains("<pre class=\"language-rust\">"));
    assert!(output.contains("<code class=\"language-rust\">"));
}

#[test]
fn test_prism_plugin_before_render() {
    let plugin = PrismPlugin::new();
    let context = PluginContext::from_content(
        r"```rust
fn main() {}
```"
        .to_string(),
        "test.md".to_string(),
    );
    let result = plugin.before_render(context);
    assert!(result.content.contains("<pre class=\"language-rust\">"));
}

/// 测试 Shiki 插件
#[test]
fn test_shiki_plugin() {
    let plugin = ShikiPlugin::new();
    let input = r#"Some text
```rust
fn main() {
    println!("Hello, world!");
}
```
More text"#;
    let output = plugin.process_code_blocks(input);
    assert!(output.contains("<pre class=\"shiki\">"));
    assert!(output.contains("<code class=\"language-rust\">"));
}

#[test]
fn test_shiki_plugin_before_render() {
    let plugin = ShikiPlugin::new();
    let context = PluginContext::from_content(
        r"```rust
fn main() {}
```"
        .to_string(),
        "test.md".to_string(),
    );
    let result = plugin.before_render(context);
    assert!(result.content.contains("<pre class=\"shiki\">"));
}
