//! Markdown 渲染器模块
//! 提供将 Markdown 文本渲染为 HTML 的功能

/// Markdown 渲染器配置
#[derive(Debug, Clone)]
pub struct MarkdownRendererConfig {
    /// 是否启用表格支持
    pub enable_tables: bool,
    /// 是否启用脚注支持
    pub enable_footnotes: bool,
    /// 是否启用删除线支持
    pub enable_strikethrough: bool,
    /// 是否启用任务列表支持
    pub enable_tasklists: bool,
    /// 是否启用智能标点
    pub enable_smart_punctuation: bool,
}

impl Default for MarkdownRendererConfig {
    fn default() -> Self {
        Self { enable_tables: true, enable_footnotes: true, enable_strikethrough: true, enable_tasklists: true, enable_smart_punctuation: true }
    }
}

/// Markdown 渲染器
#[derive(Debug, Clone)]
pub struct MarkdownRenderer {
    /// 渲染器配置
    config: MarkdownRendererConfig,
}

impl MarkdownRenderer {
    /// 创建新的 Markdown 渲染器
    pub fn new() -> Self {
        Self { config: MarkdownRendererConfig::default() }
    }

    /// 创建带配置的 Markdown 渲染器
    ///
    /// # Arguments
    ///
    /// * `config` - 渲染器配置
    pub fn with_config(config: MarkdownRendererConfig) -> Self {
        Self { config }
    }

    /// 获取渲染器配置
    pub fn config(&self) -> &MarkdownRendererConfig {
        &self.config
    }

    /// 获取可变的渲染器配置
    pub fn config_mut(&mut self) -> &mut MarkdownRendererConfig {
        &mut self.config
    }

    /// 将 Markdown 文本渲染为 HTML
    ///
    /// # Arguments
    ///
    /// * `markdown` - Markdown 文本内容
    ///
    /// # Returns
    ///
    /// 渲染后的 HTML 字符串
    pub fn render(&self, markdown: &str) -> Result<String, std::io::Error> {
        use pulldown_cmark::{Options, Parser, html};

        let mut options = Options::empty();
        if self.config.enable_tables {
            options.insert(Options::ENABLE_TABLES);
        }
        if self.config.enable_footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }
        if self.config.enable_strikethrough {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if self.config.enable_tasklists {
            options.insert(Options::ENABLE_TASKLISTS);
        }
        if self.config.enable_smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(format!("<div class=\"markdown\">{}</div>", html_output))
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}
