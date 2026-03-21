use crate::{ParseState, StyleParser};
use nargo_ir::Trivia;
use nargo_types::{Error, Result, Span};
use oak_core::{
    Language, Parser, SourceText,
    parser::ParseSession,
    tree::{RedTree, TypedNode},
};
use oak_css::{CssLanguage, CssParser};
use oak_scss::{ScssLanguage, ScssParser, ast::ScssRoot};
use oak_stylus::{StylusLanguage, StylusParser, ast::StylusRoot};

/// 基于 oak-css 的样式解析器
pub struct OakCssParser;

impl StyleParser for OakCssParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<(String, Trivia)> {
        // 首先解析源代码得到 AST 节点
        let source = SourceText::new(state.cursor.source.to_string());
        let language = CssLanguage::default();
        let parser = CssParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);

        // 处理解析错误
        if let Err(err) = parse_output.result {
            let error_message = format!("CSS parsing error: {}", err);
            return Err(Error::parse_error(error_message, state.cursor.span_at_current()));
        }

        // 返回原始样式代码，支持 Tailwind CSS 类名
        let code = state.cursor.source.to_string();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}

impl OakCssParser {
    /// 使用泛型语言类型解析样式
    fn parse_with_language<L: Language>(&self, _input: &L::TypedRoot) -> Result<(String, Trivia)> {
        // 返回空字符串和默认 trivia
        let code = String::new();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}

/// 基于 oak-scss 的样式解析器，支持 SCSS 和 Sass 语法
pub struct OakScssParser;

impl StyleParser for OakScssParser {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<(String, Trivia)> {
        // 首先解析源代码得到 AST 节点
        let source = SourceText::new(state.cursor.source.to_string());
        let language = ScssLanguage::default();
        let parser = ScssParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);

        // 处理解析错误
        if let Err(err) = parse_output.result {
            let error_message = format!("{} parsing error: {}", lang.to_uppercase(), err);
            return Err(Error::parse_error(error_message, state.cursor.span_at_current()));
        }

        // 返回原始样式代码
        let code = state.cursor.source.to_string();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}

impl OakScssParser {
    /// 使用泛型语言类型解析样式
    fn parse_with_language<L: Language>(&self, _input: &L::TypedRoot) -> Result<(String, Trivia)> {
        // 返回空字符串和默认 trivia
        let code = String::new();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}

/// 基于 oak-stylus 的样式解析器
pub struct OakStylusParser;

impl StyleParser for OakStylusParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<(String, Trivia)> {
        // 首先解析源代码得到 AST 节点
        let source = SourceText::new(state.cursor.source.to_string());
        let language = StylusLanguage::default();
        let parser = StylusParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);

        // 处理解析错误
        if let Err(err) = parse_output.result {
            let error_message = format!("Stylus parsing error: {}", err);
            return Err(Error::parse_error(error_message, state.cursor.span_at_current()));
        }

        // 返回原始样式代码
        let code = state.cursor.source.to_string();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}

impl OakStylusParser {
    /// 使用泛型语言类型解析样式
    fn parse_with_language<L: Language>(&self, _input: &L::TypedRoot) -> Result<(String, Trivia)> {
        // 返回空字符串和默认 trivia
        let code = String::new();
        let trivia = Trivia::default();

        Ok((code, trivia))
    }
}
