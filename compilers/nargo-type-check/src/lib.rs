#![warn(missing_docs)]

use console;
use glob;
use nargo_ir::{JsExpr, JsStmt, TseAttribute};
use nargo_parser::{Parser, ParserRegistry};
use nargo_script_analyzer::ScriptAnalyzer;
use nargo_types::{Error, Result, Result as NargoResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::PathBuf,
    sync::Arc,
};
use walkdir::WalkDir;

// 导入模块
use crate::modules::{
    check::ExpressionChecker,
    module::ModuleResolver,
    template::TemplateChecker,
    types::{TsConfig, TypeEnv},
    utils::Utils,
};

// 导入所有模块
mod modules;

/// 导出类型和特性
pub use modules::types::{Type, TypeCheckResult, TypeError};
pub use modules::typescript::TypeScriptStmtHandler;

/// 文件缓存信息
#[derive(Debug, Clone)]
pub struct FileCache {
    /// 文件的修改时间
    pub modified_time: std::time::SystemTime,
    /// 文件的类型环境
    pub type_env: TypeEnv,
    /// 文件的依赖项
    pub dependencies: Vec<String>,
    /// 依赖此文件的文件
    pub dependents: Vec<String>,
}

/// 类型检查器
///
/// 负责执行 HXO 文件的类型检查
pub struct TypeChecker {
    /// 解析器注册表
    registry: Arc<ParserRegistry>,
    /// 脚本分析器
    analyzer: ScriptAnalyzer,
    /// 类型环境
    type_env: TypeEnv,
    /// 类型错误
    errors: Vec<TypeError>,
    /// tsconfig.json 配置
    config: Option<TsConfig>,
    /// 模块映射，存储已解析的模块
    modules: HashMap<String, TypeEnv>,
    /// 正在解析的模块，用于检测循环依赖
    parsing_modules: HashSet<String>,
    /// 当前正在检查的文件名
    current_file: Option<String>,
    /// 文件缓存，存储已检查过的文件的类型信息
    file_cache: HashMap<String, FileCache>,
    /// 文件依赖图
    file_dependencies: HashMap<String, Vec<String>>,
}

impl TypeChecker {
    /// 创建新的类型检查器实例
    pub fn new() -> Self {
        // 创建解析器注册表
        let mut registry = ParserRegistry::new();

        // 注册默认解析器
        let template_parser = Arc::new(nargo_parser::OakVueTemplateParser);
        registry.register_template_parser("nargo", template_parser.clone());
        registry.register_template_parser("html", template_parser);

        let ts_parser = Arc::new(nargo_parser::OakTypeScriptParser);
        registry.register_script_parser("ts", ts_parser.clone());
        registry.register_script_parser("typescript", ts_parser.clone());
        registry.register_script_parser("js", ts_parser.clone());
        registry.register_script_parser("javascript", ts_parser);

        let css_parser = Arc::new(nargo_parser::OakCssParser);
        registry.register_style_parser("css", css_parser.clone());
        registry.register_style_parser("tailwind", css_parser);

        // 初始化类型环境，添加一些内置类型
        let mut type_env = TypeEnv::new();

        // 添加内置类型
        type_env.add_variable("undefined".to_string(), Type::Void);
        type_env.add_variable("null".to_string(), Type::Void);
        type_env.add_variable("true".to_string(), Type::Boolean);
        type_env.add_variable("false".to_string(), Type::Boolean);

        Self { registry: Arc::new(registry), analyzer: ScriptAnalyzer::new(), type_env, errors: Vec::new(), config: None, modules: HashMap::new(), parsing_modules: HashSet::new(), current_file: None, file_cache: HashMap::new(), file_dependencies: HashMap::new() }
    }

    /// 解析 tsconfig.json 配置文件
    ///
    /// # 参数
    ///
    /// * `path` - 配置文件路径
    ///
    /// # 返回
    ///
    /// 解析结果
    pub fn parse_tsconfig(&mut self, path: &PathBuf) -> NargoResult<()> {
        let content = std::fs::read_to_string(path).map_err(Error::io_error)?;
        let config: TsConfig = serde_json::from_str(&content).map_err(|e| Error::parse_error(format!("解析 tsconfig.json 失败: {}", e), nargo_types::Span::unknown()))?;
        self.config = Some(config);
        Ok(())
    }

    /// 从项目目录加载 tsconfig.json
    ///
    /// # 参数
    ///
    /// * `dir` - 项目目录
    ///
    /// # 返回
    ///
    /// 加载结果
    pub fn load_tsconfig(&mut self, dir: &PathBuf) -> NargoResult<()> {
        let tsconfig_path = dir.join("tsconfig.json");
        if tsconfig_path.exists() {
            self.parse_tsconfig(&tsconfig_path)?;
        }
        Ok(())
    }

    /// 类型检查单个文件
    ///
    /// # 参数
    ///
    /// * `file` - 文件路径
    /// * `source` - 文件内容
    ///
    /// # 返回
    ///
    /// 检查结果，包含错误信息
    pub fn check_file(&mut self, file: &PathBuf, source: &str) -> NargoResult<()> {
        let name = file.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let file_path_str = file.to_string_lossy().to_string();

        // 检查文件是否在缓存中，并且修改时间没有变化
        if let Ok(metadata) = file.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                if let Some(cache) = self.file_cache.get(&file_path_str) {
                    if cache.modified_time == modified_time {
                        // 使用缓存的类型环境
                        self.type_env = cache.type_env.clone();
                        return Ok(());
                    }
                }
            }
        }

        // 设置当前文件名
        self.current_file = Some(file_path_str.clone());

        // 清空当前文件的错误
        self.errors.clear();

        // 解析文件内容
        let mut parser = Parser::new(name, source, self.registry.clone());
        let ir = parser.parse_all()?;

        // 分析脚本
        if let Some(script) = &ir.script {
            // 先进行脚本分析
            let _meta = self.analyzer.analyze(script)?;

            // 执行类型检查，处理模块导入导出
            for stmt in &script.body {
                self.check_statement_with_module(stmt, &file_path_str)?;
            }
        }

        // 检查模板
        if let Some(template) = &ir.template {
            self.check_template_impl(template);
        }

        // 检查是否有类型错误
        if !self.errors.is_empty() {
            // 格式化错误消息
            let error_messages: Vec<String> = self
                .errors
                .iter()
                .map(|e| {
                    let file_name = e.file_name.as_ref().unwrap_or(&file_path_str);
                    let line = e.line.unwrap_or(0);
                    let column = e.column.unwrap_or(0);
                    let default_error_code = "TS0000".to_string();
                    let error_code = e.error_code.as_ref().unwrap_or(&default_error_code);
                    let mut message = format!("{}:{}:{} - error {}: {}", file_name, line, column, error_code, e.message);

                    // 添加相关类型信息
                    if let Some(related_types) = &e.related_types {
                        if !related_types.is_empty() {
                            message.push_str(&format!("\n  相关类型: {}", related_types.join(", ")));
                        }
                    }

                    // 添加建议的修复方案
                    if let Some(suggestion) = &e.suggestion {
                        message.push_str(&format!("\n  建议: {}", suggestion));
                    }

                    message
                })
                .collect();
            return Err(Error::parse_error(error_messages.join("\n"), nargo_types::Span::unknown()));
        }

        // 收集当前文件的依赖项
        let dependencies = self.collect_file_dependencies(&file_path_str);

        // 更新依赖图
        self.update_file_dependencies(&file_path_str, &dependencies);

        // 更新缓存
        if let Ok(metadata) = file.metadata() {
            if let Ok(modified_time) = metadata.modified() {
                let cache = FileCache { modified_time, type_env: self.type_env.clone(), dependencies: dependencies.clone(), dependents: self.get_file_dependents(&file_path_str) };
                self.file_cache.insert(file_path_str.clone(), cache);
            }
        }

        // 重置当前文件名
        self.current_file = None;

        Ok(())
    }

    /// 类型检查整个项目
    ///
    /// # 参数
    ///
    /// * `input` - 项目路径
    ///
    /// # 返回
    ///
    /// 类型检查结果
    pub async fn check_project(&mut self, input: PathBuf) -> NargoResult<TypeCheckResult> {
        // 加载 tsconfig.json 配置
        self.load_tsconfig(&input)?;

        let files = self.find_frontend_files(&input)?;
        let mut errors = 0;
        let checked_files = files.len();

        // 并行处理文件，提高性能
        use rayon::prelude::*;
        let results: Vec<(PathBuf, std::result::Result<(), Error>)> = files
            .par_iter()
            .map(|file| {
                let source = std::fs::read_to_string(file).map_err(Error::io_error);
                match source {
                    Ok(source) => {
                        let mut checker = TypeChecker::new();
                        checker.load_tsconfig(&input).unwrap();
                        (file.clone(), checker.check_file(file, &source))
                    }
                    Err(e) => (file.clone(), Err(e)),
                }
            })
            .collect();

        // 收集错误
        for (file, result) in results {
            if let Err(e) = result {
                eprintln!("{} {}: {}", console::style("✘").red(), file.display(), e);
                errors += 1;
            }
        }

        Ok(TypeCheckResult { errors, checked_files })
    }
}

// 实现模块解析器
impl ModuleResolver for TypeChecker {
    fn resolve_import(&mut self, source: &str, specifiers: &[String], current_file: &PathBuf) -> NargoResult<()> {
        // 解析模块路径
        let module_path = self.resolve_module_path(source, current_file)?;

        // 检查是否有循环依赖
        if self.parsing_modules.contains(&module_path) {
            return Err(Error::parse_error("循环依赖检测到".to_string(), nargo_types::Span::unknown()));
        }

        // 检查模块是否已经解析过
        if self.modules.contains_key(&module_path) {
            // 先获取模块环境的克隆，避免可变借用冲突
            let module_env = self.modules.get(&module_path).unwrap().clone();
            // 导入模块的类型到当前环境
            self.import_module_types(&module_env, specifiers);
            return Ok(());
        }

        // 标记模块正在解析
        self.parsing_modules.insert(module_path.clone());

        // 读取并解析模块文件
        let source_code = std::fs::read_to_string(&module_path).map_err(Error::io_error)?;
        let module_path_buf = PathBuf::from(&module_path);
        let module_name = module_path_buf.file_stem().unwrap_or_default().to_string_lossy().to_string();

        let mut parser = Parser::new(module_name, &source_code, self.registry.clone());
        let ir = parser.parse_all()?;

        // 创建模块的类型环境
        let mut module_env = TypeEnv::new();

        // 检查模块的脚本部分
        if let Some(script) = &ir.script {
            // 分析脚本
            let _meta = self.analyzer.analyze(script)?;

            // 检查模块的语句，处理导出
            for stmt in &script.body {
                self.check_statement_for_module(stmt, &mut module_env, &module_path)?;
            }
        }

        // 存储模块环境
        let module_env_clone = module_env.clone();
        self.modules.insert(module_path.clone(), module_env_clone);

        // 导入模块的类型到当前环境
        self.import_module_types(&module_env, specifiers);

        // 移除正在解析的标记
        self.parsing_modules.remove(&module_path);

        Ok(())
    }

    fn resolve_module_path(&self, source: &str, current_file: &PathBuf) -> NargoResult<String> {
        let current_dir = current_file.parent().unwrap_or_else(|| std::path::Path::new("."));

        // 处理相对路径
        if source.starts_with("./") || source.starts_with("../") {
            let module_path = current_dir.join(source);
            // 尝试添加文件扩展名
            let extensions = [".ts", ".tsx", ".js", ".jsx"];

            for ext in &extensions {
                let path_with_ext = module_path.with_extension(ext.trim_start_matches('.'));
                if path_with_ext.exists() {
                    return Ok(path_with_ext.to_string_lossy().to_string());
                }
            }

            // 尝试目录下的 index 文件
            let index_path = module_path.join("index");
            for ext in &extensions {
                let index_with_ext = index_path.with_extension(ext.trim_start_matches('.'));
                if index_with_ext.exists() {
                    return Ok(index_with_ext.to_string_lossy().to_string());
                }
            }

            return Err(Error::parse_error(format!("模块未找到: {}", source), nargo_types::Span::unknown()));
        }

        // 处理绝对路径（基于 tsconfig.json 的 baseUrl 和 paths）
        if let Some(config) = &self.config {
            if let Some(compiler_options) = &config.compiler_options {
                // 处理 paths 配置
                if let Some(paths) = &compiler_options.paths {
                    for (pattern, substitutions) in paths {
                        // 简化实现，实际需要更复杂的路径模式匹配
                        if source.starts_with(pattern) {
                            for substitution in substitutions {
                                let substituted_path = source.replace(pattern, substitution);
                                let base_path = if let Some(base_url) = &compiler_options.base_url { current_dir.join(base_url) } else { current_dir.to_path_buf() };
                                let module_path = base_path.join(substituted_path);

                                // 尝试添加文件扩展名
                                let extensions = [".ts", ".tsx", ".js", ".jsx"];

                                for ext in &extensions {
                                    let path_with_ext = module_path.with_extension(ext.trim_start_matches('.'));
                                    if path_with_ext.exists() {
                                        return Ok(path_with_ext.to_string_lossy().to_string());
                                    }
                                }

                                // 尝试目录下的 index 文件
                                let index_path = module_path.join("index");
                                for ext in &extensions {
                                    let index_with_ext = index_path.with_extension(ext.trim_start_matches('.'));
                                    if index_with_ext.exists() {
                                        return Ok(index_with_ext.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // 处理 baseUrl 配置
                if let Some(base_url) = &compiler_options.base_url {
                    let base_path = current_dir.join(base_url);
                    let module_path = base_path.join(source);

                    // 尝试添加文件扩展名
                    let extensions = [".ts", ".tsx", ".js", ".jsx"];

                    for ext in &extensions {
                        let path_with_ext = module_path.with_extension(ext.trim_start_matches('.'));
                        if path_with_ext.exists() {
                            return Ok(path_with_ext.to_string_lossy().to_string());
                        }
                    }

                    // 尝试目录下的 index 文件
                    let index_path = module_path.join("index");
                    for ext in &extensions {
                        let index_with_ext = index_path.with_extension(ext.trim_start_matches('.'));
                        if index_with_ext.exists() {
                            return Ok(index_with_ext.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // 处理 node_modules 模块
        let node_modules_path = current_dir.join("node_modules").join(source);
        let extensions = [".ts", ".tsx", ".js", ".jsx"];

        for ext in &extensions {
            let path_with_ext = node_modules_path.with_extension(ext.trim_start_matches('.'));
            if path_with_ext.exists() {
                return Ok(path_with_ext.to_string_lossy().to_string());
            }
        }

        // 尝试目录下的 index 文件
        let index_path = node_modules_path.join("index");
        for ext in &extensions {
            let index_with_ext = index_path.with_extension(ext.trim_start_matches('.'));
            if index_with_ext.exists() {
                return Ok(index_with_ext.to_string_lossy().to_string());
            }
        }

        // 处理绝对路径（暂时只支持相对路径）
        Err(Error::parse_error(format!("不支持的模块路径: {}", source), nargo_types::Span::unknown()))
    }

    fn check_statement_for_module(&mut self, stmt: &nargo_ir::JsStmt, module_env: &mut TypeEnv, module_path: &str) -> NargoResult<()> {
        match stmt {
            nargo_ir::JsStmt::Import { source, specifiers, .. } => {
                // 解析导入
                let current_file = PathBuf::from(module_path);
                self.resolve_import(source, specifiers, &current_file)?;
            }
            nargo_ir::JsStmt::Export { declaration, .. } => {
                // 处理导出声明
                self.check_statement(declaration);
                // 将导出的类型添加到模块环境
                self.add_declaration_to_module(declaration, module_env);
            }
            nargo_ir::JsStmt::ExportAll { source, .. } => {
                // 处理导出所有
                let current_file = PathBuf::from(module_path);
                self.resolve_import(source, &["*".to_string()], &current_file)?;
            }
            nargo_ir::JsStmt::ExportNamed { source, specifiers, .. } => {
                // 处理命名导出
                if let Some(source) = source {
                    let current_file = PathBuf::from(module_path);
                    self.resolve_import(source, specifiers, &current_file)?;
                }
                // TODO: 处理本地命名导出
            }
            _ => {
                // 检查普通语句
                self.check_statement(stmt);
            }
        }
        Ok(())
    }

    fn add_declaration_to_module(&self, stmt: &nargo_ir::JsStmt, module_env: &mut TypeEnv) {
        match stmt {
            nargo_ir::JsStmt::FunctionDecl { id, params, body, .. } => {
                // 解析参数类型
                let mut param_types = Vec::new();
                for _param in params {
                    param_types.push(Type::Any);
                }

                // 解析返回类型
                let return_type = Type::Any;

                // 创建函数类型
                let func_type = Type::Function(param_types, Box::new(return_type));

                // 添加函数到模块环境
                module_env.add_function(id.clone(), func_type);
            }
            nargo_ir::JsStmt::VariableDecl { id, init, .. } => {
                // 解析类型注解（如果有）
                let mut var_type = Type::Any;

                // 检查初始化表达式的类型
                if let Some(init_expr) = init {
                    // 这里需要实际的类型检查，暂时简化为 Any
                }

                // 添加变量到模块环境
                module_env.add_variable(id.clone(), var_type);
            }
            _ => {
                // 其他声明类型，暂时忽略
            }
        }
    }

    fn import_module_types(&mut self, module_env: &TypeEnv, specifiers: &[String]) {
        for specifier in specifiers {
            if specifier == "*" {
                // 导入所有类型
                for (name, ty) in &module_env.variables {
                    self.type_env.add_variable(name.clone(), ty.clone());
                }
                for (name, ty) in &module_env.functions {
                    self.type_env.add_function(name.clone(), ty.clone());
                }
            }
            else {
                // 导入指定类型
                if let Some(ty) = module_env.get_variable(specifier) {
                    self.type_env.add_variable(specifier.clone(), ty.clone());
                }
                if let Some(ty) = module_env.get_function(specifier) {
                    self.type_env.add_function(specifier.clone(), ty.clone());
                }
            }
        }
    }

    fn check_statement_with_module(&mut self, stmt: &nargo_ir::JsStmt, file_path: &str) -> NargoResult<()> {
        // 保存当前文件名，用于错误信息
        let file_name = PathBuf::from(file_path).file_name().unwrap_or_default().to_string_lossy().to_string();

        match stmt {
            nargo_ir::JsStmt::Import { source, specifiers, .. } => {
                // 解析导入
                let current_file = PathBuf::from(file_path);
                self.resolve_import(source, specifiers, &current_file)?;
            }
            nargo_ir::JsStmt::Export { declaration, .. } => {
                // 处理导出声明
                self.check_statement(declaration);
                // 将导出的类型添加到当前环境
                self.add_declaration_to_current_env(declaration);
            }
            nargo_ir::JsStmt::ExportAll { source, .. } => {
                // 处理导出所有
                let current_file = PathBuf::from(file_path);
                self.resolve_import(source, &["*".to_string()], &current_file)?;
            }
            nargo_ir::JsStmt::ExportNamed { source, specifiers, .. } => {
                // 处理命名导出
                if let Some(source) = source {
                    let current_file = PathBuf::from(file_path);
                    self.resolve_import(source, specifiers, &current_file)?;
                }
                // TODO: 处理本地命名导出
            }
            _ => {
                // 检查普通语句
                self.check_statement(stmt);
            }
        }
        Ok(())
    }

    fn add_declaration_to_current_env(&mut self, stmt: &nargo_ir::JsStmt) {
        match stmt {
            nargo_ir::JsStmt::FunctionDecl { id, params, .. } => {
                // 解析参数类型
                let mut param_types = Vec::new();
                for _param in params {
                    param_types.push(Type::Any);
                }

                // 解析返回类型
                let return_type = Type::Any;

                // 创建函数类型
                let func_type = Type::Function(param_types, Box::new(return_type));

                // 添加函数到当前环境
                self.type_env.add_function(id.clone(), func_type);
            }
            nargo_ir::JsStmt::VariableDecl { id, .. } => {
                // 解析类型注解（如果有）
                let var_type = Type::Any;

                // 添加变量到当前环境
                self.type_env.add_variable(id.clone(), var_type);
            }
            _ => {
                // 其他声明类型，暂时忽略
            }
        }
    }
}

// 实现 TypeScript 语句处理器
impl TypeScriptStmtHandler for TypeChecker {
    fn handle_typescript_stmt(&mut self, stmt_str: &str) {
        // 简单的正则表达式匹配，实际实现需要更复杂的解析
        if stmt_str.starts_with("interface ") {
            // 处理接口定义
            self.handle_interface_stmt(stmt_str);
        }
        else if stmt_str.starts_with("type ") {
            // 处理类型别名
            self.handle_type_alias_stmt(stmt_str);
        }
        else if stmt_str.starts_with("enum ") {
            // 处理枚举定义
            // Handle enum statements (not yet implemented)
            // self.handle_enum_stmt(stmt_str);
        }
    }

    // Enum statement handling will be implemented in the future

    fn handle_interface_def(&mut self, name: String, members: HashMap<String, Type>, extends: Vec<String>) {
        self.type_env.add_interface(name, members, extends);
    }

    fn handle_type_alias(&mut self, name: String, ty: Type) {
        self.type_env.add_type_alias(name, ty);
    }

    fn handle_generic_type(&self, name: String, type_args: Vec<Type>) -> Type {
        // 返回泛型接口类型
        Type::GenericInterface(name, type_args)
    }

    fn handle_union_type(&self, types: Vec<Type>) -> Type {
        Type::Union(types)
    }

    fn handle_intersection_type(&self, types: Vec<Type>) -> Type {
        Type::Intersection(types)
    }

    fn handle_conditional_type(&self, check_type: Type, extends_type: Type, true_type: Type, false_type: Type) -> Type {
        Type::Conditional(Box::new(check_type), Box::new(extends_type), Box::new(true_type), Box::new(false_type))
    }

    fn handle_mapped_type(&self, key_var: String, source_type: Type, mapped_type: Type) -> Type {
        Type::Mapped(key_var, Box::new(source_type), Box::new(mapped_type))
    }

    fn handle_index_access_type(&self, object_type: Type, index_type: Type) -> Type {
        Type::IndexAccess(Box::new(object_type), Box::new(index_type))
    }

    fn handle_keyof_type(&self, target_type: Type) -> Type {
        Type::KeyOf(Box::new(target_type))
    }

    fn handle_literal_type(&self, literal: String) -> Type {
        Type::Literal(literal)
    }

    fn handle_this_type(&self, this_type: Type) -> Type {
        Type::ThisType(Box::new(this_type))
    }

    fn handle_omit_this_parameter_type(&self, target_type: Type) -> Type {
        Type::OmitThisParameter(Box::new(target_type))
    }

    fn handle_this_parameter_type(&self, target_type: Type) -> Type {
        Type::ThisParameterType(Box::new(target_type))
    }

    fn handle_interface_stmt(&mut self, stmt_str: &str) {
        // 简单的解析，实际实现需要更复杂的语法分析
        // 示例: interface User { name: string; age: number; }
        let re = Regex::new(r"interface\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:extends\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\s*,\s*[a-zA-Z_][a-zA-Z0-9_]*)*)?)?\s*\{([\s\S]*?)\}").unwrap();
        if let Some(captures) = re.captures(stmt_str) {
            let name = captures[1].to_string();
            let extends = if let Some(extends_str) = captures.get(2) { extends_str.as_str().split(",").map(|s| s.trim().to_string()).collect() } else { Vec::new() };
            let members_str = captures[3].to_string();

            // 解析接口成员
            let members = self.parse_interface_members(&members_str);

            // 添加接口定义
            self.handle_interface_def(name, members, extends);
        }
    }

    fn parse_interface_members(&self, members_str: &str) -> HashMap<String, Type> {
        let mut members = HashMap::new();

        // 更复杂的解析，支持可选属性、只读属性和函数类型
        let member_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:\?|readonly)?\s*:\s*([^{;]+);").unwrap();
        for capture in member_re.captures_iter(members_str) {
            let name = capture[1].to_string();
            let type_str = capture[2].trim().to_string();
            let ty = self.parse_type(&type_str);
            members.insert(name, ty);
        }

        members
    }

    fn handle_type_alias_stmt(&mut self, stmt_str: &str) {
        // 更复杂的解析，支持泛型类型别名
        // 示例: type User<T> = { name: string; age: number; data: T };
        let re = Regex::new(r"type\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:<([^>]+)>)?\s*=\s*([\s\S]*?);").unwrap();
        if let Some(captures) = re.captures(stmt_str) {
            let name = captures[1].to_string();
            let type_str = captures[3].to_string();
            let ty = self.parse_type(&type_str);
            self.handle_type_alias(name, ty);
        }
    }

    fn parse_type(&self, type_str: &str) -> Type {
        let type_str = type_str.trim();

        // 处理基本类型
        match type_str {
            "any" => Type::Any,
            "void" => Type::Void,
            "never" => Type::Never,
            "unknown" => Type::Unknown,
            "number" => Type::Number,
            "string" => Type::String,
            "boolean" => Type::Boolean,
            "symbol" => Type::Symbol,
            "bigint" => Type::BigInt,
            _ => {
                // 处理泛型类型
                if let Some((name, type_args)) = self.parse_generic_type(type_str) {
                    // 处理 TypeScript 高级类型
                    match name.as_str() {
                        "Partial" => return self.handle_partial_type(type_args),
                        "Required" => return self.handle_required_type(type_args),
                        "Pick" => return self.handle_pick_type(type_args),
                        "Omit" => return self.handle_omit_type(type_args),
                        "Record" => return self.handle_record_type(type_args),
                        "Exclude" => return self.handle_exclude_type(type_args),
                        "Extract" => return self.handle_extract_type(type_args),
                        "NonNullable" => return self.handle_non_nullable_type(type_args),
                        "Parameters" => return self.handle_parameters_type(type_args),
                        "ReturnType" => return self.handle_return_type(type_args),
                        "InstanceType" => return self.handle_instance_type(type_args),
                        "ThisType" => return self.handle_this_type(type_args),
                        "OmitThisParameter" => return self.handle_omit_this_parameter_type(type_args),
                        "ThisParameterType" => return self.handle_this_parameter_type(type_args),
                        _ => return Type::GenericInterface(name, type_args),
                    }
                }

                // 处理复合类型
                if type_str.ends_with("[]") {
                    // 数组类型
                    let elem_type_str = type_str.trim_end_matches("[]");
                    let elem_type = self.parse_type(elem_type_str);
                    Type::Array(Box::new(elem_type))
                }
                else if type_str.contains("|") {
                    // 联合类型
                    let types = type_str.split("|").map(|t| self.parse_type(t.trim())).collect();
                    Type::Union(types)
                }
                else if type_str.contains("&") {
                    // 交叉类型
                    let types = type_str.split("&").map(|t| self.parse_type(t.trim())).collect();
                    Type::Intersection(types)
                }
                else if type_str.starts_with("{") && type_str.ends_with("}") {
                    // 对象类型或映射类型
                    let content = &type_str[1..type_str.len() - 1];
                    if content.contains("in") {
                        // 映射类型: { [K in keyof T]: T[K] }
                        // 简化解析，实际实现需要更复杂的语法分析
                        let parts: Vec<&str> = content.split(":").collect();
                        if parts.len() == 2 {
                            let key_part = parts[0].trim();
                            let value_type_str = parts[1].trim();

                            // 提取键变量和源类型
                            let key_re = Regex::new(r"\[(.*?)\s+in\s+(.*?)\]").unwrap();
                            if let Some(captures) = key_re.captures(key_part) {
                                let key_var = captures[1].trim().to_string();
                                let source_type_str = captures[2].trim();
                                let source_type = self.parse_type(source_type_str);
                                let mapped_type = self.parse_type(value_type_str);
                                return self.handle_mapped_type(key_var, source_type, mapped_type);
                            }
                        }
                        // 默认为对象类型
                        let members = self.parse_interface_members(content);
                        Type::Object(members)
                    }
                    else {
                        // 对象类型
                        let members = self.parse_interface_members(content);
                        Type::Object(members)
                    }
                }
                else if type_str.contains("extends") && type_str.contains("?") && type_str.contains(":") {
                    // 条件类型: T extends U ? X : Y
                    // 简化解析，实际实现需要更复杂的语法分析
                    let parts: Vec<&str> = type_str.split("?").collect();
                    if parts.len() == 2 {
                        let extends_part = parts[0].trim();
                        let result_parts: Vec<&str> = parts[1].split(":").collect();
                        if result_parts.len() == 2 {
                            let true_type_str = result_parts[0].trim();
                            let false_type_str = result_parts[1].trim();

                            let extends_parts: Vec<&str> = extends_part.split("extends").collect();
                            if extends_parts.len() == 2 {
                                let check_type_str = extends_parts[0].trim();
                                let extends_type_str = extends_parts[1].trim();

                                let check_type = self.parse_type(check_type_str);
                                let extends_type = self.parse_type(extends_type_str);
                                let true_type = self.parse_type(true_type_str);
                                let false_type = self.parse_type(false_type_str);

                                return self.handle_conditional_type(check_type, extends_type, true_type, false_type);
                            }
                        }
                    }
                    // 默认为接口类型
                    Type::Interface(type_str.to_string())
                }
                else if type_str.starts_with("keyof ") {
                    // KeyOf 类型: keyof T
                    let target_type_str = type_str.trim_start_matches("keyof ").trim();
                    let target_type = self.parse_type(target_type_str);
                    self.handle_keyof_type(target_type)
                }
                else if type_str.contains("[") && type_str.contains("]") {
                    // 索引访问类型: T[K]
                    let parts: Vec<&str> = type_str.split("[").collect();
                    if parts.len() == 2 {
                        let object_type_str = parts[0].trim();
                        let index_type_str = parts[1].trim_end_matches("]").trim();

                        let object_type = self.parse_type(object_type_str);
                        let index_type = self.parse_type(index_type_str);
                        return self.handle_index_access_type(object_type, index_type);
                    }
                    // 默认为接口类型
                    Type::Interface(type_str.to_string())
                }
                else if (type_str.starts_with('"') && type_str.ends_with('"')) || (type_str.starts_with('\'') && type_str.ends_with('\'')) {
                    // 字符串字面量类型
                    let literal = type_str[1..type_str.len() - 1].to_string();
                    self.handle_literal_type(literal)
                }
                else if type_str.chars().all(|c| c.is_digit(10)) {
                    // 数字字面量类型
                    self.handle_literal_type(type_str.to_string())
                }
                else if type_str == "true" || type_str == "false" {
                    // 布尔字面量类型
                    self.handle_literal_type(type_str.to_string())
                }
                else if let Some(_) = self.type_env.get_enum(&type_str) {
                    // 枚举类型
                    Type::Enum(type_str.to_string())
                }
                else {
                    // 接口类型或类型别名
                    Type::Interface(type_str.to_string())
                }
            }
        }
    }
}

// TypeChecker 结构体的额外方法实现
impl TypeChecker {
    /// 解析泛型类型
    ///
    /// # 参数
    ///
    /// * `type_str` - 类型字符串
    ///
    /// # 返回
    ///
    /// 解析结果，包含类型名称和类型参数
    fn parse_generic_type(&self, type_str: &str) -> Option<(String, Vec<Type>)> {
        // 查找泛型类型的开始和结束位置
        let mut bracket_count = 0;
        let mut start_index = None;
        let mut end_index = None;

        for (i, c) in type_str.chars().enumerate() {
            if c == '<' {
                if start_index.is_none() {
                    start_index = Some(i);
                }
                bracket_count += 1;
            }
            else if c == '>' {
                bracket_count -= 1;
                if bracket_count == 0 {
                    end_index = Some(i);
                    break;
                }
            }
        }

        if let (Some(start), Some(end)) = (start_index, end_index) {
            let name = type_str[..start].trim().to_string();
            let type_args_str = &type_str[start + 1..end].trim();

            // 解析类型参数
            let type_args = self.parse_type_args(type_args_str);
            Some((name, type_args))
        }
        else {
            None
        }
    }

    /// 解析类型参数列表
    ///
    /// # 参数
    ///
    /// * `type_args_str` - 类型参数字符串
    ///
    /// # 返回
    ///
    /// 解析后的类型参数列表
    fn parse_type_args(&self, type_args_str: &str) -> Vec<Type> {
        let mut type_args = Vec::new();
        let mut current_arg = String::new();
        let mut bracket_count = 0;

        for c in type_args_str.chars() {
            if c == ',' && bracket_count == 0 {
                // 类型参数分隔符
                if !current_arg.trim().is_empty() {
                    type_args.push(self.parse_type(&current_arg));
                    current_arg.clear();
                }
            }
            else {
                // 处理嵌套的泛型类型
                if c == '<' {
                    bracket_count += 1;
                }
                else if c == '>' {
                    bracket_count -= 1;
                }
                current_arg.push(c);
            }
        }

        // 处理最后一个类型参数
        if !current_arg.trim().is_empty() {
            type_args.push(self.parse_type(&current_arg));
        }

        type_args
    }

    /// 处理 Partial 类型: Partial<T>
    fn handle_partial_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            // 创建映射类型，使所有属性变为可选
            let key_var = "K".to_string();
            let source_type = self.handle_keyof_type(target_type.clone());
            let mapped_type = Type::Union(vec![target_type.clone(), Type::Void]);
            self.handle_mapped_type(key_var, source_type, mapped_type)
        }
        else {
            Type::Any
        }
    }

    /// 处理 Required 类型: Required<T>
    fn handle_required_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            // 创建映射类型，使所有属性变为必需
            let key_var = "K".to_string();
            let source_type = self.handle_keyof_type(target_type.clone());
            let mapped_type = target_type.clone();
            self.handle_mapped_type(key_var, source_type, mapped_type)
        }
        else {
            Type::Any
        }
    }

    /// 处理 Pick 类型: Pick<T, K>
    fn handle_pick_type(&self, type_args: Vec<Type>) -> Type {
        if type_args.len() >= 2 {
            let target_type = &type_args[0];
            let keys_type = &type_args[1];
            // 创建映射类型，只包含指定的属性
            let key_var = "K".to_string();
            let source_type = Type::Intersection(vec![self.handle_keyof_type(target_type.clone()), keys_type.clone()]);
            let mapped_type = self.handle_index_access_type(target_type.clone(), Type::TypeVar(key_var.clone()));
            self.handle_mapped_type(key_var, source_type, mapped_type)
        }
        else {
            Type::Any
        }
    }

    /// 处理 Omit 类型: Omit<T, K>
    fn handle_omit_type(&self, type_args: Vec<Type>) -> Type {
        if type_args.len() >= 2 {
            let target_type = &type_args[0];
            let keys_type = &type_args[1];
            // 创建映射类型，排除指定的属性
            let key_var = "K".to_string();
            let source_type = Type::Exclude(Box::new(self.handle_keyof_type(target_type.clone())), Box::new(keys_type.clone()));
            let mapped_type = self.handle_index_access_type(target_type.clone(), Type::TypeVar(key_var.clone()));
            self.handle_mapped_type(key_var, source_type, mapped_type)
        }
        else {
            Type::Any
        }
    }

    /// 处理 Record 类型: Record<K, T>
    fn handle_record_type(&self, type_args: Vec<Type>) -> Type {
        if type_args.len() >= 2 {
            let key_type = &type_args[0];
            let value_type = &type_args[1];
            // 创建映射类型，使用指定的键类型和值类型
            let key_var = "K".to_string();
            let source_type = key_type.clone();
            let mapped_type = value_type.clone();
            self.handle_mapped_type(key_var, source_type, mapped_type)
        }
        else {
            Type::Any
        }
    }

    /// 处理 Exclude 类型: Exclude<T, U>
    fn handle_exclude_type(&self, type_args: Vec<Type>) -> Type {
        if type_args.len() >= 2 {
            let target_type = &type_args[0];
            let exclude_type = &type_args[1];
            // 创建排除类型
            Type::Exclude(Box::new(target_type.clone()), Box::new(exclude_type.clone()))
        }
        else {
            Type::Any
        }
    }

    /// 处理 Extract 类型: Extract<T, U>
    fn handle_extract_type(&self, type_args: Vec<Type>) -> Type {
        if type_args.len() >= 2 {
            let target_type = &type_args[0];
            let extract_type = &type_args[1];
            // 创建提取类型
            Type::Extract(Box::new(target_type.clone()), Box::new(extract_type.clone()))
        }
        else {
            Type::Any
        }
    }

    /// 处理 NonNullable 类型: NonNullable<T>
    fn handle_non_nullable_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            // 创建非空类型，排除 null 和 undefined
            Type::Exclude(Box::new(target_type.clone()), Box::new(Type::Union(vec![Type::Void, Type::Void])))
        }
        else {
            Type::Any
        }
    }

    /// 处理 ThisType 类型: ThisType<T>
    fn handle_this_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(this_type) = type_args.get(0) {
            // 简化实现，返回 Any 类型
            // 完整实现需要处理 this 类型上下文
            Type::Any
        }
        else {
            Type::Any
        }
    }

    /// 处理 OmitThisParameter 类型: OmitThisParameter<T>
    fn handle_omit_this_parameter_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            match target_type {
                Type::Function(params, return_type) => {
                    // 移除第一个参数（假设是 this 参数）
                    let new_params = if !params.is_empty() { params[1..].to_vec() } else { params.clone() };
                    Type::Function(new_params, return_type.clone())
                }
                _ => Type::Any,
            }
        }
        else {
            Type::Any
        }
    }

    /// 处理 ThisParameterType 类型: ThisParameterType<T>
    fn handle_this_parameter_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            match target_type {
                Type::Function(params, _) => {
                    // 返回第一个参数的类型（假设是 this 参数）
                    if !params.is_empty() { params[0].clone() } else { Type::Any }
                }
                _ => Type::Any,
            }
        }
        else {
            Type::Any
        }
    }

    /// 处理 Parameters 类型: Parameters<T>
    fn handle_parameters_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            match target_type {
                Type::Function(params, _) => {
                    // 返回函数参数类型的元组
                    Type::Tuple(params.clone())
                }
                _ => Type::Any,
            }
        }
        else {
            Type::Any
        }
    }

    /// 处理 ReturnType 类型: ReturnType<T>
    fn handle_return_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            match target_type {
                Type::Function(_, return_type) => {
                    // 返回函数返回类型
                    *return_type.clone()
                }
                _ => Type::Any,
            }
        }
        else {
            Type::Any
        }
    }

    /// 处理 InstanceType 类型: InstanceType<T>
    fn handle_instance_type(&self, type_args: Vec<Type>) -> Type {
        if let Some(target_type) = type_args.get(0) {
            // 简化实现，返回 Any 类型
            // 完整实现需要处理构造函数类型
            Type::Any
        }
        else {
            Type::Any
        }
    }

    /// 从函数体推断返回类型
    fn infer_function_return_type(&mut self, body: &Vec<nargo_ir::JsStmt>) -> Type {
        let mut return_types = Vec::new();

        // 遍历函数体中的语句，查找 return 语句
        for stmt in body {
            match stmt {
                nargo_ir::JsStmt::Return(expr, _, _) => {
                    if let Some(expr) = expr {
                        // 检查返回表达式的类型
                        let return_type = self.check_expression(expr);
                        return_types.push(return_type);
                    }
                    else {
                        // 无返回值，返回 Void 类型
                        return_types.push(Type::Void);
                    }
                }
                nargo_ir::JsStmt::Block(stmts, _, _) => {
                    // 递归检查块语句
                    let block_return_type = self.infer_function_return_type(stmts);
                    if block_return_type != Type::Any {
                        return_types.push(block_return_type);
                    }
                }
                nargo_ir::JsStmt::If { consequent, alternate, .. } => {
                    // 检查 if 语句的两个分支
                    let consequent_return_type = self.infer_function_return_type(&vec![*consequent.clone()]);
                    let alternate_return_type = if let Some(alt) = alternate { self.infer_function_return_type(&vec![*alt.clone()]) } else { Type::Void };

                    if consequent_return_type != Type::Any {
                        return_types.push(consequent_return_type);
                    }
                    if alternate_return_type != Type::Any {
                        return_types.push(alternate_return_type);
                    }
                }
                _ => {
                    // 其他语句类型，跳过
                }
            }
        }

        // 确定函数的返回类型
        if return_types.is_empty() {
            // 没有 return 语句，返回 Void 类型
            Type::Void
        }
        else if return_types.len() == 1 {
            // 只有一个 return 语句，返回其类型
            return_types[0].clone()
        }
        else {
            // 多个 return 语句，返回它们的联合类型
            Type::Union(return_types)
        }
    }

    /// 收集文件的依赖项
    fn collect_file_dependencies(&self, file_path: &str) -> Vec<String> {
        // 简化实现，实际需要解析文件中的 import 语句
        // 这里我们返回一个空向量，实际实现需要解析文件内容
        Vec::new()
    }

    /// 更新文件依赖图
    fn update_file_dependencies(&mut self, file_path: &str, dependencies: &[String]) {
        // 更新依赖图
        self.file_dependencies.insert(file_path.to_string(), dependencies.to_vec());

        // 更新依赖此文件的文件列表
        for dep in dependencies {
            if let Some(dependents) = self.file_dependencies.get_mut(dep) {
                if !dependents.contains(&file_path.to_string()) {
                    dependents.push(file_path.to_string());
                }
            }
            else {
                self.file_dependencies.insert(dep.to_string(), vec![file_path.to_string()]);
            }
        }
    }

    /// 获取依赖指定文件的文件列表
    fn get_file_dependents(&self, file_path: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        // 遍历依赖图，找到所有依赖此文件的文件
        for (file, deps) in &self.file_dependencies {
            if deps.contains(&file_path.to_string()) {
                dependents.push(file.clone());
            }
        }

        dependents
    }

    /// 增量类型检查
    ///
    /// # 参数
    ///
    /// * `files` - 需要检查的文件列表
    ///
    /// # 返回
    ///
    /// 类型检查结果
    pub async fn incremental_check(&mut self, files: Vec<PathBuf>) -> NargoResult<TypeCheckResult> {
        let mut errors = 0;
        let mut checked_files = 0;

        // 收集需要检查的文件（包括修改的文件和依赖它们的文件）
        let mut files_to_check = HashSet::new();
        for file in files {
            let file_path_str = file.to_string_lossy().to_string();
            files_to_check.insert(file_path_str.clone());

            // 添加依赖此文件的文件
            let dependents = self.get_file_dependents(&file_path_str);
            for dependent in dependents {
                files_to_check.insert(dependent);
            }
        }

        // 检查这些文件
        for file_path_str in files_to_check {
            let file = PathBuf::from(file_path_str);
            if file.exists() {
                let source = std::fs::read_to_string(&file).map_err(Error::io_error)?;
                match self.check_file(&file, &source) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{} {}: {}", console::style("✘").red(), file.display(), e);
                        errors += 1;
                    }
                }
                checked_files += 1;
            }
        }

        Ok(TypeCheckResult { errors, checked_files })
    }
}

// 实现表达式类型检查器
impl ExpressionChecker for TypeChecker {
    fn check_program(&mut self, program: &nargo_ir::JsProgram) {
        for stmt in &program.body {
            self.check_statement(stmt);
        }
    }

    fn check_statement(&mut self, stmt: &nargo_ir::JsStmt) {
        match stmt {
            nargo_ir::JsStmt::VariableDecl { id, init, .. } => {
                // 暂时只支持从初始化表达式推断类型
                let mut var_type = Type::Any;

                // 检查初始化表达式的类型
                if let Some(init_expr) = init {
                    let init_type = self.check_expression(init_expr);
                    // 如果没有类型注解，使用初始化表达式的类型
                    if var_type == Type::Any {
                        var_type = init_type;
                    }
                    else if !self.is_type_compatible(&init_type, &var_type) {
                        // 类型注解与初始化表达式类型不兼容
                        let (line, column) = (0, 0); // 暂时使用默认位置
                        self.errors.push(TypeError {
                            position: (0, 0), // 暂时使用默认位置
                            message: format!("类型不兼容: 初始化表达式类型 {:?} 与类型注解 {:?} 不匹配", init_type, var_type),
                            file_name: self.current_file.clone(),
                            line: Some(line),
                            column: Some(column),
                            error_code: Some("TS2322".to_string()),
                            related_types: Some(vec![format!("{:?}", var_type), format!("{:?}", init_type)]),
                            suggestion: Some("请确保初始化表达式的类型与类型注解匹配".to_string()),
                        });
                    }
                }

                // 添加变量到类型环境
                self.type_env.add_variable(id.clone(), var_type);
            }
            nargo_ir::JsStmt::FunctionDecl { id, params, body, .. } => {
                // 解析参数类型（暂时只支持 Any 类型）
                let param_types = vec![Type::Any; params.len()];

                // 尝试从函数体推断返回类型
                let return_type = self.infer_function_return_type(body);

                // 创建函数类型
                let func_type = Type::Function(param_types, Box::new(return_type));

                // 添加函数到类型环境
                self.type_env.add_function(id.clone(), func_type);

                // 检查函数体
                for stmt in body {
                    self.check_statement(stmt);
                }
            }
            nargo_ir::JsStmt::Return(expr, _, _) => {
                if let Some(expr) = expr {
                    self.check_expression(expr);
                }
            }
            nargo_ir::JsStmt::If { test, consequent, alternate, .. } => {
                self.check_expression(test);
                self.check_statement(consequent);
                if let Some(alt) = alternate {
                    self.check_statement(alt);
                }
            }
            nargo_ir::JsStmt::While { test, body, .. } => {
                self.check_expression(test);
                self.check_statement(body);
            }
            nargo_ir::JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    self.check_statement(init_stmt);
                }
                if let Some(test_expr) = test {
                    self.check_expression(test_expr);
                }
                if let Some(update_expr) = update {
                    self.check_expression(update_expr);
                }
                self.check_statement(body);
            }
            nargo_ir::JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.check_statement(stmt);
                }
            }
            nargo_ir::JsStmt::Expr(expr, _, _) => {
                self.check_expression(expr);
            }
            nargo_ir::JsStmt::Other(stmt_str, _, _) => {
                // 处理 TypeScript 特有的语句，如接口定义和类型别名
                self.handle_typescript_stmt(stmt_str);
            }
            _ => {
                // 其他语句类型，暂时跳过
            }
        }
    }

    fn get_error_position(&self, expr: &nargo_ir::JsExpr) -> (usize, usize) {
        let span = expr.span();
        (span.start.offset as usize, span.end.offset as usize)
    }

    fn get_error_location(&self, expr: &nargo_ir::JsExpr) -> (Option<usize>, Option<usize>) {
        let span = expr.span();
        (Some(span.start.line as usize), Some(span.start.column as usize))
    }

    fn is_type_compatible(&self, actual: &Type, expected: &Type) -> bool {
        // 基本类型兼容性检查
        if actual == expected {
            return true;
        }

        // Any 类型兼容所有类型
        if *actual == Type::Any || *expected == Type::Any {
            return true;
        }

        // Unknown 类型兼容所有类型
        if *expected == Type::Unknown {
            return true;
        }

        // Never 类型兼容所有类型
        if *actual == Type::Never {
            return true;
        }

        // 联合类型兼容性检查
        if let Type::Union(types) = actual {
            return types.iter().all(|t| self.is_type_compatible(t, expected));
        }

        // 联合类型作为预期类型时的兼容性检查
        if let Type::Union(types) = expected {
            return types.iter().any(|t| self.is_type_compatible(actual, t));
        }

        // 交叉类型兼容性检查
        if let Type::Intersection(types) = expected {
            return types.iter().all(|t| self.is_type_compatible(actual, t));
        }

        // 数组类型兼容性检查
        if let (Type::Array(actual_elem), Type::Array(expected_elem)) = (actual, expected) {
            return self.is_type_compatible(actual_elem, expected_elem);
        }

        // 函数类型兼容性检查
        if let (Type::Function(actual_params, actual_return), Type::Function(expected_params, expected_return)) = (actual, expected) {
            // 函数参数数量兼容性（协变）
            if actual_params.len() > expected_params.len() {
                return false;
            }
            // 函数参数类型兼容性（逆变）
            for (actual_param, expected_param) in actual_params.iter().zip(expected_params.iter()) {
                if !self.is_type_compatible(expected_param, actual_param) {
                    return false;
                }
            }
            // 函数返回类型兼容性（协变）
            return self.is_type_compatible(actual_return, expected_return);
        }

        // 对象类型兼容性检查
        if let (Type::Object(actual_members), Type::Object(expected_members)) = (actual, expected) {
            // 预期类型的所有成员必须在实际类型中存在
            for (name, expected_type) in expected_members {
                if let Some(actual_type) = actual_members.get(name) {
                    if !self.is_type_compatible(actual_type, expected_type) {
                        return false;
                    }
                }
                else {
                    return false;
                }
            }
            return true;
        }

        // 接口类型兼容性检查（简化实现）
        if let (Type::Interface(actual_name), Type::Interface(expected_name)) = (actual, expected) {
            return actual_name == expected_name;
        }

        // 泛型接口类型兼容性检查（简化实现）
        if let (Type::GenericInterface(actual_name, actual_args), Type::GenericInterface(expected_name, expected_args)) = (actual, expected) {
            if actual_name != expected_name || actual_args.len() != expected_args.len() {
                return false;
            }
            for (actual_arg, expected_arg) in actual_args.iter().zip(expected_args.iter()) {
                if !self.is_type_compatible(actual_arg, expected_arg) {
                    return false;
                }
            }
            return true;
        }

        // 类型别名兼容性检查（简化实现）
        if let (Type::TypeAlias(actual_name), Type::TypeAlias(expected_name)) = (actual, expected) {
            return actual_name == expected_name;
        }

        // 泛型类型别名兼容性检查（简化实现）
        if let (Type::GenericTypeAlias(actual_name, actual_args), Type::GenericTypeAlias(expected_name, expected_args)) = (actual, expected) {
            if actual_name != expected_name || actual_args.len() != expected_args.len() {
                return false;
            }
            for (actual_arg, expected_arg) in actual_args.iter().zip(expected_args.iter()) {
                if !self.is_type_compatible(actual_arg, expected_arg) {
                    return false;
                }
            }
            return true;
        }

        // 元组类型兼容性检查
        if let (Type::Tuple(actual_types), Type::Tuple(expected_types)) = (actual, expected) {
            if actual_types.len() != expected_types.len() {
                return false;
            }
            for (actual_type, expected_type) in actual_types.iter().zip(expected_types.iter()) {
                if !self.is_type_compatible(actual_type, expected_type) {
                    return false;
                }
            }
            return true;
        }

        // 字面量类型兼容性检查
        if let (Type::Literal(actual_lit), Type::Literal(expected_lit)) = (actual, expected) {
            return actual_lit == expected_lit;
        }

        // 字面量类型与基本类型的兼容性检查
        if let Type::Literal(_) = actual {
            match expected {
                Type::String => return true,
                Type::Number => return true,
                Type::Boolean => return true,
                _ => return false,
            }
        }

        // 枚举类型兼容性检查
        if let (Type::Enum(actual_name), Type::Enum(expected_name)) = (actual, expected) {
            return actual_name == expected_name;
        }

        // 枚举类型与字符串/数字类型的兼容性
        if let Type::Enum(_) = actual {
            match expected {
                Type::String => return true,
                Type::Number => return true,
                _ => return false,
            }
        }

        // 条件类型兼容性检查（简化实现）
        if let (Type::Conditional(_, _, true_type, false_type), _) = (actual, expected) {
            // 条件类型的兼容性取决于其可能的结果类型
            return self.is_type_compatible(true_type, expected) || self.is_type_compatible(false_type, expected);
        }

        // 映射类型兼容性检查（简化实现）
        if let (Type::Mapped(_, _, mapped_type), _) = (actual, expected) {
            return self.is_type_compatible(mapped_type, expected);
        }

        // 索引访问类型兼容性检查（简化实现）
        if let (Type::IndexAccess(_, _), _) = (actual, expected) {
            // 简化实现，实际需要更复杂的类型计算
            return true;
        }

        // KeyOf 类型兼容性检查（简化实现）
        if let (Type::KeyOf(_), _) = (actual, expected) {
            // 简化实现，实际需要更复杂的类型计算
            return true;
        }

        // 排除类型兼容性检查（简化实现）
        if let (Type::Exclude(_, _), _) = (actual, expected) {
            // 简化实现，实际需要更复杂的类型计算
            return true;
        }

        // 提取类型兼容性检查（简化实现）
        if let (Type::Extract(_, _), _) = (actual, expected) {
            // 简化实现，实际需要更复杂的类型计算
            return true;
        }

        false
    }

    fn check_expression(&mut self, expr: &nargo_ir::JsExpr) -> Type {
        match expr {
            nargo_ir::JsExpr::Identifier(name, ..) => {
                // 查找变量或函数类型
                if let Some(ty) = self.type_env.get_variable(name) {
                    ty.clone()
                }
                else if let Some(ty) = self.type_env.get_function(name) {
                    ty.clone()
                }
                else if let Some(ty) = self.type_env.get_type_alias(name) {
                    ty.clone()
                }
                else if let Some(_) = self.type_env.get_interface(name) {
                    // 接口类型作为值使用
                    let (line, column) = self.get_error_location(expr);
                    self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("接口 '{}' 只能用作类型，不能用作值", name), file_name: self.current_file.clone(), line, column, error_code: Some("TS2693".to_string()), related_types: None, suggestion: Some(format!("请使用 '{}' 作为类型注解，而不是值", name)) });
                    Type::Any
                }
                else {
                    // 检查是否是枚举变体
                    let mut enum_type = None;
                    for (enum_name, variants) in &self.type_env.enums {
                        if variants.contains(name) {
                            enum_type = Some(Type::Enum(enum_name.clone()));
                            break;
                        }
                    }

                    if let Some(ty) = enum_type {
                        ty
                    }
                    else {
                        // 未找到变量，添加类型错误
                        let (line, column) = self.get_error_location(expr);
                        self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("未定义的标识符: '{}'", name), file_name: self.current_file.clone(), line, column, error_code: Some("TS2304".to_string()), related_types: None, suggestion: Some(format!("请确保标识符 '{}' 已声明，或者检查拼写是否正确", name)) });
                        Type::Any
                    }
                }
            }
            nargo_ir::JsExpr::Other(code, ..) => {
                // 处理其他表达式，包括类型守卫
                // 简化实现，实际需要根据表达式的具体内容进行处理
                Type::Any
            }
            nargo_ir::JsExpr::Literal(value, ..) => {
                // 根据字面量值推断类型
                match value {
                    nargo_types::NargoValue::Number(_) => Type::Number,
                    nargo_types::NargoValue::String(_) => Type::String,
                    nargo_types::NargoValue::Bool(_) => Type::Boolean,
                    nargo_types::NargoValue::Null => Type::Void,
                    _ => Type::Any,
                }
            }
            nargo_ir::JsExpr::Binary { left, right, op, .. } => {
                let left_type = self.check_expression(left);
                let right_type = self.check_expression(right);

                // 检查操作符的类型兼容性
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        // 算术运算符要求操作数为数字类型
                        if !self.is_type_compatible(&left_type, &Type::Number) || !self.is_type_compatible(&right_type, &Type::Number) {
                            let (line, column) = self.get_error_location(expr);
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: "算术运算符要求操作数为数字类型".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2365".to_string()), related_types: Some(vec!["number".to_string()]), suggestion: Some("请确保操作数为数字类型".to_string()) });
                        }
                        Type::Number
                    }
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                        // 比较运算符返回布尔类型
                        Type::Boolean
                    }
                    "&&" | "||" => {
                        // 逻辑运算符要求操作数为布尔类型
                        if !self.is_type_compatible(&left_type, &Type::Boolean) || !self.is_type_compatible(&right_type, &Type::Boolean) {
                            let (line, column) = self.get_error_location(expr);
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: "逻辑运算符要求操作数为布尔类型".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2365".to_string()), related_types: Some(vec!["boolean".to_string()]), suggestion: Some("请确保操作数为布尔类型".to_string()) });
                        }
                        Type::Boolean
                    }
                    "in" => {
                        // in 操作符用于检查属性是否存在于对象中
                        // 左侧是属性名（字符串或符号），右侧是对象
                        Type::Boolean
                    }
                    "instanceof" => {
                        // instanceof 操作符用于检查对象是否是指定类型的实例
                        Type::Boolean
                    }
                    _ => Type::Any,
                }
            }
            nargo_ir::JsExpr::Unary { op, argument, .. } => {
                let arg_type = self.check_expression(argument);

                match op.as_str() {
                    "!" => {
                        // 逻辑非运算符要求操作数为布尔类型
                        if !self.is_type_compatible(&arg_type, &Type::Boolean) {
                            let (line, column) = self.get_error_location(expr);
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: "逻辑非运算符要求操作数为布尔类型".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2365".to_string()), related_types: Some(vec!["boolean".to_string()]), suggestion: Some("请确保操作数为布尔类型".to_string()) });
                        }
                        Type::Boolean
                    }
                    "-" => {
                        // 负号运算符要求操作数为数字类型
                        if !self.is_type_compatible(&arg_type, &Type::Number) {
                            let (line, column) = self.get_error_location(expr);
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: "一元负号运算符要求操作数为数字类型".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2365".to_string()), related_types: Some(vec!["number".to_string()]), suggestion: Some("请确保操作数为数字类型".to_string()) });
                        }
                        Type::Number
                    }
                    "typeof" => {
                        // typeof 操作符返回字符串类型
                        Type::String
                    }
                    _ => Type::Any,
                }
            }
            nargo_ir::JsExpr::Call { callee, args, .. } => {
                // 检查调用表达式
                let callee_type = self.check_expression(callee);
                let (line, column) = self.get_error_location(expr);

                match callee_type {
                    Type::Function(param_types, return_type) => {
                        // 检查参数数量
                        if args.len() < param_types.len() {
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("函数调用缺少参数: 期望 {} 个参数，实际提供 {} 个参数", param_types.len(), args.len()), file_name: self.current_file.clone(), line, column, error_code: Some("TS2554".to_string()), related_types: None, suggestion: Some("请提供所有必需的参数".to_string()) });
                        }
                        else if args.len() > param_types.len() {
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("函数调用参数过多: 期望 {} 个参数，实际提供 {} 个参数", param_types.len(), args.len()), file_name: self.current_file.clone(), line, column, error_code: Some("TS2554".to_string()), related_types: None, suggestion: Some("请只提供函数声明中定义的参数".to_string()) });
                        }

                        // 检查参数类型
                        for (i, (arg, expected_type)) in args.iter().zip(param_types.iter()).enumerate() {
                            let arg_type = self.check_expression(arg);
                            if !self.is_type_compatible(&arg_type, expected_type) {
                                self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("第 {} 个参数类型不匹配: 期望类型为 {:?}，实际提供的类型为 {:?}", i + 1, expected_type, arg_type), file_name: self.current_file.clone(), line, column, error_code: Some("TS2345".to_string()), related_types: Some(vec![format!("{:?}", expected_type), format!("{:?}", arg_type)]), suggestion: Some("请确保参数类型与函数声明中定义的类型匹配".to_string()) });
                            }
                        }

                        *return_type
                    }
                    _ => {
                        // 非函数类型被调用
                        self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("类型 '{:?}' 不是函数，不能被调用", callee_type), file_name: self.current_file.clone(), line, column, error_code: Some("TS2349".to_string()), related_types: None, suggestion: Some("请确保调用的是函数类型的表达式".to_string()) });
                        Type::Any
                    }
                }
            }
            nargo_ir::JsExpr::Member { object, property, computed, .. } => {
                // 检查成员表达式
                let obj_type = self.check_expression(object);

                match obj_type {
                    Type::Object(members) => {
                        if *computed {
                            // 计算属性访问，暂时返回 Any 类型
                            Type::Any
                        }
                        else {
                            // 静态属性访问
                            if let nargo_ir::JsExpr::Identifier(prop_name, _, _) = &**property {
                                if let Some(prop_type) = members.get(prop_name) {
                                    prop_type.clone()
                                }
                                else {
                                    // 属性不存在
                                    let (line, column) = self.get_error_location(expr);
                                    self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("对象不存在属性: {}", prop_name), file_name: self.current_file.clone(), line, column, error_code: Some("TS2339".to_string()), related_types: None, suggestion: Some(format!("请确保对象具有属性 {}", prop_name)) });
                                    Type::Any
                                }
                            }
                            else {
                                Type::Any
                            }
                        }
                    }
                    Type::Array(_) => {
                        // 数组访问，返回数组元素类型
                        if let Type::Array(element_type) = obj_type { *element_type } else { Type::Any }
                    }
                    Type::Interface(interface_name) => {
                        // 接口类型的成员访问
                        if let Some(interface) = self.type_env.get_interface(&interface_name) {
                            if !*computed {
                                if let nargo_ir::JsExpr::Identifier(prop_name, _, _) = &**property {
                                    if let Some(prop_type) = interface.members.get(prop_name) {
                                        prop_type.clone()
                                    }
                                    else {
                                        // 属性不存在
                                        let (line, column) = self.get_error_location(expr);
                                        self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("接口 {} 不存在属性: {}", interface_name, prop_name), file_name: self.current_file.clone(), line, column, error_code: Some("TS2339".to_string()), related_types: None, suggestion: Some(format!("请确保接口 {} 具有属性 {}", interface_name, prop_name)) });
                                        Type::Any
                                    }
                                }
                                else {
                                    Type::Any
                                }
                            }
                            else {
                                Type::Any
                            }
                        }
                        else {
                            // 接口不存在
                            let (line, column) = self.get_error_location(expr);
                            self.errors.push(TypeError { position: self.get_error_position(expr), message: format!("未定义的接口: {}", interface_name), file_name: self.current_file.clone(), line, column, error_code: Some("TS2304".to_string()), related_types: None, suggestion: Some(format!("请确保接口 {} 已定义", interface_name)) });
                            Type::Any
                        }
                    }
                    _ => {
                        // 非对象或数组类型的成员访问
                        let (line, column) = self.get_error_location(expr);
                        self.errors.push(TypeError { position: self.get_error_position(expr), message: "非对象或数组类型不能访问成员".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2339".to_string()), related_types: None, suggestion: Some("请确保访问成员的是对象或数组类型".to_string()) });
                        Type::Any
                    }
                }
            }
            nargo_ir::JsExpr::Array(elements, _, _) => {
                // 推断数组类型
                if elements.is_empty() {
                    Type::Array(Box::new(Type::Any))
                }
                else {
                    // 收集所有元素的类型
                    let mut element_types = Vec::new();
                    for element in elements {
                        let element_type = self.check_expression(element);
                        element_types.push(element_type);
                    }

                    // 确定数组元素类型
                    let element_type = if element_types.len() == 1 {
                        element_types[0].clone()
                    }
                    else {
                        // 检查所有元素类型是否相同
                        let first_type = &element_types[0];
                        let all_same = element_types.iter().all(|t| t == first_type);
                        if all_same {
                            first_type.clone()
                        }
                        else {
                            // 返回联合类型
                            Type::Union(element_types)
                        }
                    };

                    Type::Array(Box::new(element_type))
                }
            }
            nargo_ir::JsExpr::Object(properties, _, _) => {
                // 推断对象类型
                let mut obj_type = HashMap::new();
                for (key, value) in properties {
                    let value_type = self.check_expression(value);
                    obj_type.insert(key.clone(), value_type);
                }
                Type::Object(obj_type)
            }
            nargo_ir::JsExpr::ArrowFunction { params, body, .. } => {
                // 解析参数类型
                let mut param_types = Vec::new();
                for param in params {
                    // 暂时假设所有参数都是 Any 类型
                    // 后续可以从类型注解中获取类型信息
                    param_types.push(Type::Any);
                }

                // 解析返回类型
                let return_type = self.check_expression(body);

                Type::Function(param_types, Box::new(return_type))
            }
            nargo_ir::JsExpr::Conditional { test, consequent, alternate, .. } => {
                // 检查条件表达式
                let test_type = self.check_expression(test);
                let (line, column) = self.get_error_location(expr);
                if !self.is_type_compatible(&test_type, &Type::Boolean) {
                    self.errors.push(TypeError { position: self.get_error_position(expr), message: "条件表达式要求测试表达式为布尔类型".to_string(), file_name: self.current_file.clone(), line, column, error_code: Some("TS2365".to_string()), related_types: Some(vec!["boolean".to_string()]), suggestion: Some("请确保测试表达式为布尔类型".to_string()) });
                }

                // 检查结果表达式
                let consequent_type = self.check_expression(consequent);
                let alternate_type = self.check_expression(alternate);

                // 返回两个分支的联合类型
                if consequent_type == alternate_type { consequent_type } else { Type::Union(vec![consequent_type, alternate_type]) }
            }
            nargo_ir::JsExpr::TemplateLiteral { expressions, .. } => {
                // 模板字面量返回字符串类型
                for expr in expressions {
                    self.check_expression(expr);
                }
                Type::String
            }
            nargo_ir::JsExpr::TseElement { tag, attributes, children, .. } => {
                // 检查 TSE 元素（HXO 特有）
                // 暂时返回 Any 类型
                for attr in attributes {
                    if let Some(value) = &attr.value {
                        self.check_expression(value);
                    }
                }
                for child in children {
                    self.check_expression(child);
                }
                Type::Any
            }
            nargo_ir::JsExpr::Spread(expr, ..) => {
                // 检查展开表达式
                let expr_type = self.check_expression(expr);
                expr_type
            }
            _ => {
                // 其他表达式类型，暂时返回 Any 类型
                Type::Any
            }
        }
    }

    fn check_template(&mut self, template: &nargo_ir::TemplateIR) {
        self.check_template_impl(template);
    }

    fn check_template_node(&mut self, node: &nargo_ir::TemplateNodeIR) {
        self.check_template_node_impl(node);
    }

    fn check_element(&mut self, element: &nargo_ir::ElementIR) {
        self.check_element_impl(element);
    }

    fn check_attribute(&mut self, attr: &nargo_ir::AttributeIR) {
        self.check_attribute_impl(attr);
    }

    fn check_if_node(&mut self, if_node: &nargo_ir::IfNodeIR) {
        self.check_if_node_impl(if_node);
    }

    fn check_for_node(&mut self, for_node: &nargo_ir::ForNodeIR) {
        self.check_for_node_impl(for_node);
    }

    fn check_interpolation(&mut self, interpolation: &nargo_ir::ExpressionIR) {
        self.check_interpolation_impl(interpolation);
    }
}

// 实现模板类型检查器
impl TemplateChecker for TypeChecker {
    fn check_template(&mut self, template: &nargo_ir::TemplateIR) {
        self.check_template_impl(template);
    }

    fn check_template_node(&mut self, node: &nargo_ir::TemplateNodeIR) {
        self.check_template_node_impl(node);
    }

    fn check_element(&mut self, element: &nargo_ir::ElementIR) {
        self.check_element_impl(element);
    }

    fn check_attribute(&mut self, attr: &nargo_ir::AttributeIR) {
        self.check_attribute_impl(attr);
    }

    fn check_if_node(&mut self, if_node: &nargo_ir::IfNodeIR) {
        self.check_if_node_impl(if_node);
    }

    fn check_for_node(&mut self, for_node: &nargo_ir::ForNodeIR) {
        self.check_for_node_impl(for_node);
    }

    fn check_interpolation(&mut self, interpolation: &nargo_ir::ExpressionIR) {
        self.check_interpolation_impl(interpolation);
    }
}

// 实现工具函数
impl Utils for TypeChecker {
    fn find_frontend_files(&self, dir: &PathBuf) -> NargoResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        if dir.is_file() {
            files.push(dir.clone());
        }
        else if dir.is_dir() {
            // 检查是否有 tsconfig.json 配置
            if let Some(config) = &self.config {
                // 使用配置中的 include 和 exclude
                if let Some(include) = &config.include {
                    for pattern in include {
                        let expanded_patterns = self.expand_glob_pattern(dir, pattern);
                        for pattern in expanded_patterns {
                            if let Ok(matches) = glob::glob(&pattern) {
                                for entry in matches {
                                    if let Ok(path) = entry {
                                        if path.is_file() {
                                            files.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                else {
                    // 默认包含所有前端文件
                    self.find_files_recursive(dir, &mut files);
                }
            }
            else {
                // 没有配置文件，默认包含所有前端文件
                self.find_files_recursive(dir, &mut files);
            }
        }

        Ok(files)
    }

    fn find_files_recursive(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        // 检查是否需要排除
                        if !self.should_exclude(&path) {
                            self.find_files_recursive(&path, files);
                        }
                    }
                    else if let Some(ext) = path.extension() {
                        if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    fn should_exclude(&self, path: &PathBuf) -> bool {
        if let Some(config) = &self.config {
            if let Some(exclude) = &config.exclude {
                let path_str = path.to_string_lossy();
                for pattern in exclude {
                    if self.matches_pattern(&path_str, pattern) {
                        return true;
                    }
                }
            }
        }
        // 默认排除 node_modules 和 .git 目录
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name == "node_modules" || file_name == ".git"
    }

    fn expand_glob_pattern(&self, base_dir: &PathBuf, pattern: &str) -> Vec<String> {
        let base_path = base_dir.to_string_lossy();
        let pattern = pattern.replace("**/*", "*");
        vec![format!("{}/{}", base_path, pattern)]
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // 简单的模式匹配实现
        let pattern = pattern.replace("**", ".*");
        let pattern = pattern.replace("*", ".*");
        if let Ok(re) = Regex::new(&format!("^{}$", pattern)) { re.is_match(path) } else { false }
    }
}

// 模板类型检查的具体实现
impl TypeChecker {
    /// 类型检查模板
    fn check_template_impl(&mut self, template: &nargo_ir::TemplateIR) {
        for node in &template.nodes {
            self.check_template_node_impl(node);
        }
    }

    /// 类型检查模板节点
    fn check_template_node_impl(&mut self, node: &nargo_ir::TemplateNodeIR) {
        match node {
            nargo_ir::TemplateNodeIR::Element(element) => {
                self.check_element_impl(element);
            }
            nargo_ir::TemplateNodeIR::If(if_node) => {
                self.check_if_node_impl(if_node);
            }
            nargo_ir::TemplateNodeIR::For(for_node) => {
                self.check_for_node_impl(for_node);
            }
            nargo_ir::TemplateNodeIR::Interpolation(interpolation) => {
                self.check_interpolation_impl(interpolation);
            }
            _ => {
                // 其他模板节点类型，暂时跳过
            }
        }
    }

    /// 类型检查元素
    fn check_element_impl(&mut self, element: &nargo_ir::ElementIR) {
        for attr in &element.attributes {
            self.check_attribute_impl(attr);
        }
        for child in &element.children {
            self.check_template_node_impl(child);
        }
    }

    /// 类型检查属性
    fn check_attribute_impl(&mut self, attr: &nargo_ir::AttributeIR) {
        if let Some(value_ast) = &attr.value_ast {
            self.check_expression(value_ast);
        }
    }

    /// 类型检查 if 节点
    fn check_if_node_impl(&mut self, if_node: &nargo_ir::IfNodeIR) {
        if let Some(ast) = &if_node.condition.ast {
            self.check_expression(ast);
        }
        for child in &if_node.consequent {
            self.check_template_node_impl(child);
        }
        if let Some(alternate) = &if_node.alternate {
            for child in alternate {
                self.check_template_node_impl(child);
            }
        }
        for (condition, body) in &if_node.else_ifs {
            if let Some(ast) = &condition.ast {
                self.check_expression(ast);
            }
            for child in body {
                self.check_template_node_impl(child);
            }
        }
    }

    /// 类型检查 for 节点
    fn check_for_node_impl(&mut self, for_node: &nargo_ir::ForNodeIR) {
        if let Some(ast) = &for_node.iterator.collection.ast {
            self.check_expression(ast);
        }
        for child in &for_node.body {
            self.check_template_node_impl(child);
        }
    }

    /// 类型检查插值
    fn check_interpolation_impl(&mut self, interpolation: &nargo_ir::ExpressionIR) {
        if let Some(ast) = &interpolation.ast {
            self.check_expression(ast);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        assert_eq!(checker.errors.len(), 0);
    }

    #[test]
    fn test_parse_type() {
        let checker = TypeChecker::new();
        let ty = checker.parse_type("string");
        assert_eq!(ty, Type::String);
    }

    #[test]
    fn test_parse_complex_type() {
        let checker = TypeChecker::new();
        let ty = checker.parse_type("{ name: string; age: number }");
        match ty {
            Type::Object(members) => {
                assert_eq!(members.len(), 2);
                assert_eq!(members.get("name"), Some(&Type::String));
                assert_eq!(members.get("age"), Some(&Type::Number));
            }
            _ => panic!("Expected Object type, got {:?}", ty),
        }
    }

    #[test]
    fn test_is_type_compatible() {
        let checker = TypeChecker::new();
        assert!(checker.is_type_compatible(&Type::String, &Type::String));
        assert!(checker.is_type_compatible(&Type::String, &Type::Any));
        assert!(!checker.is_type_compatible(&Type::String, &Type::Number));
    }

    #[test]
    fn test_handle_typescript_stmt() {
        let mut checker = TypeChecker::new();
        checker.handle_typescript_stmt("interface User { name: string; age: number; }");
        assert!(checker.type_env.get_interface("User").is_some());
    }

    #[test]
    fn test_handle_enum_stmt() {
        let mut checker = TypeChecker::new();
        checker.handle_typescript_stmt("enum Direction { Up, Down, Left, Right }");
        assert!(checker.type_env.get_enum("Direction").is_some());
    }

    #[test]
    fn test_handle_type_alias_stmt() {
        let mut checker = TypeChecker::new();
        checker.handle_typescript_stmt("type User = { name: string; age: number; }");
        assert!(checker.type_env.get_type_alias("User").is_some());
    }

    #[test]
    fn test_check_file() {
        let mut checker = TypeChecker::new();
        let file = PathBuf::from("test.tsx");
        let source = r#"
<template>
  <div>{{ message }}</div>
</template>
<script lang="ts">
const message: string = 'Hello';
</script>
"#;
        let result = checker.check_file(&file, source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_file_with_error() {
        let mut checker = TypeChecker::new();
        let file = PathBuf::from("test.tsx");
        let source = r#"
<template>
  <div>{{ message }}</div>
</template>
<script lang="ts">
const message: number = 'Hello'; // 类型错误
</script>
"#;
        let result = checker.check_file(&file, source);
        assert!(result.is_err());
    }

    #[test]
    fn test_collect_file_dependencies() {
        let checker = TypeChecker::new();
        let dependencies = checker.collect_file_dependencies("test.tsx");
        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_get_file_dependents() {
        let mut checker = TypeChecker::new();
        checker.update_file_dependencies("test.tsx", &["dep1.tsx".to_string(), "dep2.tsx".to_string()]);
        let dependents = checker.get_file_dependents("dep1.tsx");
        assert!(dependents.contains(&"test.tsx".to_string()));
    }
}
