//! Frontmatter 解析模块
//! 解析文档开头的 YAML 或 TOML frontmatter

use nargo_types::{FrontMatter, NargoValue, Result};

/// Frontmatter 解析器
pub struct FrontMatterParser;

impl FrontMatterParser {
    /// 解析 frontmatter
    ///
    /// # Arguments
    ///
    /// * `source` - 完整的文档内容
    ///
    /// # Returns
    ///
    /// 解析后的 frontmatter 和内容起始位置
    pub fn parse(source: &str) -> Result<(FrontMatter, usize)> {
        let source = source.trim_start();

        if !source.starts_with("---") {
            return Ok((FrontMatter::new(), 0));
        }

        let after_first = &source[3..];
        let end_pos = after_first.find("---");

        if let Some(end) = end_pos {
            let frontmatter_content = after_first[..end].trim();
            let content_start = source[3..].len() - after_first[end + 3..].len() + 3 + 3;

            let frontmatter = Self::parse_toml(frontmatter_content)?;

            Ok((frontmatter, content_start))
        }
        else {
            Ok((FrontMatter::new(), 0))
        }
    }

    fn parse_toml(content: &str) -> Result<FrontMatter> {
        let mut frontmatter = FrontMatter::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                match key {
                    "title" => {
                        frontmatter.title = Some(Self::parse_string(value));
                    }
                    "description" => {
                        frontmatter.description = Some(Self::parse_string(value));
                    }
                    "layout" => {
                        frontmatter.layout = Some(Self::parse_string(value));
                    }
                    "tags" => {
                        frontmatter.tags = Self::parse_array(value);
                    }
                    "sidebar" => {
                        frontmatter.sidebar = Self::parse_bool(value);
                    }
                    "sidebar_order" => {
                        frontmatter.sidebar_order = Self::parse_i32(value);
                    }
                    _ => {
                        let parsed_value = Self::parse_value(value);
                        frontmatter.custom.insert(key.to_string(), parsed_value);
                    }
                }
            }
        }

        Ok(frontmatter)
    }

    fn parse_string(value: &str) -> String {
        let value = value.trim();
        if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) { value[1..value.len() - 1].to_string() } else { value.to_string() }
    }

    fn parse_bool(value: &str) -> Option<bool> {
        let value = value.trim().to_lowercase();
        match value.as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    fn parse_i32(value: &str) -> Option<i32> {
        value.trim().parse().ok()
    }

    fn parse_array(value: &str) -> Vec<String> {
        let value = value.trim();
        if value.starts_with('[') && value.ends_with(']') {
            let content = &value[1..value.len() - 1];
            content.split(',').map(|s| Self::parse_string(s.trim())).filter(|s| !s.is_empty()).collect()
        }
        else {
            Vec::new()
        }
    }

    fn parse_value(value: &str) -> NargoValue {
        let value = value.trim();

        if value.starts_with('"') || value.starts_with('\'') {
            NargoValue::String(Self::parse_string(value))
        }
        else if let Ok(b) = value.parse::<bool>() {
            NargoValue::Bool(b)
        }
        else if let Ok(i) = value.parse::<i64>() {
            NargoValue::Number(i as f64)
        }
        else if let Ok(f) = value.parse::<f64>() {
            NargoValue::Number(f)
        }
        else if value.starts_with('[') && value.ends_with(']') {
            let arr = Self::parse_array(value);
            NargoValue::Array(arr.into_iter().map(NargoValue::String).collect())
        }
        else {
            NargoValue::String(value.to_string())
        }
    }
}
