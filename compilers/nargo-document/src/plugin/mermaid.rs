//! Mermaid 图表渲染插件
//!
//! 提供对 Markdown 中 Mermaid 图表的支持，包括流程图、时序图、甘特图等

use crate::plugin::{DocumentPlugin, PluginContext, PluginMeta};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// 匹配 Mermaid 代码块的正则表达式
    static ref MERMAID_BLOCK_RE: Regex = Regex::new(r"```mermaid\s*\n([\s\S]*?)\n```").unwrap();
}

/// Mermaid 图表渲染插件
pub struct MermaidPlugin {
    /// 插件元数据
    meta: PluginMeta,
}

impl MermaidPlugin {
    /// 创建新的 Mermaid 插件实例
    pub fn new() -> Self {
        Self { meta: PluginMeta::new("nargo-document-plugin-mermaid".to_string(), "0.1.0".to_string(), "Mermaid 图表渲染插件，支持流程图、时序图、甘特图等".to_string()) }
    }

    /// 处理 Mermaid 代码块，将 ```mermaid ... ``` 替换为 <div class="mermaid">...</div>
    ///
    /// # Arguments
    ///
    /// * `content` - 包含 Mermaid 图表的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    pub fn process_mermaid(&self, content: &str) -> String {
        self.process_mermaid_blocks(content)
    }

    /// 处理 Mermaid 代码块，将 ```mermaid ... ``` 替换为 <div class="mermaid">...</div>
    ///
    /// # Arguments
    ///
    /// * `content` - 包含 Mermaid 图表的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    fn process_mermaid_blocks(&self, content: &str) -> String {
        MERMAID_BLOCK_RE
            .replace_all(content, |caps: &regex::Captures| {
                let diagram = &caps[1];
                format!("<div class=\"mermaid\">{}</div>", diagram.trim())
            })
            .to_string()
    }
}

impl Default for MermaidPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentPlugin for MermaidPlugin {
    /// 获取插件元数据
    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    /// 渲染前钩子，在 Markdown 解析后、HTML 渲染前处理 Mermaid 图表
    ///
    /// # Arguments
    ///
    /// * `context` - 插件上下文，包含文档内容等信息
    ///
    /// # Returns
    ///
    /// 处理后的插件上下文
    fn before_render(&self, context: PluginContext) -> PluginContext {
        let content = self.process_mermaid_blocks(&context.content);

        PluginContext { content, frontmatter: context.frontmatter, path: context.path }
    }
}
