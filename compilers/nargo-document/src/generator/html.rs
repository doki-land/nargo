//! HTML 生成器
//! 使用主题系统生成完整的 HTML 文档页面

use crate::{
    config::Config,
    theme::{LocaleInfo, NavItem, PageContext, SidebarGroup, Theme, ThemeFactory},
};
use std::sync::Arc;

/// HTML 生成器
pub struct HtmlGenerator {
    /// 主题实例
    theme: Arc<dyn Theme>,
}

impl Clone for HtmlGenerator {
    fn clone(&self) -> Self {
        Self { theme: Arc::clone(&self.theme) }
    }
}

impl HtmlGenerator {
    /// 创建新的 HtmlGenerator
    ///
    /// 使用默认配置创建 HTML 生成器
    pub fn new() -> Self {
        let config = Config::default();
        Self::with_config_and_theme(config, "default")
    }

    /// 使用指定配置和主题名称创建 HtmlGenerator
    ///
    /// # 参数
    /// * `config` - 文档配置
    /// * `theme_name` - 主题名称
    pub fn with_config_and_theme(config: Config, theme_name: &str) -> Self {
        let theme = ThemeFactory::create(theme_name, config.clone()).unwrap_or_else(|_| ThemeFactory::create("default", config).expect("Failed to create default theme"));
        Self { theme }
    }

    /// 使用指定配置创建 HtmlGenerator（使用默认主题）
    ///
    /// # 参数
    /// * `config` - 文档配置
    pub fn with_config(config: Config) -> Self {
        Self::with_config_and_theme(config, "default")
    }

    /// 切换主题
    ///
    /// # 参数
    /// * `theme_name` - 新的主题名称
    pub fn switch_theme(&mut self, theme_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.theme.config().clone();
        self.theme = ThemeFactory::create(theme_name, config)?;
        Ok(())
    }

    /// 获取当前主题名称
    pub fn current_theme_name(&self) -> &str {
        self.theme.name()
    }

    /// 获取可用主题列表
    pub fn available_themes() -> Vec<&'static str> {
        ThemeFactory::available_themes()
    }

    /// 生成 HTML 文档
    ///
    /// # 参数
    /// * `content` - Markdown 渲染后的 HTML 内容
    /// * `title` - 页面标题
    ///
    /// # 返回值
    /// 返回完整的 HTML 页面字符串
    pub fn generate(&self, content: &str, title: &str) -> String {
        let site_title = self.theme.config().title.clone().unwrap_or_default();

        let site_title_str = site_title.to_string();
        let page_title = if !title.is_empty() && title != site_title_str { format!("{} | {}", title, site_title_str) } else { site_title_str.clone() };

        let nav_items: Vec<NavItem> = Vec::new();
        let sidebar_group = SidebarGroup { text: "文档".to_string(), items: Vec::new() };
        let sidebar_groups = vec![sidebar_group];

        let (has_footer, has_footer_message, footer_message, has_footer_copyright, footer_copyright) = if let Some(footer) = &self.theme.config().theme.footer { (true, footer.message.is_some(), footer.message.clone().unwrap_or_default(), footer.copyright.is_some(), footer.copyright.clone().unwrap_or_default()) } else { (false, false, String::new(), false, String::new()) };

        let locale_infos: Vec<LocaleInfo> = Vec::new();

        let context = PageContext { page_title, site_title: site_title_str, content: content.to_string(), nav_items, sidebar_groups, current_path: "".to_string(), has_footer, has_footer_message, footer_message, has_footer_copyright, footer_copyright, current_lang: "zh-CN".to_string(), available_locales: locale_infos, root_path: "./".to_string() };

        self.theme.render_page(&context).unwrap_or_else(|_| format!("<html><head><title>{}</title></head><body><h1>{}</h1>{}</body></html>", title, title, content))
    }

    /// 使用完整上下文生成 HTML 文档
    ///
    /// # 参数
    /// * `context` - 页面上下文
    ///
    /// # 返回值
    /// 返回完整的 HTML 页面字符串
    pub fn generate_with_context(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        self.theme.render_page(context)
    }

    /// 获取主题的静态资源
    pub fn static_resources(&self) -> Vec<(String, String)> {
        self.theme.static_resources()
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}
