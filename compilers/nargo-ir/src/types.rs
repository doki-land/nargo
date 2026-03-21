#![warn(missing_docs)]

//! 基础类型模块
//!
//! 提供 HXO IR 系统的基础类型定义，包括错误类型、常量和基础数据结构。

use nargo_types::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 代码位置信息
pub use nargo_types::Span;

/// IR 错误类型
///
/// 表示在处理 IR 时可能遇到的各种错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum IRError {
    /// 无效的输入
    ///
    /// 当输入数据不符合预期格式时返回此错误
    InvalidInput(String),
    /// 结构不一致
    ///
    /// 当 IR 结构内部不一致时返回此错误
    InconsistentStructure(String),
    /// 超出大小限制
    ///
    /// 当 IR 元素大小超出预设限制时返回此错误
    SizeLimitExceeded(String),
    /// 循环引用
    ///
    /// 当 IR 中存在循环引用时返回此错误
    CircularReference(String),
    /// 无效的表达式
    ///
    /// 当表达式语法或语义无效时返回此错误
    InvalidExpression(String),
    /// 其他错误
    ///
    /// 其他未分类的错误
    Other(String),
}

impl fmt::Display for IRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            IRError::InconsistentStructure(msg) => write!(f, "Inconsistent structure: {}", msg),
            IRError::SizeLimitExceeded(msg) => write!(f, "Size limit exceeded: {}", msg),
            IRError::CircularReference(msg) => write!(f, "Circular reference: {}", msg),
            IRError::InvalidExpression(msg) => write!(f, "Invalid expression: {}", msg),
            IRError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<IRError> for Error {
    fn from(err: IRError) -> Self {
        Error::external_error("nargo-ir".to_string(), err.to_string(), Span::unknown())
    }
}

/// 大小限制常量
pub const MAX_STRING_LENGTH: usize = 1024 * 1024; // 1MB
pub const MAX_ARRAY_LENGTH: usize = 10000;
pub const MAX_OBJECT_SIZE: usize = 1000;
pub const MAX_RECURSION_DEPTH: usize = 100;

/// 代码中的 trivia 信息，包括空白和注释
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Trivia {
    /// 前导空白
    pub leading_whitespace: String,
    /// 前导注释
    pub leading_comments: Vec<Comment>,
    /// 尾随注释
    pub trailing_comments: Vec<Comment>,
}

impl Trivia {
    /// 创建一个空的 Trivia
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查 Trivia 是否为空
    pub fn is_empty(&self) -> bool {
        self.leading_whitespace.is_empty() && self.leading_comments.is_empty() && self.trailing_comments.is_empty()
    }
}

/// 代码注释
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// 注释内容
    pub content: String,
    /// 是否为块注释
    pub is_block: bool,
    /// 注释的位置信息
    pub span: Span,
}

impl Comment {
    /// 创建一个新的注释
    pub fn new(content: String, is_block: bool, span: Span) -> Self {
        Self { content, is_block, span }
    }

    /// 检查注释是否为空
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}
