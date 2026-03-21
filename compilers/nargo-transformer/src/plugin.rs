use nargo_ir::IRModule;
use nargo_types::{NargoValue, Result, Span};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsStr, fs, path::Path};

/// 插件生命周期阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginLifecycle {
    /// 初始化阶段，插件首次加载时执行
    Init,
    /// 变换前阶段，在所有变换开始前执行
    PreTransform,
    /// 变换阶段，与内置变换一起执行
    Transform,
    /// 变换后阶段，在所有变换完成后执行
    PostTransform,
    /// 清理阶段，在整个变换过程结束后执行
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
    /// 插件优先级（数值越小，优先级越高）
    pub priority: i32,
    /// 插件配置参数
    pub config: HashMap<String, NargoValue>,
    /// 插件启用状态
    pub enabled: bool,
}

/// 插件接口
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> String;

    /// 获取插件描述
    fn description(&self) -> String;

    /// 初始化插件
    fn init(&mut self, config: &PluginConfig) -> Result<()>;

    /// 执行插件逻辑
    fn run(&mut self, ir: &mut IRModule, lifecycle: PluginLifecycle) -> Result<()>;

    /// 清理插件资源
    fn cleanup(&mut self) -> Result<()>;
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self { plugins: Vec::new(), configs: HashMap::new() }
    }

    /// 加载插件
    pub fn load_plugin<P: Plugin + 'static>(&mut self, plugin: P, config: PluginConfig) -> Result<()> {
        let plugin_name = plugin.name();

        // 初始化插件
        let mut plugin_box = Box::new(plugin);
        plugin_box.init(&config)?;

        // 存储插件和配置
        self.plugins.push(plugin_box);
        self.configs.insert(plugin_name, config);

        // 按优先级排序插件
        self.plugins.sort_by(|a, b| {
            let a_name = a.name();
            let b_name = b.name();
            let a_priority = self.configs.get(&a_name).map(|c| c.priority).unwrap_or(0);
            let b_priority = self.configs.get(&b_name).map(|c| c.priority).unwrap_or(0);
            a_priority.cmp(&b_priority)
        });

        Ok(())
    }

    /// 从目录加载插件
    pub fn load_plugins_from_directory(&mut self, directory: &Path) -> Result<()> {
        if !directory.exists() || !directory.is_dir() {
            return Ok(()); // 目录不存在，直接返回
        }

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            // 只处理 .rs 文件
            if path.is_file() && path.extension() == Some(OsStr::new("rs")) {
                // 这里简化处理，实际实现可能需要动态加载或编译插件
                // 目前我们假设插件已经通过 crate 依赖的方式集成
                // 后续可以扩展为支持动态加载
            }
        }

        Ok(())
    }

    /// 执行指定生命周期阶段的所有插件
    pub fn run_plugins(&mut self, ir: &mut IRModule, lifecycle: PluginLifecycle) -> Result<()> {
        for plugin in &mut self.plugins {
            let plugin_name = plugin.name();
            let config = self.configs.get(&plugin_name).unwrap();

            if config.enabled {
                plugin.run(ir, lifecycle.clone())?;
            }
        }

        Ok(())
    }

    /// 清理所有插件
    pub fn cleanup_plugins(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.cleanup()?;
        }

        Ok(())
    }

    /// 获取所有插件信息
    pub fn get_plugins_info(&self) -> Vec<(String, PluginConfig)> {
        self.configs.iter().map(|(name, config)| (name.clone(), config.clone())).collect()
    }

    /// 启用或禁用插件
    pub fn set_plugin_enabled(&mut self, plugin_name: &str, enabled: bool) -> Result<()> {
        if let Some(config) = self.configs.get_mut(plugin_name) {
            config.enabled = enabled;
            Ok(())
        }
        else {
            Err(nargo_types::Error::external_error("PluginManager".to_string(), format!("Plugin {} not found", plugin_name), Span::unknown()))
        }
    }
}
