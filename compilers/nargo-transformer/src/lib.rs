#![warn(missing_docs)]

pub use nargo_ir::*;
use nargo_types::{Result, Span};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::Path};

/// 插件系统模块
pub mod plugin;
pub use plugin::*;

/// 表示一次具体的变换操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub name: String,
    pub description: String,
    pub timestamp: u64,
    pub affected_span: Option<Span>,
    /// 变换前的 IR 快照 (JSON)
    pub ir_before: Option<String>,
    /// 变换后的 IR 快照 (JSON)
    pub ir_after: Option<String>,
}

/// 变换追踪器，用于记录和管理 IR 的变换过程
pub struct Transformer {
    logs: VecDeque<Transformation>,
    /// 是否开启 IR 快照记录（由于性能开销，默认关闭）
    pub enable_snapshots: bool,
    /// 插件管理器
    plugin_manager: PluginManager,
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Transformer {
    fn clone(&self) -> Self {
        Self {
            logs: self.logs.clone(),
            enable_snapshots: self.enable_snapshots,
            plugin_manager: PluginManager::new(), // 插件管理器重新初始化，避免状态共享
        }
    }
}

impl Transformer {
    pub fn new() -> Self {
        Self { logs: VecDeque::new(), enable_snapshots: false, plugin_manager: PluginManager::new() }
    }

    /// 记录一次变换
    pub fn log(&mut self, name: &str, description: &str, span: Option<Span>, ir_before: Option<String>, ir_after: Option<String>) {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        self.logs.push_back(Transformation { name: name.into(), description: description.into(), timestamp, affected_span: span, ir_before, ir_after });
    }

    /// 获取所有变换记录
    pub fn get_logs(&self) -> Vec<Transformation> {
        self.logs.iter().cloned().collect()
    }

    /// 清空变换记录
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    /// 将变换历史导出为 JSON 字符串，用于可视化工具
    pub fn export_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.get_logs()).map_err(|e| nargo_types::Error::external_error("Transformer".to_string(), format!("Failed to export logs: {}", e), Span::unknown()))
    }

    /// 加载插件
    pub fn load_plugin<P: Plugin + 'static>(&mut self, plugin: P, config: PluginConfig) -> Result<()> {
        self.plugin_manager.load_plugin(plugin, config)
    }

    /// 从目录加载插件
    pub fn load_plugins_from_directory(&mut self, directory: &Path) -> Result<()> {
        self.plugin_manager.load_plugins_from_directory(directory)
    }

    /// 执行指定生命周期阶段的所有插件
    pub fn run_plugins(&mut self, ir: &mut IRModule, lifecycle: PluginLifecycle) -> Result<()> {
        self.plugin_manager.run_plugins(ir, lifecycle)
    }

    /// 清理所有插件
    pub fn cleanup_plugins(&mut self) -> Result<()> {
        self.plugin_manager.cleanup_plugins()
    }

    /// 获取所有插件信息
    pub fn get_plugins_info(&self) -> Vec<(String, PluginConfig)> {
        self.plugin_manager.get_plugins_info()
    }

    /// 启用或禁用插件
    pub fn set_plugin_enabled(&mut self, plugin_name: &str, enabled: bool) -> Result<()> {
        self.plugin_manager.set_plugin_enabled(plugin_name, enabled)
    }

    /// 应用变换插件
    pub fn apply<T: TransformPass>(&mut self, ir: &mut IRModule, pass: &mut T) -> Result<()> {
        // 执行变换前插件
        self.run_plugins(ir, PluginLifecycle::PreTransform)?;

        let name = pass.name();
        let description = pass.description();

        let ir_before = if self.enable_snapshots { serde_json::to_string(ir).ok() } else { None };

        // 执行变换
        pass.transform(ir)?;

        // 执行变换阶段插件
        self.run_plugins(ir, PluginLifecycle::Transform)?;

        let ir_after = if self.enable_snapshots { serde_json::to_string(ir).ok() } else { None };

        // 记录变换
        self.log(&name, &description, None, ir_before, ir_after);

        // 执行变换后插件
        self.run_plugins(ir, PluginLifecycle::PostTransform)?;

        Ok(())
    }

    /// 并行应用多个变换插件
    pub fn apply_parallel<T: TransformPass + Sync>(&mut self, ir: &mut IRModule, passes: &mut [&mut T]) -> Result<()> {
        // 执行变换前插件
        self.run_plugins(ir, PluginLifecycle::PreTransform)?;

        // 并行执行变换插件
        for pass in passes {
            pass.transform(ir)?;
        }

        // 执行变换阶段插件
        self.run_plugins(ir, PluginLifecycle::Transform)?;

        // 执行变换后插件
        self.run_plugins(ir, PluginLifecycle::PostTransform)?;

        Ok(())
    }

    /// 辅助方法：在变换节点时保持 Span
    pub fn with_span<T, F>(_span: Span, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        // 这是一个占位符，未来可以用于在 thread_local 中追踪当前的 Span 传播
        f()
    }
}

/// 变换插件接口
pub trait TransformPass {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn transform(&mut self, ir: &mut IRModule) -> Result<()>;
}

// 导入所有变换插件
pub mod passes;
pub use passes::*;
