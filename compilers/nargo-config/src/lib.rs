#![warn(missing_docs)]

use nargo_types::{NargoValue, Result};
use oak_json;
use oak_toml;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::path::PathBuf;

/// Nargo 配置文件的默认名称
pub const DEFAULT_CONFIG_FILE: &str = "Nargo.toml";

/// Nargo 配置文件的可能扩展名
pub const CONFIG_EXTENSIONS: &[&str] = &[".toml"];

/// Nargo 框架的配置结构
///
/// 用于管理 Nargo 项目的编译、构建和运行时配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NargoConfig {
    /// 项目名称
    pub name: Option<String>,

    /// 项目版本
    pub version: Option<String>,

    /// 编译配置
    pub build: Option<BuildConfig>,

    /// 开发服务器配置
    pub dev: Option<DevConfig>,

    /// 插件配置
    pub plugins: Option<Vec<PluginConfig>>,

    /// 路径别名配置
    pub aliases: Option<std::collections::HashMap<String, String>>,

    /// 格式化配置
    pub formatter: Option<NargoValue>,

    ///  lint 配置
    pub lint: Option<NargoValue>,
}

/// 编译配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// 输出目录
    pub out_dir: Option<PathBuf>,

    /// 是否启用生产模式
    pub prod: Option<bool>,

    /// 是否启用源码映射
    pub sourcemap: Option<bool>,

    /// 目标浏览器配置
    pub browserslist: Option<Vec<String>>,

    /// 构建目标
    pub targets: Option<Vec<String>>,
}

/// 开发服务器配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevConfig {
    /// 开发服务器端口
    pub port: Option<u16>,

    /// 开发服务器主机
    pub host: Option<String>,

    /// 是否启用热更新
    pub hot: Option<bool>,

    /// 代理配置
    pub proxy: Option<std::collections::HashMap<String, String>>,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,

    /// 插件选项
    pub options: Option<NargoValue>,
}

/// 配置加载器
///
/// 负责从文件系统加载和解析 Nargo 配置文件。
pub struct ConfigLoader {
    /// 配置文件路径
    pub config_path: PathBuf,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self { config_path: PathBuf::from(DEFAULT_CONFIG_FILE) }
    }
}

impl ConfigLoader {
    /// 创建一个新的配置加载器
    ///
    /// # Arguments
    ///
    /// * `config_path` - 配置文件路径
    ///
    /// # Returns
    ///
    /// 配置加载器实例
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 加载配置文件
    ///
    /// # Returns
    ///
    /// 配置结果
    pub fn load(&self) -> Result<NargoConfig> {
        use std::fs;

        let extension = self.config_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        match extension.as_str() {
            "toml" => {
                let content = fs::read_to_string(&self.config_path).map_err(nargo_types::Error::io_error)?;
                let config: NargoConfig = oak_toml::from_str(&content).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Invalid TOML config: {}", e), nargo_types::Span::default()))?;
                Ok(config)
            }
            "json" => {
                let content = fs::read_to_string(&self.config_path).map_err(nargo_types::Error::io_error)?;
                let config: NargoConfig = oak_json::from_str(&content).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Invalid JSON config: {}", e), nargo_types::Span::default()))?;
                Ok(config)
            }
            _ => Err(nargo_types::Error::external_error("config".to_string(), format!("Unsupported config file extension: {}", extension), nargo_types::Span::default())),
        }
    }

    /// 保存配置到文件
    ///
    /// # Arguments
    ///
    /// * `config` - 配置对象
    ///
    /// # Returns
    ///
    /// 保存结果
    pub fn save(&self, config: &NargoConfig) -> Result<()> {
        use std::fs;

        let extension = self.config_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        let content = match extension.as_str() {
            "toml" => to_string_pretty(config).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Failed to serialize TOML: {}", e), nargo_types::Span::default()))?,
            "json" => to_string_pretty(config).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Failed to serialize JSON: {}", e), nargo_types::Span::default()))?,
            _ => return Err(nargo_types::Error::external_error("config".to_string(), format!("Unsupported config file extension: {}", extension), nargo_types::Span::default())),
        };

        fs::write(&self.config_path, content).map_err(nargo_types::Error::io_error)?;
        Ok(())
    }

    /// 保存 NargoToml 配置到文件
    ///
    /// # Arguments
    ///
    /// * `config` - NargoToml 配置对象
    ///
    /// # Returns
    ///
    /// 保存结果
    pub fn save_toml(&self, config: &NargoToml) -> Result<()> {
        use std::fs;

        let content = to_string_pretty(config).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Failed to serialize TOML: {}", e), nargo_types::Span::default()))?;
        fs::write(&self.config_path, content).map_err(nargo_types::Error::io_error)?;
        Ok(())
    }

    /// 加载 NargoToml 配置
    ///
    /// # Returns
    ///
    /// 配置结果
    pub fn load_toml(&self) -> Result<NargoToml> {
        use std::fs;

        let content = fs::read_to_string(&self.config_path).map_err(nargo_types::Error::io_error)?;
        let config: NargoToml = oak_toml::from_str(&content).map_err(|e| nargo_types::Error::external_error("config".to_string(), format!("Invalid TOML config: {}", e), nargo_types::Span::default()))?;
        Ok(config)
    }

    /// 查找配置文件
    ///
    /// 在指定目录及其父目录中查找配置文件。
    ///
    /// # Arguments
    ///
    /// * `start_dir` - 开始查找的目录
    ///
    /// # Returns
    ///
    /// 找到的配置文件路径，或 None
    pub fn find_config_file(start_dir: &PathBuf) -> Option<PathBuf> {
        let mut current_dir = start_dir.clone();

        loop {
            let config_path = current_dir.join(DEFAULT_CONFIG_FILE);
            if config_path.exists() {
                return Some(config_path);
            }

            if !current_dir.pop() {
                break;
            }
        }

        None
    }
}

pub mod types;
pub use types::Dependency;

/// Nargo 项目配置文件结构
///
/// 用于管理 Nargo 项目的依赖、脚本、注册表等配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NargoToml {
    /// 包配置
    pub package: crate::types::PackageConfig,
    /// 依赖项
    pub dependencies: std::collections::HashMap<String, crate::types::Dependency>,
    /// 开发依赖项
    pub dev_dependencies: std::collections::HashMap<String, crate::types::Dependency>,
    /// 构建依赖项
    pub build_dependencies: std::collections::HashMap<String, crate::types::Dependency>,
    /// 工作区配置
    pub workspace: Option<crate::types::WorkspaceConfig>,
    /// 特性定义
    pub features: std::collections::HashMap<String, Vec<String>>,
    /// 配置文件
    pub profile: crate::types::ProfileConfig,
    /// 目标配置
    pub target: std::collections::HashMap<String, crate::types::TargetConfig>,
    /// 脚本配置
    pub scripts: std::collections::HashMap<String, crate::types::ScriptConfig>,
    /// 注册表配置
    pub registries: std::collections::HashMap<String, crate::types::RegistryEntry>,
    /// 安全配置
    pub security: crate::types::SecurityConfig,
    ///  lint 配置
    pub lint: crate::types::LintConfig,
    /// 格式化配置
    pub format: crate::types::FormatConfig,
}

impl NargoToml {
    /// 验证配置的有效性
    pub fn validate(&self) -> nargo_types::Result<()> {
        // 实现验证逻辑
        Ok(())
    }

    /// 获取依赖项
    pub fn get_dependency(&self, name: &str) -> Option<&crate::types::Dependency> {
        self.dependencies.get(name)
    }
}

/// 核心 API 的统一导出
pub mod prelude {
    pub use crate::{BuildConfig, CONFIG_EXTENSIONS, ConfigLoader, DEFAULT_CONFIG_FILE, DevConfig, NargoConfig, NargoToml, PluginConfig, types::*};
}
