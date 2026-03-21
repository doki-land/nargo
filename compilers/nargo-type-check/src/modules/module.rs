//! 模块解析模块
//!
//! 包含模块解析和处理相关的代码

use nargo_types::Error;
use std::path::PathBuf;

use crate::modules::types::{Type, TypeEnv};

/// 模块解析器
pub trait ModuleResolver {
    /// 解析模块导入
    ///
    /// # 参数
    ///
    /// * `source` - 导入源
    /// * `specifiers` - 导入说明符
    /// * `current_file` - 当前文件路径
    ///
    /// # 返回
    ///
    /// 解析结果
    fn resolve_import(&mut self, source: &str, specifiers: &[String], current_file: &PathBuf) -> Result<(), Error>;

    /// 解析模块路径
    ///
    /// # 参数
    ///
    /// * `source` - 导入源
    /// * `current_file` - 当前文件路径
    ///
    /// # 返回
    ///
    /// 解析后的模块路径
    fn resolve_module_path(&self, source: &str, current_file: &PathBuf) -> Result<String, Error>;

    /// 检查语句是否为模块相关语句（导入/导出）
    ///
    /// # 参数
    ///
    /// * `stmt` - 语句
    /// * `module_env` - 模块的类型环境
    /// * `module_path` - 模块路径
    ///
    /// # 返回
    ///
    /// 检查结果
    fn check_statement_for_module(&mut self, stmt: &nargo_ir::JsStmt, module_env: &mut TypeEnv, module_path: &str) -> Result<(), Error>;

    /// 将声明添加到模块环境
    ///
    /// # 参数
    ///
    /// * `stmt` - 声明语句
    /// * `module_env` - 模块的类型环境
    fn add_declaration_to_module(&self, stmt: &nargo_ir::JsStmt, module_env: &mut TypeEnv);

    /// 导入模块类型到当前环境
    ///
    /// # 参数
    ///
    /// * `module_env` - 模块的类型环境
    /// * `specifiers` - 导入说明符
    fn import_module_types(&mut self, module_env: &TypeEnv, specifiers: &[String]);

    /// 检查语句并处理模块导入导出
    ///
    /// # 参数
    ///
    /// * `stmt` - 语句
    /// * `file_path` - 文件路径
    ///
    /// # 返回
    ///
    /// 检查结果
    fn check_statement_with_module(&mut self, stmt: &nargo_ir::JsStmt, file_path: &str) -> Result<(), Error>;

    /// 将声明添加到当前环境
    ///
    /// # 参数
    ///
    /// * `stmt` - 声明语句
    fn add_declaration_to_current_env(&mut self, stmt: &nargo_ir::JsStmt);
}
