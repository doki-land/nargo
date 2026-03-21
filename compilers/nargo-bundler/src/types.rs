#![warn(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

/// 文件指纹，用于检测文件变化
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileFingerprint {
    /// 文件修改时间
    pub modified_time: SystemTime,
    /// 文件内容哈希
    pub content_hash: u64,
}

/// 模块缓存项
#[derive(Debug, Clone)]
pub struct ModuleCache {
    /// 模块的文件指纹
    pub fingerprint: FileFingerprint,
    /// 编译后的代码
    pub compiled_code: String,
    /// 模块的依赖列表
    pub dependencies: HashSet<String>,
    /// 模块的特征集
    pub features: FeatureSet,
}

/// 代码分割策略枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitStrategy {
    /// 单文件打包，不进行代码分割
    SingleFile,
    /// 按路由分割，每个路由生成一个 chunk
    ByRoute,
    /// 按组件分割，每个组件生成一个 chunk
    ByComponent,
    /// 自定义分割策略
    Custom,
}

/// 模块系统枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleSystem {
    /// ES 模块
    ESM,
    /// CommonJS 模块
    CommonJS,
    /// UMD 模块
    UMD,
    /// IIFE 模块
    IIFE,
}

/// 代码生成目标格式枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// JavaScript 代码
    JavaScript,
    /// CSS 代码
    CSS,
    /// HTML 代码
    HTML,
    /// TypeScript 类型定义
    TypeScript,
    /// WebAssembly 代码
    WebAssembly,
}

/// 构建输出结果结构
#[derive(Debug, Clone)]
pub struct BuildOutput {
    /// 生成的代码
    pub code: Vec<u8>,
    /// 输出格式
    pub format: OutputFormat,
    /// 文件名
    pub filename: String,
}

/// 构建输出结果集合，用于代码分割
#[derive(Debug, Clone)]
pub struct BuildOutputs {
    /// 输出文件集合
    pub outputs: Vec<BuildOutput>,
    /// 入口文件
    pub entry_file: String,
}

/// 特征集，用于分析模块使用的功能
#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    /// 是否使用了信号
    pub has_signals: bool,
    /// 是否使用了副作用
    pub has_effects: bool,
    /// 是否使用了虚拟 DOM
    pub has_vdom: bool,
    /// 是否使用了 SSR
    pub has_ssr: bool,
    /// 使用的核心函数
    pub used_core_functions: HashSet<String>,
    /// 使用的 DOM 函数
    pub used_dom_functions: HashSet<String>,
}

impl FeatureSet {
    /// 合并另一个特征集到当前集
    pub fn merge(&mut self, other: &FeatureSet) {
        self.has_signals |= other.has_signals;
        self.has_effects |= other.has_effects;
        self.has_vdom |= other.has_vdom;
        self.has_ssr |= other.has_ssr;
        self.used_core_functions.extend(other.used_core_functions.iter().cloned());
        self.used_dom_functions.extend(other.used_dom_functions.iter().cloned());
    }
}
