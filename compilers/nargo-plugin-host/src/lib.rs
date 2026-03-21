#![warn(missing_docs)]

use nargo_types::{NargoContext, NargoValue, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// 插件生命周期阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginLifecycle {
    /// 初始化阶段，插件首次加载时执行
    Init,
    /// 解析前阶段
    PreParse,
    /// 解析阶段
    Parse,
    /// 解析后阶段
    PostParse,
    /// 变换前阶段
    PreTransform,
    /// 变换阶段
    Transform,
    /// 变换后阶段
    PostTransform,
    /// 打包前阶段
    PreBundle,
    /// 打包阶段
    Bundle,
    /// 打包后阶段
    PostBundle,
    /// 清理阶段
    Cleanup,
}

/// 插件配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 插件作者
    pub author: Option<String>,
    /// 插件主页
    pub homepage: Option<String>,
    /// 插件优先级（数值越小，优先级越高）
    pub priority: i32,
    /// 插件配置参数
    pub config: HashMap<String, NargoValue>,
    /// 插件启用状态
    pub enabled: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self { name: String::new(), version: "0.1.0".to_string(), description: String::new(), author: None, homepage: None, priority: 0, config: HashMap::new(), enabled: true }
    }
}

/// Plugin trait for extending Nargo functionality.
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Returns the version of the plugin.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Returns the description of the plugin.
    fn description(&self) -> &str {
        ""
    }

    /// Returns the author of the plugin.
    fn author(&self) -> Option<&str> {
        None
    }

    /// Returns the homepage of the plugin.
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Returns the priority of the plugin.
    fn priority(&self) -> i32 {
        0
    }

    /// Called when the plugin is initialized.
    fn on_init(&self, _ctx: Arc<NargoContext>, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    /// Called before parsing phase.
    fn on_pre_parse(&self, _source: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during parsing phase.
    fn on_parse(&self, _source: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called after parsing phase.
    fn on_post_parse(&self, _source: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called before transform phase.
    fn on_pre_transform(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during transform phase.
    fn on_transform(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called after transform phase.
    fn on_post_transform(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called before bundle phase.
    fn on_pre_bundle(&self, _bundle: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during bundle phase.
    fn on_bundle(&self, _bundle: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called after bundle phase.
    fn on_post_bundle(&self, _bundle: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during cleanup phase.
    fn on_cleanup(&self) -> Result<()> {
        Ok(())
    }
}

/// JavaScript-based plugin implementation.
pub struct JsPlugin {
    name: String,
    version: String,
    description: String,
}

impl JsPlugin {
    /// Creates a new JavaScript plugin.
    pub fn new(name: String, version: String, description: String, _source: &str) -> Result<Self> {
        Ok(Self { name, version, description })
    }
}

impl Plugin for JsPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Manager for plugins.
pub struct PluginManager {
    ctx: Arc<NargoContext>,
    plugins: Vec<Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
}

impl PluginManager {
    /// Creates a new plugin manager.
    pub fn new(ctx: Arc<NargoContext>) -> Self {
        Self { ctx, plugins: Vec::new(), configs: HashMap::new() }
    }

    /// Registers a plugin with default config.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let config = PluginConfig { name: plugin.name().to_string(), version: plugin.version().to_string(), description: plugin.description().to_string(), author: plugin.author().map(|s| s.to_string()), homepage: plugin.homepage().map(|s| s.to_string()), priority: plugin.priority(), enabled: true, ..Default::default() };
        self.register_with_config(plugin, config);
    }

    /// Registers a plugin with custom config.
    pub fn register_with_config(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) {
        tracing::info!("Registering plugin: {} v{}", plugin.name(), plugin.version());
        self.configs.insert(plugin.name().to_string(), config);
        self.plugins.push(plugin);
        self.sort_plugins();
    }

    /// Sorts plugins by priority.
    fn sort_plugins(&mut self) {
        self.plugins.sort_by(|a, b| {
            let a_priority = a.priority();
            let b_priority = b.priority();
            a_priority.cmp(&b_priority)
        });
    }

    /// Initializes all registered plugins.
    pub fn init_all(&self) -> Result<()> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if config.enabled {
                    plugin.on_init(self.ctx.clone(), config)?;
                }
            }
        }
        Ok(())
    }

    /// Processes source code through all registered plugins at parse phase.
    pub async fn parse(&self, mut source: String) -> Result<String> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(new_source) = plugin.on_pre_parse(&source)? {
                source = new_source;
            }
            if let Some(new_source) = plugin.on_parse(&source)? {
                source = new_source;
            }
            if let Some(new_source) = plugin.on_post_parse(&source)? {
                source = new_source;
            }
        }
        Ok(source)
    }

    /// Transforms code through all registered plugins.
    pub async fn transform(&self, mut code: String) -> Result<String> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(new_code) = plugin.on_pre_transform(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_transform(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_post_transform(&code)? {
                code = new_code;
            }
        }
        Ok(code)
    }

    /// Bundles code through all registered plugins.
    pub async fn bundle(&self, mut bundle: String) -> Result<String> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(new_bundle) = plugin.on_pre_bundle(&bundle)? {
                bundle = new_bundle;
            }
            if let Some(new_bundle) = plugin.on_bundle(&bundle)? {
                bundle = new_bundle;
            }
            if let Some(new_bundle) = plugin.on_post_bundle(&bundle)? {
                bundle = new_bundle;
            }
        }
        Ok(bundle)
    }

    /// Cleans up all registered plugins.
    pub fn cleanup_all(&self) -> Result<()> {
        for plugin in &self.plugins {
            plugin.on_cleanup()?;
        }
        Ok(())
    }

    /// Gets all registered plugins.
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Gets a plugin by name.
    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.iter().find(|p| p.name() == name)
    }

    /// Gets a plugin config by name.
    pub fn get_config(&self, name: &str) -> Option<&PluginConfig> {
        self.configs.get(name)
    }

    /// Enables or disables a plugin.
    pub fn set_plugin_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = enabled;
            Ok(())
        }
        else {
            Err(nargo_types::Error::external_error("PluginManager".to_string(), format!("Plugin {} not found", name), nargo_types::Span::unknown()))
        }
    }

    /// Gets all plugins info.
    pub fn get_plugins_info(&self) -> Vec<(&str, &PluginConfig)> {
        self.plugins.iter().filter_map(|p| self.configs.get(p.name()).map(|c| (p.name(), c))).collect()
    }
}
