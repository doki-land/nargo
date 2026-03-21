//! Markdown 解析模块
//! 处理 Markdown 内容

use nargo_types::Result;

/// Markdown 解析器
pub struct MarkdownParser;

impl MarkdownParser {
    /// 解析 Markdown 内容（当前仅返回原始内容）
    ///
    /// # Arguments
    ///
    /// * `source` - Markdown 源文本
    ///
    /// # Returns
    ///
    /// 解析后的内容（当前为原始字符串）
    pub fn parse(source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    /// 解析 Markdown 为 HTML（占位实现）
    ///
    /// # Arguments
    ///
    /// * `source` - Markdown 源文本
    ///
    /// # Returns
    ///
    /// HTML 字符串
    pub fn to_html(source: &str) -> Result<String> {
        Ok(source.to_string())
    }
}
