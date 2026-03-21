//! 自定义容器插件
//!
//! 提供对 Markdown 中自定义容器的支持，包括 tip、warning、danger、info 等

use crate::plugin::{DocumentPlugin, PluginContext, PluginMeta};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    /// 匹配自定义容器的正则表达式
    static ref CONTAINER_RE: Regex = Regex::new(r":::(\w+)\s*\n([\s\S]*?)\n:::").unwrap();
}

/// 自定义容器配置
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// 内置容器列表
    pub containers: Vec<String>,
    /// 自定义容器配置
    pub custom_containers: HashMap<String, ContainerOptions>,
    /// 是否启用默认图标
    pub default_icons: bool,
}

/// 容器选项
#[derive(Debug, Clone)]
pub struct ContainerOptions {
    /// 容器标题
    pub title: Option<String>,
    /// 容器图标
    pub icon: Option<String>,
    /// 容器 CSS 类
    pub class: Option<String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self { containers: vec!["tip".to_string(), "warning".to_string(), "danger".to_string(), "info".to_string()], custom_containers: HashMap::new(), default_icons: true }
    }
}

/// 自定义容器插件
pub struct ContainerPlugin {
    /// 插件元数据
    meta: PluginMeta,
    /// 容器配置
    config: ContainerConfig,
}

impl ContainerPlugin {
    /// 创建新的 Container 插件实例
    pub fn new() -> Self {
        Self { meta: PluginMeta::new("nargo-document-plugin-container".to_string(), "0.1.0".to_string(), "自定义容器插件，提供信息提示、警告提示等容器组件".to_string()), config: ContainerConfig::default() }
    }

    /// 创建带自定义配置的 Container 插件实例
    ///
    /// # Arguments
    ///
    /// * `config` - 自定义容器配置
    pub fn with_config(config: ContainerConfig) -> Self {
        Self { meta: PluginMeta::new("nargo-document-plugin-container".to_string(), "0.1.0".to_string(), "自定义容器插件，提供信息提示、警告提示等容器组件".to_string()), config }
    }

    /// 处理自定义容器，将 :::type ... ::: 替换为 <div class="container-type">...</div>
    ///
    /// # Arguments
    ///
    /// * `content` - 包含自定义容器的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    pub fn process_containers(&self, content: &str) -> String {
        CONTAINER_RE
            .replace_all(content, |caps: &regex::Captures| {
                let container_type = &caps[1];
                let inner_content = &caps[2];

                let options = self.config.custom_containers.get(container_type);
                let class_name = options.and_then(|o| o.class.clone()).unwrap_or_else(|| format!("container-{}", container_type));

                let mut html = String::new();
                html.push_str(&format!("<div class=\"{}\">", class_name));

                if let Some(options) = options {
                    if let Some(title) = &options.title {
                        html.push_str(&format!("<div class=\"container-title\">{}</div>", title));
                    }
                }

                html.push_str(&format!("<div class=\"container-content\">{}</div>", inner_content.trim()));
                html.push_str("</div>");

                html
            })
            .to_string()
    }
}

impl Default for ContainerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentPlugin for ContainerPlugin {
    /// 获取插件元数据
    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    /// 渲染前钩子，在 Markdown 解析后、HTML 渲染前处理自定义容器
    ///
    /// # Arguments
    ///
    /// * `context` - 插件上下文，包含文档内容等信息
    ///
    /// # Returns
    ///
    /// 处理后的插件上下文
    fn before_render(&self, context: PluginContext) -> PluginContext {
        let content = self.process_containers(&context.content);

        PluginContext { content, frontmatter: context.frontmatter, path: context.path }
    }
}
