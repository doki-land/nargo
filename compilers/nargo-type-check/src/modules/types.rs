//! 类型定义模块
//!
//! 包含类型检查器使用的各种类型定义

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 类型检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCheckResult {
    /// 错误数量
    pub errors: usize,
    /// 检查的文件数量
    pub checked_files: usize,
}

/// tsconfig.json 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsConfig {
    /// 编译选项
    pub compiler_options: Option<CompilerOptions>,
    /// 包含的文件
    pub include: Option<Vec<String>>,
    /// 排除的文件
    pub exclude: Option<Vec<String>>,
}

/// 编译选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOptions {
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 模块解析策略
    pub module_resolution: Option<String>,
    /// 是否生成声明文件
    pub declaration: Option<bool>,
    /// 是否启用严格模式
    pub strict: Option<bool>,
    /// 是否启用 noImplicitAny
    pub no_implicit_any: Option<bool>,
    /// 是否启用 strictNullChecks
    pub strict_null_checks: Option<bool>,
    /// 包含的类型声明文件
    pub types: Option<Vec<String>>,
    /// 基础 URL
    pub base_url: Option<String>,
    /// 路径映射
    pub paths: Option<HashMap<String, Vec<String>>>,
}

/// 类型错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeError {
    /// 错误位置
    pub position: (usize, usize),
    /// 错误消息
    pub message: String,
    /// 文件名
    pub file_name: Option<String>,
    /// 行号
    pub line: Option<usize>,
    /// 列号
    pub column: Option<usize>,
    /// 错误代码
    pub error_code: Option<String>,
    /// 相关类型信息
    pub related_types: Option<Vec<String>>,
    /// 建议的修复方案
    pub suggestion: Option<String>,
}

/// 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 基本类型
    Any,
    Void,
    Never,
    Unknown,
    Number,
    String,
    Boolean,
    Symbol,
    BigInt,
    /// 复合类型
    Array(Box<Type>),
    Object(HashMap<String, Type>),
    Function(Vec<Type>, Box<Type>),
    /// 类型变量
    TypeVar(String),
    /// 接口类型
    Interface(String),
    /// 泛型接口类型
    GenericInterface(String, Vec<Type>),
    /// 类型别名
    TypeAlias(String),
    /// 泛型类型别名
    GenericTypeAlias(String, Vec<Type>),
    /// 联合类型
    Union(Vec<Type>),
    /// 交叉类型
    Intersection(Vec<Type>),
    /// 条件类型: T extends U ? X : Y
    Conditional(Box<Type>, Box<Type>, Box<Type>, Box<Type>),
    /// 映射类型: { [K in keyof T]: T[K] }
    Mapped(String, Box<Type>, Box<Type>),
    /// 索引访问类型: T[K]
    IndexAccess(Box<Type>, Box<Type>),
    /// 关键字类型
    KeyOf(Box<Type>),
    /// 字面量类型
    Literal(String),
    /// 元组类型
    Tuple(Vec<Type>),
    /// 排除类型: Exclude<T, U>
    Exclude(Box<Type>, Box<Type>),
    /// 提取类型: Extract<T, U>
    Extract(Box<Type>, Box<Type>),
    /// this 类型: ThisType<T>
    ThisType(Box<Type>),
    /// 省略 this 参数类型: OmitThisParameter<T>
    OmitThisParameter(Box<Type>),
    /// this 参数类型: ThisParameterType<T>
    ThisParameterType(Box<Type>),
    /// 枚举类型
    Enum(String),
}

/// 接口定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDef {
    /// 接口成员
    pub members: HashMap<String, Type>,
    /// 继承的接口
    pub extends: Vec<String>,
}

/// 类型环境
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// 变量类型映射
    pub variables: HashMap<String, Type>,
    /// 函数类型映射
    pub functions: HashMap<String, Type>,
    /// 接口定义
    pub interfaces: HashMap<String, InterfaceDef>,
    /// 类型别名
    pub type_aliases: HashMap<String, Type>,
    /// 枚举定义
    pub enums: HashMap<String, HashSet<String>>,
}

impl TypeEnv {
    /// 创建新的类型环境
    pub fn new() -> Self {
        Self { variables: HashMap::new(), functions: HashMap::new(), interfaces: HashMap::new(), type_aliases: HashMap::new(), enums: HashMap::new() }
    }

    /// 添加变量类型
    pub fn add_variable(&mut self, name: String, ty: Type) {
        self.variables.insert(name, ty);
    }

    /// 添加函数类型
    pub fn add_function(&mut self, name: String, ty: Type) {
        self.functions.insert(name, ty);
    }

    /// 添加接口定义
    pub fn add_interface(&mut self, name: String, members: HashMap<String, Type>, extends: Vec<String>) {
        self.interfaces.insert(name, InterfaceDef { members, extends });
    }

    /// 添加类型别名
    pub fn add_type_alias(&mut self, name: String, ty: Type) {
        self.type_aliases.insert(name, ty);
    }

    /// 添加枚举定义
    pub fn add_enum(&mut self, name: String, variants: HashSet<String>) {
        self.enums.insert(name, variants);
    }

    /// 获取变量类型
    pub fn get_variable(&self, name: &str) -> Option<&Type> {
        self.variables.get(name)
    }

    /// 获取函数类型
    pub fn get_function(&self, name: &str) -> Option<&Type> {
        self.functions.get(name)
    }

    /// 获取接口定义
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceDef> {
        self.interfaces.get(name)
    }

    /// 获取类型别名
    pub fn get_type_alias(&self, name: &str) -> Option<&Type> {
        self.type_aliases.get(name)
    }

    /// 获取枚举定义
    pub fn get_enum(&self, name: &str) -> Option<&HashSet<String>> {
        self.enums.get(name)
    }
}
