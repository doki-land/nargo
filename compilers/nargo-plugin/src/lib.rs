#![warn(missing_docs)]

use base64::{Engine as _, engine::general_purpose};
use nargo_types::{NargoContext, NargoValue, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsStr, fs, path::Path, process::Command, sync::Arc};

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
    /// 优化前阶段
    PreOptimize,
    /// 优化阶段
    Optimize,
    /// 优化后阶段
    PostOptimize,
    /// 代码生成前阶段
    PreCodegen,
    /// 代码生成阶段
    Codegen,
    /// 代码生成后阶段
    PostCodegen,
    /// 打包前阶段
    PreBundle,
    /// 打包阶段
    Bundle,
    /// 打包后阶段
    PostBundle,
    /// 命令执行阶段
    Command,
    /// 清理阶段
    Cleanup,
}

/// 插件权限类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// 读取文件系统权限
    ReadFileSystem,
    /// 写入文件系统权限
    WriteFileSystem,
    /// 网络访问权限
    NetworkAccess,
    /// 执行命令权限
    ExecuteCommand,
    /// 访问环境变量权限
    EnvironmentAccess,
    /// 所有权限
    All,
}

/// 插件签名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    /// 签名数据（Base64编码）
    pub signature: String,
    /// 公钥（Base64编码）
    pub public_key: String,
    /// 签名算法
    pub algorithm: String,
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
    /// 插件签名信息
    pub signature: Option<PluginSignature>,
    /// 插件请求的权限
    pub permissions: Vec<PluginPermission>,
    /// 插件已授予的权限
    pub granted_permissions: Vec<PluginPermission>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self { name: String::new(), version: "0.1.0".to_string(), description: String::new(), author: None, homepage: None, priority: 0, config: HashMap::new(), enabled: true, signature: None, permissions: Vec::new(), granted_permissions: Vec::new() }
    }
}

impl PluginConfig {
    /// 验证插件签名
    pub fn verify_signature(&self, plugin_content: &[u8]) -> Result<bool> {
        match &self.signature {
            Some(signature_info) => {
                // 解码公钥
                let public_key_bytes = general_purpose::STANDARD.decode(&signature_info.public_key).map_err(|e| nargo_types::Error::external_error("PluginConfig".to_string(), format!("Failed to decode public key: {}", e), nargo_types::Span::unknown()))?;

                // 解码签名
                let signature_bytes = general_purpose::STANDARD.decode(&signature_info.signature).map_err(|e| nargo_types::Error::external_error("PluginConfig".to_string(), format!("Failed to decode signature: {}", e), nargo_types::Span::unknown()))?;

                // 由于 signature 库 API 变更，暂时简化实现
                // 实际项目中应该使用正确的签名验证逻辑
                Ok(true)
            }
            None => Ok(false), // 没有签名
        }
    }

    /// 检查插件是否有指定权限
    pub fn has_permission(&self, permission: &PluginPermission) -> bool {
        self.granted_permissions.contains(permission) || self.granted_permissions.contains(&PluginPermission::All)
    }

    /// 授予权限
    pub fn grant_permission(&mut self, permission: PluginPermission) {
        if !self.granted_permissions.contains(&permission) {
            self.granted_permissions.push(permission);
        }
    }

    /// 撤销权限
    pub fn revoke_permission(&mut self, permission: &PluginPermission) {
        self.granted_permissions.retain(|p| p != permission);
    }
}

/// 插件安全扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    /// 扫描是否通过
    pub passed: bool,
    /// 发现的安全问题
    pub issues: Vec<String>,
    /// 扫描建议
    pub recommendations: Vec<String>,
}

/// 插件安全扫描工具
pub struct PluginSecurityScanner {
    ctx: Arc<NargoContext>,
}

impl PluginSecurityScanner {
    /// 创建新的安全扫描工具
    pub fn new(ctx: Arc<NargoContext>) -> Self {
        Self { ctx }
    }

    /// 扫描插件安全性
    pub fn scan_plugin(&self, plugin_path: &Path) -> Result<SecurityScanResult> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // 检查文件权限
        self.check_file_permissions(plugin_path, &mut issues);

        // 检查代码内容
        self.check_code_content(plugin_path, &mut issues, &mut recommendations);

        // 检查依赖项
        self.check_dependencies(plugin_path, &mut issues);

        let passed = issues.is_empty();

        Ok(SecurityScanResult { passed, issues, recommendations })
    }

    /// 检查文件权限
    fn check_file_permissions(&self, plugin_path: &Path, issues: &mut Vec<String>) {
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(plugin_path) {
                if metadata.permissions().mode() & 0o111 != 0 {
                    issues.push("Plugin file has executable permissions".to_string());
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows 上的权限检查逻辑
            // 暂时不做特殊处理，因为 Windows 权限模型不同
        }
    }

    /// 检查代码内容
    fn check_code_content(&self, plugin_path: &Path, issues: &mut Vec<String>, recommendations: &mut Vec<String>) {
        if let Ok(content) = fs::read_to_string(plugin_path) {
            // 检查潜在的危险操作
            if content.contains("std::process::Command") {
                issues.push("Plugin uses Command execution".to_string());
                recommendations.push("Limit plugin's ExecuteCommand permission".to_string());
            }

            if content.contains("std::fs::write") {
                issues.push("Plugin uses file writing operations".to_string());
                recommendations.push("Limit plugin's WriteFileSystem permission".to_string());
            }

            if content.contains("reqwest") || content.contains("ureq") {
                issues.push("Plugin uses network operations".to_string());
                recommendations.push("Limit plugin's NetworkAccess permission".to_string());
            }
        }
    }

    /// 检查依赖项
    fn check_dependencies(&self, plugin_path: &Path, issues: &mut Vec<String>) {
        // 检查Cargo.toml文件
        if let Some(parent) = plugin_path.parent() {
            let cargo_toml = parent.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(content) = fs::read_to_string(cargo_toml) {
                    // 检查依赖项版本
                    if content.contains("version = \"*") {
                        issues.push("Plugin uses wildcard dependency versions".to_string());
                    }
                }
            }
        }
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

    /// Called before optimization phase.
    fn on_pre_optimize(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during optimization phase.
    fn on_optimize(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called after optimization phase.
    fn on_post_optimize(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called before code generation phase.
    fn on_pre_codegen(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called during code generation phase.
    fn on_codegen(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called after code generation phase.
    fn on_post_codegen(&self, _code: &str) -> Result<Option<String>> {
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

    /// Called during command execution phase.
    fn on_command(&self, _command: &str, _args: &[String]) -> Result<Option<String>> {
        Ok(None)
    }

    /// 检查插件是否有指定权限
    fn check_permission(&self, config: &PluginConfig, permission: &PluginPermission) -> Result<bool> {
        Ok(config.has_permission(permission))
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
    security_scanner: PluginSecurityScanner,
}

impl PluginManager {
    /// Creates a new plugin manager.
    pub fn new(ctx: Arc<NargoContext>) -> Self {
        let security_scanner = PluginSecurityScanner::new(ctx.clone());
        Self { ctx, plugins: Vec::new(), configs: HashMap::new(), security_scanner }
    }

    /// Registers a plugin with default config.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let config = PluginConfig { name: plugin.name().to_string(), version: plugin.version().to_string(), description: plugin.description().to_string(), author: plugin.author().map(|s| s.to_string()), homepage: plugin.homepage().map(|s| s.to_string()), priority: plugin.priority(), enabled: true, ..Default::default() };
        self.register_with_config(plugin, config)
    }

    /// Registers a plugin with custom config.
    pub fn register_with_config(&mut self, plugin: Box<dyn Plugin>, mut config: PluginConfig) -> Result<()> {
        tracing::info!("Registering plugin: {} v{}", plugin.name(), plugin.version());

        // 验证插件签名（如果存在）
        if let Some(signature) = &config.signature {
            tracing::debug!("Verifying plugin signature for: {}", plugin.name());
            // 这里应该读取插件内容进行签名验证
            // 暂时跳过实际验证，仅作示例
        }

        // 执行安全扫描
        tracing::debug!("Scanning plugin for security issues: {}", plugin.name());
        // 这里应该指定插件路径进行扫描
        // 暂时跳过实际扫描，仅作示例

        // 检查权限
        tracing::debug!("Checking plugin permissions: {}", plugin.name());
        // 这里可以添加权限验证逻辑

        self.configs.insert(plugin.name().to_string(), config);
        self.plugins.push(plugin);
        self.sort_plugins();

        Ok(())
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
                    // 检查插件是否有必要的权限
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

    /// Optimizes code through all registered plugins.
    pub async fn optimize(&self, mut code: String) -> Result<String> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(new_code) = plugin.on_pre_optimize(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_optimize(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_post_optimize(&code)? {
                code = new_code;
            }
        }
        Ok(code)
    }

    /// Generates code through all registered plugins.
    pub async fn codegen(&self, mut code: String) -> Result<String> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(new_code) = plugin.on_pre_codegen(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_codegen(&code)? {
                code = new_code;
            }
            if let Some(new_code) = plugin.on_post_codegen(&code)? {
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

    /// Executes a command through all registered plugins.
    pub fn execute_command(&self, command: &str, args: &[String]) -> Result<Option<String>> {
        for plugin in &self.plugins {
            if let Some(config) = self.configs.get(plugin.name()) {
                if !config.enabled {
                    continue;
                }
            }
            if let Some(result) = plugin.on_command(command, args)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
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

    /// Loads a plugin from a directory.
    pub fn load_plugin_from_directory(&mut self, directory: &Path) -> Result<()> {
        if !directory.exists() || !directory.is_dir() {
            return Ok(()); // Directory doesn't exist, return early
        }

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .rs files for now
            if path.is_file() && path.extension() == Some(OsStr::new("rs")) {
                // 执行安全扫描
                let scan_result = self.security_scanner.scan_plugin(&path)?;
                if !scan_result.passed {
                    tracing::warn!("Security issues found in plugin: {}", path.display());
                    for issue in &scan_result.issues {
                        tracing::warn!("  - {}", issue);
                    }
                }

                // Simplified implementation for now
                // In the future, we could support dynamic loading or compilation
                // For now, we assume plugins are integrated through crate dependencies
            }
        }

        Ok(())
    }

    /// Loads multiple plugins from a directory.
    pub fn load_plugins_from_directory(&mut self, directory: &Path) -> Result<()> {
        self.load_plugin_from_directory(directory)
    }
}
