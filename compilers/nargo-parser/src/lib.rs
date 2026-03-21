#![feature(new_range_api)]
#![warn(missing_docs)]
use nargo_ir::IRModule;
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

mod base;
mod oak_script_parser;
mod oak_style_parser;
mod oak_template_parser;
mod registry;
pub mod shell;
pub mod template;
pub mod vutex;

pub use base::ParseState;
pub use nargo_types::Cursor;
pub use oak_script_parser::OakTypeScriptParser;
pub use oak_style_parser::{OakCssParser, OakScssParser, OakStylusParser};
pub use oak_template_parser::OakVueTemplateParser;
pub use registry::{MetadataParser, ParserRegistry, ScriptParser, StyleParser, TemplateParser};
pub use shell::VocShell as Parser;

pub use vutex::{
    Document, DocumentMeta, FrontMatter, FrontMatterParser, MarkdownParser,
    document::{parse_document, parse_frontmatter},
};

/// 解析 Vmz 格式的文件
///
/// # Arguments
///
/// * `source` - Vmz 源文件内容
///
/// # Returns
///
/// 解析结果，包含模板、脚本和样式部分
pub fn parse(source: &str) -> nargo_types::Result<IRModule> {
    let registry = std::sync::Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("anonymous.tsx".to_string(), source, registry);
    parser.parse_all()
}

pub fn parse_i18n(source: &str, lang: &str) -> nargo_types::Result<HashMap<String, NargoValue>> {
    let registry = std::sync::Arc::new(ParserRegistry::new());
    if let Some(parser) = registry.get_metadata_parser(lang) {
        let mut state = ParseState::new(source);
        let (val, _trivia) = parser.parse(&mut state, lang)?;
        if let NargoValue::Object(map) = val {
            return Ok(map);
        }
    }
    Err(nargo_types::Error::parse_error(format!("Unsupported i18n lang: {}", lang), Span::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;
        let result = parse(source);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.template.is_some());
    }

    #[test]
    fn test_parse_with_script() {
        let source = r#"
<template>
  <div>{{ message }}</div>
</template>
<script>
const message = 'Hello';
</script>
"#;
        let result = parse(source);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.template.is_some());
        assert!(module.script.is_some());
    }

    #[test]
    fn test_parse_with_style() {
        let source = r#"
<template>
  <div>Hello World</div>
</template>
<style>
div {
  color: red;
}
</style>
"#;
        let result = parse(source);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.template.is_some());
        assert!(!module.styles.is_empty());
    }

    #[test]
    fn test_parse_i18n() {
        let source = r#"{
  "hello": "Hello",
  "world": "World"
}"#;
        let result = parse_i18n(source, "json");
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(!map.is_empty());
    }

    #[test]
    fn test_parse_i18n_unsupported_lang() {
        let source = r#"{
  "hello": "Hello"
}"#;
        let result = parse_i18n(source, "unsupported");
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_creation() {
        let registry = std::sync::Arc::new(ParserRegistry::new());
        let mut parser = Parser::new("test.tsx".to_string(), "<template></template>", registry);
        // 测试解析功能，确保解析器能正常工作
        let result = parser.parse_all();
        assert!(result.is_ok());
    }
}
