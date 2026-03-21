//! 文档模块
//! 定义文档的元信息结构

use crate::{Error, ErrorKind, NargoValue, Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 文档
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Document {
    /// 文档元信息
    pub meta: DocumentMeta,
    /// 前置元数据（frontmatter）
    pub frontmatter: FrontMatter,
    /// 文档内容（原始文本）
    pub content: String,
    /// 解析后的内容（可以是 HTML 或结构化数据）
    pub rendered_content: Option<String>,
    /// 位置信息
    pub span: Span,
}

impl Document {
    /// 创建新的文档
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置文档路径
    pub fn with_path(mut self, path: String) -> Self {
        self.meta.path = path;
        self
    }

    /// 设置前置元数据
    pub fn with_frontmatter(mut self, frontmatter: FrontMatter) -> Self {
        self.frontmatter = frontmatter;
        self
    }

    /// 设置文档内容
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    /// 获取文档标题
    pub fn title(&self) -> Option<&str> {
        self.frontmatter.title.as_deref().or_else(|| self.meta.title.as_deref())
    }

    /// 获取文档描述
    pub fn description(&self) -> Option<&str> {
        self.frontmatter.description.as_deref()
    }

    /// 获取文档标签
    pub fn tags(&self) -> &[String] {
        &self.frontmatter.tags
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String> {
        // 实现 oak_json 序列化
        Ok(format!("{:?}", self))
    }

    /// 序列化为美化的 JSON
    pub fn to_json_pretty(&self) -> Result<String> {
        // 实现 oak_json 序列化
        Ok(format!("{:?}", self))
    }
}

/// 文档元信息
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DocumentMeta {
    /// 文档路径
    pub path: String,
    /// 文档标题
    pub title: Option<String>,
    /// 文档语言
    pub lang: Option<String>,
    /// 最后更新时间
    pub last_updated: Option<i64>,
    /// 其他元数据
    pub extra: HashMap<String, NargoValue>,
}

/// 前置元数据（Front Matter）
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    /// 页面标题
    pub title: Option<String>,
    /// 页面描述
    pub description: Option<String>,
    /// 页面布局
    pub layout: Option<String>,
    /// 页面标签
    pub tags: Vec<String>,
    /// 是否在侧边栏中隐藏
    pub sidebar: Option<bool>,
    /// 侧边栏顺序
    pub sidebar_order: Option<i32>,
    /// 自定义元数据
    pub custom: HashMap<String, NargoValue>,
}

impl FrontMatter {
    /// 创建新的前置元数据
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置标题
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置布局
    pub fn with_layout(mut self, layout: String) -> Self {
        self.layout = Some(layout);
        self
    }

    /// 添加标签
    pub fn add_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}
