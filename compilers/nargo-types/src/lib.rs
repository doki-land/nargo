#![warn(missing_docs)]

pub mod context;
pub mod document;
pub mod errors;
pub mod paths;

pub use context::NargoContext;
pub use document::{Document, DocumentMeta, FrontMatter};
pub use errors::{Error, ErrorKind, Result};
pub use paths::{cache_dir, CACHE_DIR_REL};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, hash::Hash};

/// 源代码中一段文本的位置范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// 起始位置
    pub start: Position,
    /// 结束位置
    pub end: Position,
}

impl Span {
    /// 创建一个新的 Span
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// 仅从字节偏移量创建位置范围（行列信息暂设为 0）
    pub fn from_offsets(start: usize, end: usize) -> Self {
        Self { start: Position::from_offset(start), end: Position::from_offset(end) }
    }

    /// 创建一个表示未知位置的 Span
    pub fn unknown() -> Self {
        Self { start: Position::unknown(), end: Position::unknown() }
    }

    /// 判断位置范围是否为未知
    pub fn is_unknown(&self) -> bool {
        self.start.is_unknown() && self.end.is_unknown()
    }

    /// 合并多个 Span，取其最大范围
    pub fn merge(spans: &[Span]) -> Self {
        if spans.is_empty() {
            return Self::unknown();
        }
        let mut start = spans[0].start;
        let mut end = spans[0].end;

        for span in &spans[1..] {
            if span.is_unknown() {
                continue;
            }
            if start.is_unknown() || (span.start.offset < start.offset) {
                start = span.start;
            }
            if end.is_unknown() || (span.end.offset > end.offset) {
                end = span.end;
            }
        }
        Self { start, end }
    }
}

/// 源代码中的具体位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    /// 行号（从 1 开始）
    pub line: u32,
    /// 列号（从 1 开始，按 UTF-16 字符计数）
    pub column: u32,
    /// 字节偏移量（从 0 开始）
    pub offset: u32,
}

impl Position {
    /// 创建一个新的 Position
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self { line, column, offset }
    }

    /// 仅从字节偏移量创建位置（行列信息暂设为 0）
    pub fn from_offset(offset: usize) -> Self {
        Self { line: 0, column: 0, offset: offset as u32 }
    }

    /// 创建一个表示未知位置的 Position
    pub fn unknown() -> Self {
        Self { line: 0, column: 0, offset: 0 }
    }

    /// 判断位置是否为未知
    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}

/// Nargo 文件结构
#[derive(Debug, Clone)]
pub struct NargoFile {
    /// 文件中的所有块
    pub blocks: Vec<NargoBlock>,
    /// 文件的整体位置范围
    pub span: Span,
}

/// Nargo 文件中的代码块
#[derive(Debug, Clone)]
pub struct NargoBlock {
    /// 块名称
    pub name: String,
    /// 块属性
    pub attributes: HashMap<String, String>,
    /// 块内容
    pub content: String,
    /// 块的位置范围
    pub span: Span,
    /// 块内容的位置范围
    pub content_span: Span,
}

/// 编译模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileMode {
    /// Vue 2 兼容模式
    Vue2,
    /// Vue 3 模式
    Vue,
}

impl Default for CompileMode {
    fn default() -> Self {
        Self::Vue
    }
}

/// 编译选项
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// 编译模式
    pub mode: CompileMode,
    /// 是否启用服务端渲染
    pub ssr: bool,
    /// 是否启用水合
    pub hydrate: bool,
    /// 是否压缩代码
    pub minify: bool,
    /// 是否为生产环境
    pub is_prod: bool,
    /// 目标平台
    pub target: Option<String>,
    /// 作用域 ID
    pub scope_id: Option<String>,
    /// i18n 语言
    pub i18n_locale: Option<String>,
    /// Vue 响应式转换
    pub vue_reactivity_transform: bool,
    /// Vue define_model
    pub vue_define_model: bool,
    /// Vue Props 解构
    pub vue_props_destructure: bool,
}

/// Nargo 值类型
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum NargoValue {
    /// Null 值
    #[default]
    Null,
    /// 布尔值
    Bool(bool),
    /// 数字
    Number(f64),
    /// 字符串
    String(String),
    /// 数组
    Array(Vec<NargoValue>),
    /// 对象
    Object(HashMap<String, NargoValue>),
    /// 响应式信号引用
    Signal(String),
    /// 二进制数据 (如 WASM)
    Binary(Vec<u8>),
    /// 源码片段或表达式代码
    Raw(String),
    /// 跨节点引用或 ID
    Ref(String),
}

impl NargoValue {
    /// 获取对象中的值
    pub fn get(&self, key: &str) -> Option<&NargoValue> {
        match self {
            NargoValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// 转换为数组引用
    pub fn as_array(&self) -> Option<&Vec<NargoValue>> {
        match self {
            NargoValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// 检查值是否为 null
    pub fn is_null(&self) -> bool {
        matches!(self, NargoValue::Null)
    }

    /// 验证值的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        const MAX_RECURSION_DEPTH: usize = 100;
        const MAX_STRING_LENGTH: usize = 1024 * 1024;
        const MAX_ARRAY_LENGTH: usize = 10000;
        const MAX_OBJECT_SIZE: usize = 1000;

        if depth > MAX_RECURSION_DEPTH {
            return Err(Error::external_error("nargo-value".to_string(), "Value recursion depth exceeded".to_string(), Span::unknown()));
        }

        match self {
            NargoValue::Null => Ok(()),
            NargoValue::Bool(_) => Ok(()),
            NargoValue::Number(_) => Ok(()),
            NargoValue::String(s) => {
                if s.len() > MAX_STRING_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "String length exceeded".to_string(), Span::unknown()));
                }
                Ok(())
            }
            NargoValue::Array(arr) => {
                if arr.len() > MAX_ARRAY_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "Array length exceeded".to_string(), Span::unknown()));
                }
                for item in arr {
                    item.validate(depth + 1)?;
                }
                Ok(())
            }
            NargoValue::Object(map) => {
                if map.len() > MAX_OBJECT_SIZE {
                    return Err(Error::external_error("nargo-value".to_string(), "Object size exceeded".to_string(), Span::unknown()));
                }
                for (key, value) in map {
                    if key.len() > MAX_STRING_LENGTH {
                        return Err(Error::external_error("nargo-value".to_string(), "Object key length exceeded".to_string(), Span::unknown()));
                    }
                    value.validate(depth + 1)?;
                }
                Ok(())
            }
            NargoValue::Signal(s) => {
                if s.len() > MAX_STRING_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "Signal length exceeded".to_string(), Span::unknown()));
                }
                Ok(())
            }
            NargoValue::Binary(b) => {
                if b.len() > MAX_STRING_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "Binary data length exceeded".to_string(), Span::unknown()));
                }
                Ok(())
            }
            NargoValue::Raw(s) => {
                if s.len() > MAX_STRING_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "Raw data length exceeded".to_string(), Span::unknown()));
                }
                Ok(())
            }
            NargoValue::Ref(s) => {
                if s.len() > MAX_STRING_LENGTH {
                    return Err(Error::external_error("nargo-value".to_string(), "Ref length exceeded".to_string(), Span::unknown()));
                }
                Ok(())
            }
        }
    }
}

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// 路由列表
    pub routes: Vec<Route>,
    /// 路由模式
    pub mode: String,
    /// 基础路径
    pub base: Option<String>,
    /// 配置的位置范围
    pub span: Span,
}

/// 单个路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// 路由路径
    pub path: String,
    /// 关联组件
    pub component: String,
    /// 路由名称
    pub name: Option<String>,
    /// 重定向路径
    pub redirect: Option<String>,
    /// 子路由
    pub children: Option<Vec<Route>>,
    /// 路由元数据
    pub meta: Option<NargoValue>,
    /// 路由的位置范围
    pub span: Span,
}

/// 判断 HTML 标签是否为自闭合标签
pub fn is_void_element(tag: &str) -> bool {
    matches!(tag, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

/// 判断位置是否在指定的范围内
pub fn is_pos_in_span(pos: Position, span: Span) -> bool {
    if span.is_unknown() {
        return false;
    }

    if pos.line < span.start.line || pos.line > span.end.line {
        return false;
    }

    if pos.line == span.start.line && pos.column < span.start.column {
        return false;
    }

    if pos.line == span.end.line && pos.column > span.end.column {
        return false;
    }

    true
}

/// 判断字符是否为字母或下划线或美元符号
pub fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

/// 判断字符是否为字母数字或下划线或美元符号
pub fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// 用于 TypeScript 生成的桥接字段信息
#[derive(Debug, Clone)]
pub struct BridgeField {
    /// 字段名称
    pub name: String,
    /// Rust 类型
    pub ty: String,
}

/// 用于 TypeScript 生成的桥接类型信息
#[derive(Debug, Clone)]
pub struct BridgeType {
    /// 类型名称
    pub name: String,
    /// 字段列表
    pub fields: Vec<BridgeField>,
}

/// 可以桥接到 TypeScript 的类型 trait
pub trait TypeBridge {
    /// 获取桥接信息
    fn bridge_info() -> BridgeType;
}

impl NargoValue {
    /// 转换为字符串引用
    pub fn as_str(&self) -> Option<&str> {
        match self {
            NargoValue::String(s) => Some(s),
            NargoValue::Signal(s) => Some(s),
            NargoValue::Raw(s) => Some(s),
            NargoValue::Ref(s) => Some(s),
            _ => None,
        }
    }

    /// 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NargoValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 转换为数字
    pub fn as_number(&self) -> Option<f64> {
        match self {
            NargoValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 转换为二进制数据引用
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            NargoValue::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// 转换为对象引用
    pub fn as_object(&self) -> Option<&HashMap<String, NargoValue>> {
        match self {
            NargoValue::Object(o) => Some(o),
            _ => None,
        }
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String> {
        // 实现 oak_json 序列化
        Ok(format!("{:?}", self))
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        // 实现 oak_json 反序列化
        Err(Error::new(ErrorKind::NotImplemented { feature: "from_json".to_string(), span: Span::unknown() }))
    }

    /// 判断值是否与 JSON 兼容
    pub fn is_json_compatible(&self) -> bool {
        match self {
            NargoValue::Null | NargoValue::Bool(_) | NargoValue::Number(_) | NargoValue::String(_) | NargoValue::Array(_) | NargoValue::Object(_) => true,
            NargoValue::Signal(_) | NargoValue::Binary(_) | NargoValue::Raw(_) | NargoValue::Ref(_) => false,
        }
    }
}

impl std::fmt::Display for NargoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NargoValue::Null => write!(f, "null"),
            NargoValue::Bool(b) => write!(f, "{}", b),
            NargoValue::Number(n) => write!(f, "{}", n),
            NargoValue::String(s) => write!(f, "{:?}", s),
            NargoValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            NargoValue::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            }
            NargoValue::Signal(s) => write!(f, "${}", s),
            NargoValue::Binary(_) => write!(f, "<binary>"),
            NargoValue::Raw(s) => write!(f, "{}", s),
            NargoValue::Ref(s) => write!(f, "@{}", s),
        }
    }
}

/// 源代码解析游标
pub struct Cursor<'a> {
    /// 源代码字符串
    pub source: &'a str,
    /// 当前字节偏移位置
    pub pos: usize,
    /// 当前行号
    pub line: usize,
    /// 当前列号
    pub column: usize,
    /// 基础偏移量
    pub base_offset: usize,
}

impl<'a> Cursor<'a> {
    /// 创建一个新的游标
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0, line: 1, column: 1, base_offset: 0 }
    }

    /// 从指定位置创建游标
    pub fn with_position(source: &'a str, pos: Position) -> Self {
        Self { source, pos: pos.offset as usize, line: pos.line as usize, column: pos.column as usize, base_offset: 0 }
    }

    /// 从切片源创建游标
    pub fn with_sliced_source(source: &'a str, pos: Position) -> Self {
        Self { source, pos: 0, line: pos.line as usize, column: pos.column as usize, base_offset: pos.offset as usize }
    }

    /// 判断是否已到达文件末尾
    pub fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// 查看当前字符（不消费）
    pub fn peek(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }

    /// 查看第 n 个字符（不消费）
    pub fn peek_n(&self, n: usize) -> char {
        self.source[self.pos..].chars().nth(n).unwrap_or('\0')
    }

    /// 判断当前位置是否以指定字符串开头
    pub fn peek_str(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    /// 消费当前字符
    pub fn consume(&mut self) -> char {
        let c = self.peek();
        if c == '\0' {
            return c;
        }
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        }
        else {
            self.column += c.len_utf16();
        }
        c
    }

    /// 消费 n 个字符
    pub fn consume_n(&mut self, n: usize) {
        for _ in 0..n {
            self.consume();
        }
    }

    /// 如果匹配则消费字符串
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.peek_str(s) {
            self.consume_n(s.chars().count());
            true
        }
        else {
            false
        }
    }

    /// 消费满足条件的字符
    pub fn consume_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        while !self.is_eof() && f(self.peek()) {
            self.consume();
        }
        self.current_str(start).to_string()
    }

    /// 跳过空白字符
    pub fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.peek().is_whitespace() {
            self.consume();
        }
    }

    /// 消费并返回空白字符
    pub fn consume_whitespace(&mut self) -> String {
        let start = self.pos;
        while !self.is_eof() && self.peek().is_whitespace() {
            self.consume();
        }
        self.current_str(start).to_string()
    }

    /// 跳过空格和制表符
    pub fn skip_spaces(&mut self) {
        while !self.is_eof() && (self.peek() == ' ' || self.peek() == '\t') {
            self.consume();
        }
    }

    /// 期望消费指定字符
    pub fn expect(&mut self, expected: char) -> Result<()> {
        if self.peek() == expected {
            self.consume();
            Ok(())
        }
        else {
            Err(Error::expected_char(expected, self.peek(), self.span_at_current()))
        }
    }

    /// 期望消费指定字符串
    pub fn expect_str(&mut self, expected: &str) -> Result<()> {
        if self.peek_str(expected) {
            self.consume_n(expected.chars().count());
            Ok(())
        }
        else {
            Err(Error::expected_string(expected.to_string(), self.peek().to_string(), self.span_at_current()))
        }
    }

    /// 获取当前位置
    pub fn position(&self) -> Position {
        Position { line: self.line as u32, column: self.column as u32, offset: (self.base_offset + self.pos) as u32 }
    }

    /// 获取从起始位置到当前位置的字符串
    pub fn current_str(&self, start: usize) -> &str {
        &self.source[start..self.pos]
    }

    /// 获取当前字符的位置范围
    pub fn span_at_current(&self) -> Span {
        let start = self.position();
        let mut end = start;
        let c = self.peek();
        if c != '\0' {
            end.column += c.len_utf16() as u32;
            end.offset += c.len_utf8() as u32;
        }
        Span { start, end }
    }

    /// 获取从指定起始位置到当前位置的范围
    pub fn span_from(&self, start: Position) -> Span {
        Span { start, end: self.position() }
    }

    /// 消费一个标识符
    pub fn consume_ident(&mut self) -> Result<String> {
        let start = self.pos;
        if !is_alphabetic(self.peek()) {
            return Err(Error::parse_error("Expected identifier".to_string(), self.span_at_current()));
        }
        while !self.is_eof() && is_alphanumeric(self.peek()) {
            self.consume();
        }
        Ok(self.current_str(start).to_string())
    }

    /// 消费一个字符串字面量
    pub fn consume_string(&mut self) -> Result<String> {
        let quote = self.peek();
        if quote != '"' && quote != '\'' {
            return Err(Error::parse_error("Expected string".to_string(), self.span_at_current()));
        }
        self.consume();
        let start = self.pos;
        while !self.is_eof() && self.peek() != quote {
            self.consume();
        }
        let s = self.current_str(start).to_string();
        self.expect(quote)?;
        Ok(s)
    }

    /// 在指定位置创建一个零宽度的 Span
    pub fn span_at_pos(&self, pos: Position) -> Span {
        Span { start: pos, end: pos }
    }
}

/// 代码生成器
#[derive(Debug, Clone, Default)]
pub struct CodeWriter {
    buffer: String,
    indent_level: usize,
    mappings: Vec<(Position, Span)>,
    current_pos: Position,
}

impl CodeWriter {
    /// 创建一个新的代码生成器
    pub fn new() -> Self {
        Self::default()
    }

    /// 写入文本
    pub fn write(&mut self, text: &str) {
        if self.buffer.is_empty() || self.buffer.ends_with('\n') {
            let indent = "  ".repeat(self.indent_level);
            self.buffer.push_str(&indent);
            self.current_pos.column += indent.len() as u32;
            self.current_pos.offset += indent.len() as u32;
        }

        self.buffer.push_str(text);
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.len() > 1 {
            self.current_pos.line += (lines.len() - 1) as u32;
            if let Some(last_line) = lines.last() {
                self.current_pos.column = last_line.len() as u32;
            }
        }
        else {
            self.current_pos.column += text.len() as u32;
        }
        self.current_pos.offset += text.len() as u32;
    }

    /// 写入文本并记录源位置映射
    pub fn write_with_span(&mut self, text: &str, span: Span) {
        if !span.is_unknown() {
            self.mappings.push((self.current_pos, span));
        }
        self.write(text);
    }

    /// 写入一行文本
    pub fn write_line(&mut self, text: &str) {
        self.write(text);
        self.newline();
    }

    /// 写入一行文本并记录源位置映射
    pub fn write_line_with_span(&mut self, text: &str, span: Span) {
        self.write_with_span(text, span);
        self.newline();
    }

    /// 写入换行符
    pub fn newline(&mut self) {
        self.buffer.push('\n');
        self.current_pos.line += 1;
        self.current_pos.column = 0;
        self.current_pos.offset += 1;
    }

    /// 增加缩进级别
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// 减少缩进级别
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// 获取当前位置
    pub fn position(&self) -> Position {
        self.current_pos
    }

    /// 追加另一个代码生成器的内容
    pub fn append(&mut self, other: CodeWriter) {
        let (other_buf, other_mappings) = other.finish();
        for (mut pos, span) in other_mappings {
            pos.line += self.current_pos.line;
            if pos.line == self.current_pos.line {
                pos.column += self.current_pos.column;
            }
            pos.offset += self.current_pos.offset;
            self.mappings.push((pos, span));
        }
        self.write(&other_buf);
    }

    /// 完成生成并返回代码和位置映射
    pub fn finish(self) -> (String, Vec<(Position, Span)>) {
        (self.buffer, self.mappings)
    }
}
