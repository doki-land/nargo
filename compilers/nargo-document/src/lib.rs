#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

pub mod config;
pub mod generator;
pub mod plugin;
pub mod server;
pub mod templates;
pub mod theme;

pub use config::{BuildConfig, Config, ConfigError, ConfigValidation, FooterConfig, LocaleConfig, MarkdownConfig, NavItem, PluginConfig, SidebarItem, SocialLink, ThemeConfig};
pub use generator::{Generator, html::HtmlGenerator, markdown::MarkdownRenderer, pdf::PdfGenerator, static_::StaticProcessor};
pub use plugin::{ContainerConfig, ContainerOptions, ContainerPlugin, DocumentPlugin, KaTeXPlugin, MermaidPlugin, PluginContext, PluginMeta, PluginRegistry, PrismPlugin, ShikiPlugin};
pub use server::DevServer;
pub use templates::{Template, TemplateColors, TemplateConfig, TemplateFeatures, TemplateFonts, TemplateI18n};
pub use theme::{DarkTheme, DefaultTheme, LocaleInfo, NavItem as ThemeNavItem, PageContext, SidebarGroup, SidebarLink, TechTheme, Theme, ThemeFactory};
