//! 工具函数模块
//!
//! 包含类型检查器使用的各种工具函数

use regex::Regex;
use std::path::PathBuf;

use crate::modules::types::TsConfig;

/// 工具函数
pub trait Utils {
    /// 查找前端文件
    ///
    /// # 参数
    ///
    /// * `dir` - 目录路径
    ///
    /// # 返回
    ///
    /// 找到的前端文件列表
    fn find_frontend_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>, nargo_types::Error>;

    /// 递归查找文件
    ///
    /// # 参数
    ///
    /// * `dir` - 目录路径
    /// * `files` - 文件列表
    fn find_files_recursive(&self, dir: &PathBuf, files: &mut Vec<PathBuf>);

    /// 检查是否应该排除文件或目录
    ///
    /// # 参数
    ///
    /// * `path` - 路径
    ///
    /// # 返回
    ///
    /// 是否应该排除
    fn should_exclude(&self, path: &PathBuf) -> bool;

    /// 展开 glob 模式
    ///
    /// # 参数
    ///
    /// * `base_dir` - 基础目录
    /// * `pattern` - 模式
    ///
    /// # 返回
    ///
    /// 展开后的模式列表
    fn expand_glob_pattern(&self, base_dir: &PathBuf, pattern: &str) -> Vec<String>;

    /// 检查路径是否匹配模式
    ///
    /// # 参数
    ///
    /// * `path` - 路径
    /// * `pattern` - 模式
    ///
    /// # 返回
    ///
    /// 是否匹配
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool;
}
