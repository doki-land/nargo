use nargo_parser::{OakCssParser, OakTypeScriptParser, OakVueTemplateParser, ParseState, ScriptParser, StyleParser, TemplateParser};
use nargo_types::{Cursor, Position, Span};
#[test]
fn test_script_parser() {
    let source = "console.log('Hello, World!');".to_string();
    let mut state = ParseState { cursor: Cursor::new(source.as_str(), Span::default()) };
    let parser = OakTypeScriptParser;
    let result = parser.parse(&mut state, "ts");
    assert!(result.is_ok());
}
#[test]
fn test_template_parser() {
    let source = r#"<template>  <div>Hello, World!</div></template>"#.to_string();
    let mut state = ParseState { cursor: Cursor::new(source.as_str(), Span::default()) };
    let parser = OakVueTemplateParser;
    let result = parser.parse(&mut state, "vue");
    assert!(result.is_ok());
}
#[test]
fn test_style_parser() {
    let source = r#"div { color: red; }"#.to_string();
    let mut state = ParseState { cursor: Cursor::new(source.as_str(), Span::default()) };
    let parser = OakCssParser;
    let result = parser.parse(&mut state, "css");
    assert!(result.is_ok());
}
