//! 配置模块
//! 提供文档配置的加载和验证功能

use oak_toml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配置加载和验证相关的错误类型
#[derive(Debug)]
pub enum ConfigError {
    /// 文件读取错误
    FileReadError(std::io::Error),
    /// JSON 解析错误
    JsonParseError(String),
    /// TOML 解析错误
    TomlParseError(String),
    /// 配置验证错误
    ValidationError(String),
    /// 不支持的配置文件格式
    UnsupportedFormat(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileReadError(err) => write!(f, "Failed to read config file: {}", err),
            ConfigError::JsonParseError(err) => write!(f, "Failed to parse JSON config: {}", err),
            ConfigError::TomlParseError(err) => write!(f, "Failed to parse TOML config: {}", err),
            ConfigError::ValidationError(msg) => write!(f, "Config validation error: {}", msg),
            ConfigError::UnsupportedFormat(fmt) => write!(f, "Unsupported config file format: {}", fmt),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::FileReadError(err)
    }
}

/// 文档配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// 站点标题
    pub title: Option<String>,
    /// 站点描述
    pub description: Option<String>,
    /// 基础路径
    pub base: Option<String>,
    /// 主题配置
    #[serde(default)]
    pub theme: ThemeConfig,
    /// 导航栏配置
    #[serde(default)]
    pub nav: Vec<NavItem>,
    /// 侧边栏配置
    #[serde(default)]
    pub sidebar: HashMap<String, Vec<SidebarItem>>,
    /// 语言配置
    #[serde(default)]
    pub locales: Vec<LocaleConfig>,
    /// Markdown 配置
    #[serde(default)]
    pub markdown: MarkdownConfig,
    /// 插件配置
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    /// 构建配置
    #[serde(default)]
    pub build: BuildConfig,
}

/// 主题配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// 主题名称
    pub name: Option<String>,
    /// 页脚配置
    #[serde(default)]
    pub footer: Option<FooterConfig>,
    /// 社交链接
    #[serde(default)]
    pub social_links: Vec<SocialLink>,
}

/// 页脚配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterConfig {
    /// 页脚消息
    pub message: Option<String>,
    /// 版权信息
    pub copyright: Option<String>,
}

/// 导航栏项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    /// 显示文本
    pub text: String,
    /// 链接地址
    pub link: Option<String>,
    /// 外部链接
    #[serde(default)]
    pub external: bool,
}

/// 侧边栏项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItem {
    /// 显示文本
    pub text: String,
    /// 链接地址
    pub link: Option<String>,
}

/// 语言配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    /// 语言代码
    pub code: String,
    /// 语言标签
    pub label: String,
}

/// Markdown 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// 是否启用 TOC
    #[serde(default)]
    pub toc: bool,
    /// 是否启用锚点
    #[serde(default)]
    pub anchor: bool,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件选项
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// 构建配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// 输出目录
    pub out_dir: Option<String>,
    /// 是否压缩
    #[serde(default)]
    pub minify: bool,
}

/// 社交链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    /// 平台名称
    pub platform: String,
    /// 链接地址
    pub url: String,
    /// 图标
    pub icon: Option<String>,
}

/// 配置验证 trait
pub trait ConfigValidation {
    /// 验证配置
    fn validate(&self) -> Result<(), ConfigError>;
}

impl ConfigValidation for Config {
    fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Config {
    /// 从文件加载配置
    ///
    /// # 参数
    /// * `path` - 配置文件路径
    ///
    /// # 返回值
    /// 返回 Result，成功时为 Config，失败时为错误信息
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = oak_toml::from_str(&content).map_err(|e| ConfigError::TomlParseError(format!("{:?}", e)))?;
        Ok(config)
    }
}
