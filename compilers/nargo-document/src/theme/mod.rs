//! 主题系统模块
//! 提供主题管理和模板渲染功能，兼容 VuTeX 主题系统

pub mod dark_theme;
pub mod default_theme;
pub mod tech_theme;

pub use dark_theme::DarkTheme;
pub use default_theme::{DefaultTheme, LocaleInfo, NavItem, PageContext, SidebarGroup, SidebarLink};
pub use tech_theme::TechTheme;

use crate::config::Config;
use std::sync::Arc;

/// 主题 Trait，定义了主题必须实现的功能
pub trait Theme: Send + Sync {
    /// 获取主题名称
    fn name(&self) -> &str;

    /// 渲染页面
    fn render_page(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>>;

    /// 获取主题配置
    fn config(&self) -> &Config;

    /// 获取主题的静态资源
    fn static_resources(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

/// 主题工厂，用于创建主题实例
pub struct ThemeFactory;

impl ThemeFactory {
    /// 创建主题实例
    ///
    /// # 参数
    /// * `theme_name` - 主题名称
    /// * `config` - 文档配置
    ///
    /// # 返回值
    /// 返回主题实例或错误
    pub fn create(theme_name: &str, config: Config) -> Result<Arc<dyn Theme>, Box<dyn std::error::Error>> {
        match theme_name.to_lowercase().as_str() {
            "default" => Ok(Arc::new(DefaultTheme::new(config)?)),
            "dark" => Ok(Arc::new(DarkTheme::new(config)?)),
            "tech" => Ok(Arc::new(TechTheme::new(config)?)),
            "vutex-theme-default" => Ok(Arc::new(DefaultTheme::new(config)?)),
            _ => Err(format!("Unknown theme: {}", theme_name).into()),
        }
    }

    /// 获取所有可用主题名称
    pub fn available_themes() -> Vec<&'static str> {
        vec!["default", "dark", "tech"]
    }
}
