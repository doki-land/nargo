//! 静态资源处理模块
//! 提供文档站点所需静态资源的处理和复制功能

use std::result::Result;

/// 静态资源处理器
#[derive(Clone)]
pub struct StaticProcessor;

impl StaticProcessor {
    /// 创建新的静态资源处理器
    ///
    /// # 返回值
    /// 返回一个新的静态资源处理器实例
    pub fn new() -> Self {
        Self
    }

    /// 处理静态资源
    ///
    /// # 返回值
    /// 返回包含静态资源名称和内容的元组向量
    pub fn process(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 复制静态文件
    ///
    /// # 参数
    /// * `_input_dir` - 输入目录
    /// * `_output_dir` - 输出目录
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    pub fn copy_static_files(&self, _input_dir: &String, _output_dir: &String) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl Default for StaticProcessor {
    fn default() -> Self {
        Self::new()
    }
}
