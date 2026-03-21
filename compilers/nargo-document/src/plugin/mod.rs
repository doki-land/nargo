//! 插件模块
//! 提供 nargo-document 文档编译器的插件系统

use nargo_types::NargoValue;
use std::collections::HashMap;

pub mod container;
pub mod katex;
pub mod mermaid;
pub mod prism;
pub mod shiki;

pub use container::{ContainerConfig, ContainerOptions, ContainerPlugin};
pub use katex::KaTeXPlugin;
pub use mermaid::MermaidPlugin;
pub use prism::PrismPlugin;
pub use shiki::ShikiPlugin;

/// 插件元数据
#[derive(Debug, Clone)]
pub struct PluginMeta {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
}

impl PluginMeta {
    /// 创建新的插件元数据
    pub fn new(name: String, version: String, description: String) -> Self {
        Self { name, version, description }
    }
}

/// 插件上下文
/// 提供给插件处理的文档信息
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// 文档内容
    pub content: String,
    /// 前置元数据
    pub frontmatter: HashMap<String, NargoValue>,
    /// 文档路径
    pub path: String,
}

impl PluginContext {
    /// 创建新的插件上下文
    pub fn new(content: String, frontmatter: HashMap<String, NargoValue>, path: String) -> Self {
        Self { content, frontmatter, path }
    }

    /// 从文档内容创建插件上下文
    pub fn from_content(content: String, path: String) -> Self {
        Self { content, frontmatter: HashMap::new(), path }
    }
}

/// 文档插件 Trait
/// 定义插件需要实现的钩子方法
pub trait DocumentPlugin: Send + Sync {
    /// 获取插件元数据
    fn meta(&self) -> &PluginMeta;

    /// 插件初始化钩子
    /// 在编译开始前调用，用于插件的初始化
    fn setup(&mut self, config: Option<HashMap<String, NargoValue>>) {
        let _ = config;
    }

    /// 渲染前钩子
    /// 在 Markdown 解析后、HTML 渲染前调用，用于修改文档内容
    fn before_render(&self, context: PluginContext) -> PluginContext {
        context
    }

    /// 渲染后钩子
    /// 在 HTML 渲染后调用，用于修改渲染后的内容
    fn after_render(&self, context: PluginContext) -> PluginContext {
        context
    }
}

/// 插件注册表
/// 用于管理和注册文档插件
pub struct PluginRegistry {
    /// 已注册的插件列表
    plugins: Vec<Box<dyn DocumentPlugin>>,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// 注册一个插件
    pub fn register<P: DocumentPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// 初始化所有已注册的插件
    pub fn setup_all(&mut self, config: Option<HashMap<String, NargoValue>>) {
        for plugin in &mut self.plugins {
            plugin.setup(config.clone());
        }
    }

    /// 对所有已注册的插件调用渲染前钩子
    pub fn before_render_all(&self, context: PluginContext) -> PluginContext {
        let mut current_context = context;
        for plugin in &self.plugins {
            current_context = plugin.before_render(current_context);
        }
        current_context
    }

    /// 对所有已注册的插件调用渲染后钩子
    pub fn after_render_all(&self, context: PluginContext) -> PluginContext {
        let mut current_context = context;
        for plugin in &self.plugins {
            current_context = plugin.after_render(current_context);
        }
        current_context
    }

    /// 获取已注册插件的数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// 获取所有已注册插件的元数据
    pub fn plugin_metas(&self) -> Vec<&PluginMeta> {
        self.plugins.iter().map(|p| p.meta()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
