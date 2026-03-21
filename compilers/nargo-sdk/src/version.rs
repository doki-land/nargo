#![warn(missing_docs)]

//! 版本管理模块
//!
//! 提供版本检测、兼容性检查和版本迁移功能。

use nargo_metadata::ApiMetadata;
use nargo_types::{Error, Result};
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// 版本管理器
///
/// 负责版本检测、兼容性检查和版本迁移
pub struct VersionManager;

impl VersionManager {
    /// 检查版本兼容性
    ///
    /// # Arguments
    /// * `metadata` - API元数据
    /// * `required_version` - 要求的版本范围
    ///
    /// # Returns
    /// * `Result<bool>` - 是否兼容
    pub fn check_compatibility(metadata: &ApiMetadata, required_version: &str) -> Result<bool> {
        let current_version = Version::parse(&metadata.version).map_err(|e| Error::external_error("VersionManager".to_string(), format!("Invalid version format: {}", e), nargo_types::Span::unknown()))?;

        let version_req = VersionReq::parse(required_version).map_err(|e| Error::external_error("VersionManager".to_string(), format!("Invalid version requirement: {}", e), nargo_types::Span::unknown()))?;

        Ok(version_req.matches(&current_version))
    }

    /// 迁移API元数据到目标版本
    ///
    /// # Arguments
    /// * `metadata` - API元数据
    /// * `target_version` - 目标版本
    ///
    /// # Returns
    /// * `Result<ApiMetadata>` - 迁移后的API元数据
    pub fn migrate_metadata(metadata: &ApiMetadata, target_version: &str) -> Result<ApiMetadata> {
        let current_version = Version::parse(&metadata.version).map_err(|e| Error::external_error("VersionManager".to_string(), format!("Invalid version format: {}", e), nargo_types::Span::unknown()))?;

        let target_version = Version::parse(target_version).map_err(|e| Error::external_error("VersionManager".to_string(), format!("Invalid target version format: {}", e), nargo_types::Span::unknown()))?;

        // 执行版本迁移
        if current_version < target_version {
            // 升级迁移
            Self::upgrade_metadata(metadata, &current_version, &target_version)
        }
        else if current_version > target_version {
            // 降级迁移
            Self::downgrade_metadata(metadata, &current_version, &target_version)
        }
        else {
            // 版本相同，无需迁移
            Ok(metadata.clone())
        }
    }

    /// 升级API元数据
    ///
    /// # Arguments
    /// * `metadata` - API元数据
    /// * `current_version` - 当前版本
    /// * `target_version` - 目标版本
    ///
    /// # Returns
    /// * `Result<ApiMetadata>` - 升级后的API元数据
    fn upgrade_metadata(metadata: &ApiMetadata, current_version: &Version, target_version: &Version) -> Result<ApiMetadata> {
        // 实现具体的升级逻辑
        // 这里只做简单的版本号更新，实际项目中需要根据版本差异进行具体的迁移
        let mut migrated_metadata = metadata.clone();
        migrated_metadata.version = target_version.to_string();

        // 示例：如果从1.0.0升级到2.0.0，可能需要添加某些字段或修改结构
        if current_version.major == 1 && target_version.major == 2 {
            // 执行v1到v2的迁移逻辑
            // 例如：添加新的字段、修改类型定义等
        }

        Ok(migrated_metadata)
    }

    /// 降级API元数据
    ///
    /// # Arguments
    /// * `metadata` - API元数据
    /// * `current_version` - 当前版本
    /// * `target_version` - 目标版本
    ///
    /// # Returns
    /// * `Result<ApiMetadata>` - 降级后的API元数据
    fn downgrade_metadata(metadata: &ApiMetadata, current_version: &Version, target_version: &Version) -> Result<ApiMetadata> {
        // 实现具体的降级逻辑
        // 这里只做简单的版本号更新，实际项目中需要根据版本差异进行具体的迁移
        let mut migrated_metadata = metadata.clone();
        migrated_metadata.version = target_version.to_string();

        // 示例：如果从2.0.0降级到1.0.0，可能需要移除某些字段或修改结构
        if current_version.major == 2 && target_version.major == 1 {
            // 执行v2到v1的迁移逻辑
            // 例如：移除不兼容的字段、修改类型定义等
        }

        Ok(migrated_metadata)
    }

    /// 获取版本兼容性信息
    ///
    /// # Arguments
    /// * `metadata` - API元数据
    ///
    /// # Returns
    /// * `HashMap<String, String>` - 兼容性信息
    pub fn get_compatibility_info(metadata: &ApiMetadata) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("current_version".to_string(), metadata.version.clone());
        info.insert("compatible_versions".to_string(), "^1.0.0".to_string());
        info.insert("minimum_required_version".to_string(), "1.0.0".to_string());
        info
    }
}
