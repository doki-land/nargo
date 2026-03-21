#![warn(missing_docs)]

//! 版本迁移工具
//!
//! 提供命令行工具，用于在不同版本的 SDK 之间进行迁移。

use nargo_metadata::ApiMetadata;
use nargo_types::Result;
use oak_json;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::VersionManager;

/// 版本迁移工具
///
/// 负责在不同版本的 SDK 之间进行迁移
pub struct MigrationTool;

impl MigrationTool {
    /// 从文件读取 API 元数据
    ///
    /// # Arguments
    /// * `path` - 文件路径
    ///
    /// # Returns
    /// * `Result<ApiMetadata>` - 读取的 API 元数据
    pub fn read_metadata(path: &Path) -> Result<ApiMetadata> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let metadata: ApiMetadata = oak_json::from_str(&content).map_err(|e| nargo_types::Error::external_error("oak_json".to_string(), e.to_string(), nargo_types::Span::unknown()))?;
        Ok(metadata)
    }

    /// 将 API 元数据写入文件
    ///
    /// # Arguments
    /// * `metadata` - API 元数据
    /// * `path` - 文件路径
    ///
    /// # Returns
    /// * `Result<()>` - 操作结果
    pub fn write_metadata(metadata: &ApiMetadata, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        let content = oak_json::to_string(metadata).map_err(|e| nargo_types::Error::external_error("oak_json".to_string(), e.to_string(), nargo_types::Span::unknown()))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// 执行版本迁移
    ///
    /// # Arguments
    /// * `input_path` - 输入文件路径
    /// * `output_path` - 输出文件路径
    /// * `target_version` - 目标版本
    ///
    /// # Returns
    /// * `Result<()>` - 操作结果
    pub fn migrate(input_path: &Path, output_path: &Path, target_version: &str) -> Result<()> {
        // 读取输入文件
        let metadata = Self::read_metadata(input_path)?;

        // 执行迁移
        let migrated_metadata = VersionManager::migrate_metadata(&metadata, target_version)?;

        // 写入输出文件
        Self::write_metadata(&migrated_metadata, output_path)?;

        Ok(())
    }
}
