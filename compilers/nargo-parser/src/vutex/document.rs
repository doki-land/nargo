use crate::vutex::FrontMatterParser;
use nargo_types::{Document, FrontMatter, Result};

/// 从 Markdown 内容中提取第一个标题
fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            let mut heading = line.trim_start_matches('#').trim();
            if heading.is_empty() {
                continue;
            }
            return Some(heading.to_string());
        }
    }
    None
}

/// 解析文档
///
/// # Arguments
///
/// * `source` - 完整的文档内容，包含可选的 frontmatter 和 markdown
/// * `path` - 文档路径
///
/// # Returns
///
/// 解析后的文档
pub fn parse_document(source: &str, path: &str) -> Result<Document> {
    let (frontmatter, content_start) = FrontMatterParser::parse(source)?;

    let content = if content_start < source.len() { source[content_start..].to_string() } else { String::new() };

    let mut doc = Document::new().with_path(path.to_string()).with_frontmatter(frontmatter).with_content(content);

    if doc.frontmatter.title.is_none() {
        doc.meta.title = extract_first_heading(&doc.content);
    }
    else {
        doc.meta.title = doc.frontmatter.title.clone();
    }

    Ok(doc)
}

/// 仅解析 frontmatter
///
/// # Arguments
///
/// * `source` - 文档内容
///
/// # Returns
///
/// 解析后的 frontmatter 和内容起始位置
pub fn parse_frontmatter(source: &str) -> Result<(FrontMatter, usize)> {
    FrontMatterParser::parse(source)
}
