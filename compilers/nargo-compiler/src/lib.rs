#![warn(missing_docs)]

use nargo_bundler::{
    Bundler,
    types::{BuildOutputs, OutputFormat},
};
use nargo_ir::IRModule;
use nargo_optimizer::Optimizer;
use nargo_parser::{Parser, ParserRegistry};
use nargo_transformer::Transformer;
pub use nargo_types as types;
pub use nargo_types::{CompileMode, CompileOptions, Result};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::{Instant, SystemTime},
};

/// 线程池配置
pub struct ThreadPoolConfig {
    /// 线程数
    pub thread_count: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self { thread_count: num_cpus::get() }
    }
}

impl Clone for ThreadPoolConfig {
    fn clone(&self) -> Self {
        Self { thread_count: self.thread_count }
    }
}

/// 编译缓存键
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct CompileCacheKey {
    /// 文件名
    pub name: String,
    /// 源代码哈希
    pub source_hash: u64,
    /// 编译选项哈希
    pub options_hash: u64,
}

/// 编译缓存值
#[derive(Clone)]
pub struct CompileCacheValue {
    /// 编译结果
    pub result: CompileResult,
    /// 编译时间
    pub timestamp: Instant,
    /// 使用次数
    pub usage_count: u64,
    /// 最后使用时间
    pub last_used: Instant,
}

/// Nargo 框架的便捷编译函数
///
/// 这个函数封装了 Compiler 的实例化和默认配置，适合简单的单文件编译场景。
pub fn compile(name: &str, source: &str) -> Result<CompileResult> {
    // 获取或创建编译器缓存
    let compiler_cache = COMPILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // 从缓存中获取编译器实例，如果没有则创建新的
    let mut cache = compiler_cache.lock().unwrap();
    let compiler = cache.entry(name.to_string()).or_insert_with(|| Compiler::new());

    // 使用编译器实例
    compiler.compile(name, source)
}

/// 编译 Vmz 格式的文件
///
/// # Arguments
///
/// * `name` - 文件名
/// * `source` - Vmz 源文件内容
///
/// # Returns
///
/// 编译结果，包含生成的代码、CSS、HTML 和 WASM
pub fn compile_vmz(name: &str, source: &str) -> Result<CompileResult> {
    // 获取或创建编译器缓存
    let compiler_cache = COMPILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // 从缓存中获取编译器实例，如果没有则创建新的
    let mut cache = compiler_cache.lock().unwrap();
    let compiler = cache.entry(name.to_string()).or_insert_with(|| Compiler::new());

    // 使用编译器实例
    compiler.compile(name, source)
}

/// 使用自定义选项编译 Nargo 源代码
pub fn compile_with_options(name: &str, source: &str, options: CompileOptions) -> Result<CompileResult> {
    // 获取或创建编译器缓存
    let compiler_cache = COMPILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // 从缓存中获取编译器实例，如果没有则创建新的
    let mut cache = compiler_cache.lock().unwrap();
    let compiler = cache.entry(name.to_string()).or_insert_with(|| Compiler::new());

    // 使用编译器实例
    compiler.compile_with_options(name, source, options)
}

/// 使用自定义选项和格式编译 Nargo 源代码
pub fn compile_with_format(name: &str, source: &str, format: OutputFormat, options: CompileOptions) -> Result<BuildOutputs> {
    // 获取或创建编译器缓存
    let compiler_cache = COMPILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // 从缓存中获取编译器实例，如果没有则创建新的
    let mut cache = compiler_cache.lock().unwrap();
    let compiler = cache.entry(name.to_string()).or_insert_with(|| Compiler::new());

    // 使用编译器实例
    compiler.compile_with_format(name, source, format, options)
}

/// 使用自定义选项编译 Nargo 源代码并生成所有支持的格式
pub fn compile_all_formats(name: &str, source: &str, options: CompileOptions) -> Result<HashMap<OutputFormat, BuildOutputs>> {
    // 获取或创建编译器缓存
    let compiler_cache = COMPILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // 从缓存中获取编译器实例，如果没有则创建新的
    let mut cache = compiler_cache.lock().unwrap();
    let compiler = cache.entry(name.to_string()).or_insert_with(|| Compiler::new());

    // 使用编译器实例
    compiler.compile_all_formats(name, source, options)
}

/// Nargo 核心 API 的统一导出
pub mod prelude {
    pub use crate::{CompileMode, CompileOptions, CompileResult, Compiler, compile, compile_all_formats, compile_vmz, compile_with_format, compile_with_options};
    pub use nargo_bundler::types::OutputFormat;
    pub use nargo_types::{Error, ErrorKind, Position, Result, Span};
}

use serde::Serialize;

pub mod adapter;
pub mod codegen;

/// 编译结果
#[derive(Serialize, Clone)]
pub struct CompileResult {
    /// 生成的 JavaScript 代码
    pub code: String,
    /// 生成的 CSS 代码
    pub css: String,
    /// 生成的 HTML 代码
    pub html: String,
    /// 生成的 WASM 代码（WAT 格式，用于 playground）
    pub wasm: String,
    /// 编译时间（毫秒）
    pub compile_time_ms: u64,
}

/// 编译阶段
#[derive(Debug, Clone, Serialize, Eq, Hash, PartialEq)]
pub enum CompileStage {
    /// 解析阶段
    Parse,
    /// 转换阶段
    Transform,
    /// 优化阶段
    Optimize,
    /// 代码生成阶段
    CodeGen,
}

/// 编译统计信息
#[derive(Debug, Clone, Serialize)]
pub struct CompileStats {
    /// 各阶段耗时（毫秒）
    pub stage_times: std::collections::HashMap<CompileStage, u64>,
    /// 总编译时间（毫秒）
    pub total_time: u64,
    /// 源代码大小（字节）
    pub source_size: usize,
    /// 生成代码大小（字节）
    pub output_size: usize,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 并行编译线程数
    pub parallel_threads: usize,
    /// 模块依赖数量
    pub dependency_count: usize,
    /// 内存使用（MB）
    pub memory_used: f64,
}

// 静态缓存ParserRegistry实例，避免每次创建编译器时重复注册解析器
static REGISTRY_CACHE: OnceLock<Arc<ParserRegistry>> = OnceLock::new();

// 静态缓存编译器实例，避免重复创建
type CompilerCache = Mutex<HashMap<String, Compiler>>;
static COMPILER_CACHE: OnceLock<CompilerCache> = OnceLock::new();

/// 模块依赖图
pub struct DependencyGraph {
    /// 依赖关系映射
    pub dependencies: HashMap<String, Vec<String>>,
    /// 被依赖关系映射
    pub dependents: HashMap<String, Vec<String>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self { dependencies: HashMap::new(), dependents: HashMap::new() }
    }
}

/// 文件修改时间跟踪
pub struct FileModificationTracker {
    /// 文件路径到修改时间的映射
    pub modification_times: HashMap<String, SystemTime>,
}

impl Default for FileModificationTracker {
    fn default() -> Self {
        Self { modification_times: HashMap::new() }
    }
}

/// Nargo 编译器
///
/// 负责协调各个编译阶段，集成 nargo-parser、nargo-transformer 等组件，并提供编译配置和优化选项。
pub struct Compiler {
    /// 解析器注册表
    pub registry: Arc<ParserRegistry>,
    /// 上次生成的 CSS
    pub last_css: String,
    /// 转换器
    pub transformer: Transformer,
    /// 编译统计信息
    pub stats: Option<CompileStats>,
    /// 线程池配置
    pub thread_pool_config: ThreadPoolConfig,
    /// 编译缓存
    pub compile_cache: HashMap<CompileCacheKey, CompileCacheValue>,
    /// 缓存大小限制（默认 1000）
    pub cache_size_limit: usize,
    /// 热编译路径快速缓存
    pub hot_cache: Option<(CompileCacheKey, CompileResult)>,
    /// 打包器，用于生成不同格式的代码
    pub bundler: Bundler,
    /// 模块依赖图
    pub dependency_graph: DependencyGraph,
    /// 文件修改时间跟踪
    pub file_tracker: FileModificationTracker,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Compiler {
    fn clone(&self) -> Self {
        Self { registry: self.registry.clone(), last_css: self.last_css.clone(), transformer: self.transformer.clone(), stats: self.stats.clone(), thread_pool_config: self.thread_pool_config.clone(), compile_cache: self.compile_cache.clone(), cache_size_limit: self.cache_size_limit, hot_cache: self.hot_cache.clone(), bundler: Bundler::new(None), dependency_graph: DependencyGraph { dependencies: self.dependency_graph.dependencies.clone(), dependents: self.dependency_graph.dependents.clone() }, file_tracker: FileModificationTracker { modification_times: self.file_tracker.modification_times.clone() }, cache_hits: self.cache_hits, cache_misses: self.cache_misses }
    }
}

impl Compiler {
    /// 创建新的编译器实例
    pub fn new() -> Self {
        // 获取或创建缓存的ParserRegistry实例
        let registry = REGISTRY_CACHE.get_or_init(|| {
            let registry = ParserRegistry::new();

            // Register default parsers
            // 简化实现，实际需要根据 nargo_parser 的 API 进行调整
            // 由于 ParserRegistry 期望特定类型的解析器，我们暂时不注册任何解析器
            // 实际使用时需要根据 nargo_parser 的 API 提供正确的解析器实现

            Arc::new(registry)
        });

        Self { registry: registry.clone(), last_css: String::new(), transformer: Transformer::new(), stats: None, thread_pool_config: ThreadPoolConfig::default(), compile_cache: HashMap::with_capacity(100), cache_size_limit: 1000, hot_cache: None, bundler: Bundler::new(None), dependency_graph: DependencyGraph::default(), file_tracker: FileModificationTracker::default(), cache_hits: 0, cache_misses: 0 }
    }

    /// 设置缓存大小限制
    ///
    /// # Arguments
    ///
    /// * `limit` - 缓存大小限制
    ///
    /// # Returns
    ///
    /// 返回编译器实例，便于链式调用
    pub fn with_cache_size_limit(mut self, limit: usize) -> Self {
        self.cache_size_limit = limit;
        self
    }

    /// 设置线程池配置
    ///
    /// # Arguments
    ///
    /// * `config` - 线程池配置
    ///
    /// # Returns
    ///
    /// 返回编译器实例，便于链式调用
    pub fn with_thread_pool_config(mut self, config: ThreadPoolConfig) -> Self {
        self.thread_pool_config = config;
        self
    }

    /// 编译 Nargo 源代码
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    /// * `source` - Nargo 源代码
    ///
    /// # Returns
    ///
    /// 编译结果
    pub fn compile(&mut self, name: &str, source: &str) -> Result<CompileResult> {
        self.compile_with_options(name, source, CompileOptions::default())
    }

    /// 使用自定义选项编译 Nargo 源代码
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    /// * `source` - Nargo 源代码
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// 编译结果
    pub fn compile_with_options(&mut self, name: &str, source: &str, mut options: CompileOptions) -> Result<CompileResult> {
        // 快速路径：使用更快的哈希计算
        let source_hash = fast_hash(source);
        let options_hash = fast_hash_options(&options);
        let cache_key = CompileCacheKey { name: name.to_string(), source_hash, options_hash };

        // 热缓存快速检查（用于增量构建）
        if let Some((ref hot_key, ref hot_result)) = self.hot_cache {
            if hot_key == &cache_key {
                // 热缓存命中，极快返回
                self.cache_hits += 1;
                self.stats = Some(CompileStats { stage_times: HashMap::new(), total_time: 0, source_size: source.len(), output_size: hot_result.code.len() + hot_result.css.len(), cache_hits: self.cache_hits, cache_misses: self.cache_misses, parallel_threads: self.thread_pool_config.thread_count, dependency_count: self.dependency_graph.dependencies.len(), memory_used: self.get_memory_usage() });
                return Ok(hot_result.clone());
            }
        }

        // 检查主缓存
        if let Some(cache_value) = self.compile_cache.get_mut(&cache_key) {
            // 缓存命中，更新热缓存和缓存使用统计
            self.cache_hits += 1;
            cache_value.usage_count += 1;
            cache_value.last_used = Instant::now();
            let result = cache_value.result.clone();
            let memory_used = self.get_memory_usage();
            self.hot_cache = Some((cache_key.clone(), result.clone()));
            self.stats = Some(CompileStats { stage_times: HashMap::new(), total_time: 0, source_size: source.len(), output_size: result.code.len() + result.css.len(), cache_hits: self.cache_hits, cache_misses: self.cache_misses, parallel_threads: self.thread_pool_config.thread_count, dependency_count: self.dependency_graph.dependencies.len(), memory_used });
            return Ok(result);
        }

        // 缓存未命中，执行编译
        self.cache_misses += 1;
        let start_time = Instant::now();

        // 预分配空间，减少内存分配
        let mut stage_times = std::collections::HashMap::with_capacity(4);

        // 1. 解析和转换阶段
        let parse_transform_start = Instant::now();
        let ir = self.compile_to_ir(name, source, &mut options)?;
        stage_times.insert(CompileStage::Parse, parse_transform_start.elapsed().as_millis() as u64);

        // 2. 分析模块依赖关系
        self.analyze_dependencies(&ir, name);

        // 3. 优化阶段（已在 compile_to_ir 中完成）
        stage_times.insert(CompileStage::Transform, 0);

        // 4. 代码生成
        let codegen_start = Instant::now();

        // 使用 bundler 生成 JavaScript 代码
        let js_output = self.bundler.bundle_all(&[ir.clone()], OutputFormat::JavaScript)?;
        let code = if let Some(output) = js_output.outputs.first() { String::from_utf8_lossy(&output.code).to_string() } else { String::new() };

        // 生成 HTML 代码
        let html_output = self.bundler.bundle_all(&[ir.clone()], OutputFormat::HTML)?;
        let html = if let Some(output) = html_output.outputs.first() { String::from_utf8_lossy(&output.code).to_string() } else { String::new() };

        // 生成 WASM 代码（如果支持）
        let wasm_output = self.bundler.bundle_all(&[ir], OutputFormat::WebAssembly)?;
        let wasm = if let Some(output) = wasm_output.outputs.first() { String::from_utf8_lossy(&output.code).to_string() } else { String::new() };

        stage_times.insert(CompileStage::CodeGen, codegen_start.elapsed().as_millis() as u64);

        // 计算总编译时间
        let total_time = start_time.elapsed().as_millis() as u64;

        // 生成编译统计信息
        self.stats = Some(CompileStats { stage_times, total_time, source_size: source.len(), output_size: code.len() + self.last_css.len(), cache_hits: self.cache_hits, cache_misses: self.cache_misses, parallel_threads: self.thread_pool_config.thread_count, dependency_count: self.dependency_graph.dependencies.len(), memory_used: self.get_memory_usage() });

        // 创建编译结果
        let result = CompileResult { code, css: std::mem::take(&mut self.last_css), html, wasm, compile_time_ms: total_time };

        // 将结果存入热缓存和主缓存
        self.hot_cache = Some((cache_key.clone(), result.clone()));
        self.compile_cache.insert(cache_key, CompileCacheValue { result: result.clone(), timestamp: Instant::now(), usage_count: 1, last_used: Instant::now() });

        // 智能缓存清理，当缓存接近限制时就开始清理
        if self.compile_cache.len() > self.cache_size_limit * 3 / 2 {
            self.cleanup_cache();
        }

        Ok(result)
    }

    /// 编译源代码到 IR 模块
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    /// * `source` - Nargo 源代码
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// IR 模块
    pub fn compile_to_ir(&mut self, name: &str, source: &str, options: &mut CompileOptions) -> Result<IRModule> {
        // 1. 解析源代码到 IR
        let mut parser = Parser::new(name.to_string(), source, self.registry.clone());
        let mut ir = parser.parse_all()?;

        // 2. 转换脚本（VOC TypeScript 适配器）
        let ts_adapter = adapter::TsAdapter::new();
        ts_adapter.transform(&mut ir)?;

        // 3. 重新分析脚本以更新转换后的元数据
        let analyzer = nargo_script_analyzer::ScriptAnalyzer::new();
        if let Some(script) = &ir.script {
            if let Ok(meta) = analyzer.analyze(script) {
                ir.script_meta = Some(meta.to_nargo_value());
            }
        }

        // 4. 优化和转换 IR
        let mut optimizer = Optimizer::new();

        // 根据编译模式设置优化级别
        if options.is_prod {
            optimizer.set_optimization_level(nargo_optimizer::OptimizationLevel::Aggressive);
        }
        else {
            optimizer.set_optimization_level(nargo_optimizer::OptimizationLevel::Basic);
        }

        // 处理作用域 ID
        let has_scoped_style = ir.styles.iter().any(|s| s.scoped);
        if has_scoped_style && options.scope_id.is_none() {
            options.scope_id = Some(optimizer.generate_scope_id(name));
        }

        // 通过优化器应用所有转换
        optimizer.optimize(&mut ir, options.i18n_locale.as_deref(), options.is_prod);

        // 应用 scoped CSS 转换（如果需要）
        if let Some(scope_id) = &options.scope_id {
            optimizer.apply_scope_id(&mut ir, scope_id);
        }

        // 处理样式（Tailwind/实用 CSS）
        optimizer.process_styles(&ir)?;

        // 直接获取CSS，避免不必要的克隆
        self.last_css = optimizer.get_css();

        Ok(ir)
    }

    /// 获取上次生成的 CSS
    ///
    /// # Returns
    ///
    /// CSS 代码
    pub fn get_css(&self) -> String {
        self.last_css.clone()
    }

    /// 获取编译统计信息
    ///
    /// # Returns
    ///
    /// 编译统计信息
    pub fn get_stats(&self) -> Option<&CompileStats> {
        self.stats.as_ref()
    }

    /// 重置编译器状态
    pub fn reset(&mut self) {
        self.last_css.clear();
        self.transformer.clear_logs();
        self.stats = None;
        self.thread_pool_config = ThreadPoolConfig::default();
        self.compile_cache.clear();
        self.cache_size_limit = 1000;
        self.hot_cache = None;
        self.bundler = Bundler::new(None);
        self.dependency_graph = DependencyGraph::default();
        self.file_tracker = FileModificationTracker::default();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// 计算字符串的哈希值（保留向后兼容性）
    fn hash_string(&self, s: &str) -> u64 {
        fast_hash(s)
    }

    /// 计算编译选项的哈希值（保留向后兼容性）
    fn hash_options(&self, options: &CompileOptions) -> u64 {
        fast_hash_options(options)
    }

    /// 清理缓存，保持缓存大小在限制范围内
    fn cleanup_cache(&mut self) {
        if self.compile_cache.len() > self.cache_size_limit {
            // 按使用频率和最后使用时间排序，优先保留使用频率高和最近使用的项
            let mut entries: Vec<(CompileCacheKey, CompileCacheValue)> = self.compile_cache.drain().collect();

            // 排序策略：
            // 1. 首先按使用频率降序排序
            // 2. 其次按最后使用时间降序排序
            entries.sort_by(|a, b| if a.1.usage_count != b.1.usage_count { b.1.usage_count.cmp(&a.1.usage_count) } else { b.1.last_used.cmp(&a.1.last_used) });

            let keep_count = self.cache_size_limit;
            let to_keep = entries.into_iter().take(keep_count).collect();
            self.compile_cache = to_keep;
        }
    }

    /// 检查文件是否被修改
    ///
    /// # Arguments
    ///
    /// * `file_path` - 文件路径
    ///
    /// # Returns
    ///
    /// 是否被修改
    pub fn is_file_modified(&mut self, file_path: &str) -> bool {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if let Ok(modification_time) = metadata.modified() {
                if let Some(old_time) = self.file_tracker.modification_times.get(file_path) {
                    if modification_time > *old_time {
                        // 文件被修改，更新时间戳
                        self.file_tracker.modification_times.insert(file_path.to_string(), modification_time);
                        return true;
                    }
                }
                else {
                    // 新文件，记录时间戳
                    self.file_tracker.modification_times.insert(file_path.to_string(), modification_time);
                    return true;
                }
            }
        }
        false
    }

    /// 分析模块依赖关系
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    /// * `module_name` - 模块名称
    pub fn analyze_dependencies(&mut self, ir: &IRModule, module_name: &str) {
        // 提取模块依赖
        let dependencies = self.extract_dependencies(ir);

        // 更新依赖图
        self.dependency_graph.dependencies.insert(module_name.to_string(), dependencies.clone());

        // 更新被依赖关系
        for dep in &dependencies {
            let dependents = self.dependency_graph.dependents.entry(dep.to_string()).or_insert_with(Vec::new);
            if !dependents.contains(&module_name.to_string()) {
                dependents.push(module_name.to_string());
            }
        }
    }

    /// 提取模块依赖
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    ///
    /// # Returns
    ///
    /// 依赖的模块列表
    fn extract_dependencies(&self, ir: &IRModule) -> Vec<String> {
        let mut dependencies = Vec::new();

        // 从脚本中提取 import 语句
        if let Some(script) = &ir.script {
            // 简单的正则表达式匹配 import 语句
            let import_re = regex::Regex::new(r#"import\s+.*?from\s+["']([^"']+)["']"#).unwrap();
            for capture in import_re.captures_iter(&script.code) {
                if let Some(dep) = capture.get(1) {
                    let dep_str = dep.as_str().to_string();
                    // 过滤掉相对路径和内置模块
                    if !dep_str.starts_with('.') && !dep_str.starts_with('/') && !dep_str.starts_with('@') {
                        dependencies.push(dep_str);
                    }
                }
            }
        }

        // 从模板中提取组件依赖
        if let Some(template) = &ir.template {
            // 简单的正则表达式匹配组件标签
            let component_re = regex::Regex::new(r#"<([A-Z][a-zA-Z0-9]+)"#).unwrap();
            for capture in component_re.captures_iter(&template.code) {
                if let Some(component) = capture.get(1) {
                    dependencies.push(component.as_str().to_string());
                }
            }
        }

        // 去重
        let mut unique_deps = std::collections::HashSet::new();
        unique_deps.extend(dependencies);
        unique_deps.into_iter().collect()
    }

    /// 获取需要重新编译的模块列表
    ///
    /// # Arguments
    ///
    /// * `modified_modules` - 修改的模块列表
    ///
    /// # Returns
    ///
    /// 需要重新编译的模块列表
    pub fn get_modules_to_recompile(&self, modified_modules: &[String]) -> Vec<String> {
        let mut to_recompile = HashSet::new();
        let mut queue = modified_modules.to_vec();

        while let Some(module) = queue.pop() {
            if to_recompile.insert(module.clone()) {
                // 添加被依赖的模块
                if let Some(dependents) = self.dependency_graph.dependents.get(&module) {
                    queue.extend(dependents.iter().cloned());
                }
            }
        }

        to_recompile.into_iter().collect()
    }

    /// 并行编译多个 Nargo 源代码文件
    ///
    /// # Arguments
    ///
    /// * `files` - 文件名和源代码的映射
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// 编译结果的映射，键为文件名，值为编译结果
    pub fn compile_parallel(&self, files: &HashMap<String, String>, options: CompileOptions) -> Result<HashMap<String, CompileResult>> {
        let thread_count = self.thread_pool_config.thread_count;
        let files: Vec<(String, String)> = files.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let files_len = files.len();

        // 按文件大小排序，优先处理较大的文件
        let mut sorted_files = files;
        sorted_files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        let chunk_size = (sorted_files.len() + thread_count - 1) / thread_count;

        // 预分配足够的空间，避免运行时扩容
        let mut handles = Vec::with_capacity(thread_count);
        let start_time = Instant::now();

        for chunk in sorted_files.chunks(chunk_size) {
            // 只克隆当前块的数据，避免克隆整个文件列表
            let chunk_clone: Vec<(String, String)> = chunk.to_vec();
            let options_clone = options.clone();
            let compiler_clone = self.clone();

            let handle = thread::spawn(move || {
                // 预分配结果空间
                let mut results = HashMap::with_capacity(chunk_clone.len());
                let chunk_start = Instant::now();

                for (name, source) in chunk_clone {
                    let mut compiler = compiler_clone.clone();
                    match compiler.compile_with_options(&name, &source, options_clone.clone()) {
                        Ok(result) => {
                            results.insert(name, result);
                        }
                        Err(e) => {
                            // 处理错误
                            eprintln!("编译文件 {} 时出错: {:?}", name, e);
                        }
                    }
                }

                let chunk_time = chunk_start.elapsed().as_millis();
                eprintln!("Chunk compiled in {}ms", chunk_time);
                results
            });

            handles.push(handle);
        }

        // 预分配结果空间
        let mut all_results = HashMap::with_capacity(files_len);
        for handle in handles {
            if let Ok(results) = handle.join() {
                all_results.extend(results);
            }
        }

        let total_time = start_time.elapsed().as_millis();
        eprintln!("Parallel compilation completed in {}ms", total_time);

        Ok(all_results)
    }

    /// 并行编译多个模块，考虑依赖关系
    ///
    /// # Arguments
    ///
    /// * `modules` - 模块名称和源代码的映射
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// 编译结果的映射，键为模块名称，值为编译结果
    pub fn compile_parallel_with_dependencies(&mut self, modules: &HashMap<String, String>, options: CompileOptions) -> Result<HashMap<String, CompileResult>> {
        // 1. 分析所有模块的依赖关系
        let mut module_irs = HashMap::new();
        for (name, source) in modules {
            let mut compiler = self.clone();
            let mut module_options = options.clone();
            let ir = compiler.compile_to_ir(name, source, &mut module_options)?;
            self.analyze_dependencies(&ir, name);
            module_irs.insert(name.to_string(), ir);
        }

        // 2. 拓扑排序，确定编译顺序
        let sorted_modules = self.topological_sort(modules.keys().collect());

        // 3. 分组编译，优先处理依赖少的模块
        let thread_count = self.thread_pool_config.thread_count;
        let chunk_size = (sorted_modules.len() + thread_count - 1) / thread_count;

        let mut handles = Vec::with_capacity(thread_count);
        let start_time = Instant::now();

        for chunk in sorted_modules.chunks(chunk_size) {
            let chunk_clone: Vec<String> = chunk.to_vec();
            let modules_clone: HashMap<String, String> = modules.iter().filter(|(name, _)| chunk_clone.contains(name)).map(|(k, v)| (k.clone(), v.clone())).collect();
            let options_clone = options.clone();
            let compiler_clone = self.clone();

            let handle = thread::spawn(move || {
                let mut results = HashMap::new();
                let chunk_start = Instant::now();

                for (name, source) in &modules_clone {
                    let mut compiler = compiler_clone.clone();
                    match compiler.compile_with_options(name, source, options_clone.clone()) {
                        Ok(result) => {
                            results.insert(name.clone(), result);
                        }
                        Err(e) => {
                            eprintln!("编译模块 {} 时出错: {:?}", name, e);
                        }
                    }
                }

                let chunk_time = chunk_start.elapsed().as_millis();
                eprintln!("Dependency-aware chunk compiled in {}ms", chunk_time);
                results
            });

            handles.push(handle);
        }

        // 4. 收集结果
        let mut all_results = HashMap::with_capacity(modules.len());
        for handle in handles {
            if let Ok(results) = handle.join() {
                all_results.extend(results);
            }
        }

        let total_time = start_time.elapsed().as_millis();
        eprintln!("Dependency-aware parallel compilation completed in {}ms", total_time);

        Ok(all_results)
    }

    /// 增量编译模块，只重新编译修改的模块及其依赖
    ///
    /// # Arguments
    ///
    /// * `modules` - 模块名称和源代码的映射
    /// * `options` - 编译选项
    /// * `modified_modules` - 修改的模块列表
    ///
    /// # Returns
    ///
    /// 编译结果的映射，键为模块名称，值为编译结果
    pub fn incremental_compile(&mut self, modules: &HashMap<String, String>, options: CompileOptions, modified_modules: &[String]) -> Result<HashMap<String, CompileResult>> {
        // 1. 确定需要重新编译的模块
        let modules_to_recompile = self.get_modules_to_recompile(modified_modules);
        eprintln!("需要重新编译的模块: {:?}", modules_to_recompile);

        if modules_to_recompile.is_empty() {
            eprintln!("没有模块需要重新编译");
            return Ok(HashMap::new());
        }

        // 2. 过滤出需要重新编译的模块
        let modules_to_compile: HashMap<String, String> = modules.iter().filter(|(name, _)| modules_to_recompile.contains(name)).map(|(k, v)| (k.clone(), v.clone())).collect();

        // 3. 并行编译需要重新编译的模块
        if modules_to_compile.is_empty() {
            return Ok(HashMap::new());
        }

        // 4. 使用依赖感知的并行编译
        self.compile_parallel_with_dependencies(&modules_to_compile, options)
    }

    /// 拓扑排序模块依赖
    ///
    /// # Arguments
    ///
    /// * `modules` - 模块名称列表
    ///
    /// # Returns
    ///
    /// 排序后的模块名称列表
    fn topological_sort(&self, modules: Vec<&String>) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut result = Vec::new();

        for module in modules {
            if !visited.contains(module) {
                self.visit_module(module, &mut visited, &mut temp_visited, &mut result);
            }
        }

        result
    }

    /// 访问模块进行拓扑排序
    ///
    /// # Arguments
    ///
    /// * `module` - 模块名称
    /// * `visited` - 已访问的模块
    /// * `temp_visited` - 临时访问的模块（用于检测循环依赖）
    /// * `result` - 排序结果
    fn visit_module<'a>(&'a self, module: &'a String, visited: &mut HashSet<&'a String>, temp_visited: &mut HashSet<&'a String>, result: &mut Vec<String>) {
        if temp_visited.contains(module) {
            // 检测到循环依赖
            eprintln!("警告: 检测到循环依赖: {}", module);
            return;
        }

        if !visited.contains(module) {
            temp_visited.insert(module);

            // 先访问依赖的模块
            if let Some(dependencies) = self.dependency_graph.dependencies.get(module) {
                for dep in dependencies {
                    self.visit_module(dep, visited, temp_visited, result);
                }
            }

            temp_visited.remove(module);
            visited.insert(module);
            result.push(module.clone());
        }
    }

    /// 获取当前内存使用情况（MB）
    ///
    /// # Returns
    ///
    /// 内存使用量（MB）
    fn get_memory_usage(&self) -> f64 {
        // 更准确的内存使用估计
        let mut total_mem = 0.0;

        // 编译器自身大小
        total_mem += std::mem::size_of_val(self) as f64;

        // 编译缓存大小
        let cache_entry_size = std::mem::size_of::<(CompileCacheKey, CompileCacheValue)>() as f64;
        total_mem += (self.compile_cache.len() as f64) * cache_entry_size;

        // 依赖图大小
        for (_, deps) in &self.dependency_graph.dependencies {
            total_mem += (deps.len() as f64) * (std::mem::size_of::<String>() as f64);
        }
        for (_, dependents) in &self.dependency_graph.dependents {
            total_mem += (dependents.len() as f64) * (std::mem::size_of::<String>() as f64);
        }

        // 文件跟踪器大小
        total_mem += (self.file_tracker.modification_times.len() as f64) * ((std::mem::size_of::<String>() + std::mem::size_of::<SystemTime>()) as f64);

        // 转换为 MB
        total_mem / (1024.0 * 1024.0)
    }

    /// 使用 bundler 生成指定格式的代码
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    /// * `source` - Nargo 源代码
    /// * `format` - 输出格式
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// 打包结果
    pub fn compile_with_format(&mut self, name: &str, source: &str, format: OutputFormat, options: CompileOptions) -> Result<BuildOutputs> {
        // 编译到 IR
        let ir = self.compile_to_ir(name, source, &mut options.clone())?;

        // 使用 bundler 生成指定格式的代码
        self.bundler.bundle_all(&[ir], format)
    }

    /// 使用 bundler 生成所有支持的格式的代码
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    /// * `source` - Nargo 源代码
    /// * `options` - 编译选项
    ///
    /// # Returns
    ///
    /// 所有格式的打包结果
    pub fn compile_all_formats(&mut self, name: &str, source: &str, options: CompileOptions) -> Result<HashMap<OutputFormat, BuildOutputs>> {
        // 编译到 IR
        let ir = self.compile_to_ir(name, source, &mut options.clone())?;

        // 生成所有格式的代码
        let formats = vec![OutputFormat::JavaScript, OutputFormat::CSS, OutputFormat::HTML, OutputFormat::TypeScript, OutputFormat::WebAssembly];
        let mut results = HashMap::with_capacity(formats.len());

        for format in formats {
            let output = self.bundler.bundle_all(&[ir.clone()], format.clone())?;
            results.insert(format, output);
        }

        Ok(results)
    }
}

/// 快速字符串哈希函数
#[inline(always)]
fn fast_hash(s: &str) -> u64 {
    // 使用更高效的哈希算法
    let mut hash = 0xcbf29ce484222325u64;
    let prime = 0x100000001b3u64;

    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(prime);
    }

    hash
}

/// 快速编译选项哈希函数
#[inline(always)]
fn fast_hash_options(options: &CompileOptions) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let prime = 0x100000001b3u64;

    // 哈希 mode
    hash ^= options.mode as u64;
    hash = hash.wrapping_mul(prime);

    // 哈希 is_prod
    hash ^= options.is_prod as u64;
    hash = hash.wrapping_mul(prime);

    // 哈希 scope_id
    if let Some(scope_id) = &options.scope_id {
        for byte in scope_id.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(prime);
        }
    }

    // 哈希 i18n_locale
    if let Some(locale) = &options.i18n_locale {
        for byte in locale.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(prime);
        }
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.get_css(), "");
        assert!(compiler.get_stats().is_none());
    }

    #[test]
    fn test_compiler_reset() {
        let mut compiler = Compiler::new();
        let source = r#"
<template>
  <div class="p-4">Hello</div>
</template>
<style>
.p-4 {
  padding: 1rem;
}
</style>
"#;
        let result = compiler.compile("Test", source).unwrap();
        assert!(!result.css.is_empty());
        assert!(compiler.get_stats().is_some());
        compiler.reset();
        assert_eq!(compiler.get_css(), "");
        assert!(compiler.get_stats().is_none());
    }

    #[test]
    fn test_compile_basic() {
        let mut compiler = Compiler::new();
        let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;
        let result = compiler.compile("Test", source).unwrap();
        assert!(!result.code.is_empty());
        assert!(!result.html.is_empty());
    }

    #[test]
    fn test_compile_with_options() {
        let mut compiler = Compiler::new();
        let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;
        let mut options = CompileOptions::default();
        options.is_prod = true;
        let result = compiler.compile_with_options("Test", source, options).unwrap();
        assert!(!result.code.is_empty());
    }

    #[test]
    fn test_compile_with_format() {
        let mut compiler = Compiler::new();
        let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;
        let options = CompileOptions::default();
        let result = compiler.compile_with_format("Test", source, OutputFormat::JavaScript, options).unwrap();
        assert!(!result.outputs.is_empty());
    }

    #[test]
    fn test_compile_all_formats() {
        let mut compiler = Compiler::new();
        let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;
        let options = CompileOptions::default();
        let result = compiler.compile_all_formats("Test", source, options).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains_key(&OutputFormat::JavaScript));
        assert!(result.contains_key(&OutputFormat::HTML));
    }

    #[test]
    fn test_incremental_compile() {
        let mut compiler = Compiler::new();
        let mut modules = HashMap::new();
        modules.insert(
            "Module1".to_string(),
            r#"
<template>
  <div>Module 1</div>
</template>
"#
            .to_string(),
        );
        modules.insert(
            "Module2".to_string(),
            r#"
<template>
  <div>Module 2</div>
</template>
"#
            .to_string(),
        );
        let options = CompileOptions::default();
        let modified_modules = vec!["Module1".to_string()];
        let result = compiler.incremental_compile(&modules, options, &modified_modules).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_is_file_modified() {
        let mut compiler = Compiler::new();
        // 测试不存在的文件
        assert!(!compiler.is_file_modified("non_existent_file.txt"));
    }

    #[test]
    fn test_hash_functions() {
        let s = "test string";
        let hash1 = fast_hash(s);
        let hash2 = fast_hash(s);
        assert_eq!(hash1, hash2);

        let options = CompileOptions::default();
        let options_hash1 = fast_hash_options(&options);
        let options_hash2 = fast_hash_options(&options);
        assert_eq!(options_hash1, options_hash2);
    }

    #[test]
    fn test_extract_dependencies() {
        let compiler = Compiler::new();
        let ir = IRModule::new("test".to_string());
        let dependencies = compiler.extract_dependencies(&ir);
        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_get_modules_to_recompile() {
        let mut compiler = Compiler::new();
        // 添加依赖关系: Module1 -> Module2 -> Module3
        compiler.dependency_graph.dependents.insert("Module1".to_string(), vec!["Module2".to_string()]);
        compiler.dependency_graph.dependents.insert("Module2".to_string(), vec!["Module3".to_string()]);

        let modified_modules = vec!["Module1".to_string()];
        let to_recompile = compiler.get_modules_to_recompile(&modified_modules);
        assert!(to_recompile.contains(&"Module1".to_string()));
        assert!(to_recompile.contains(&"Module2".to_string()));
        assert!(to_recompile.contains(&"Module3".to_string()));
    }

    #[test]
    fn test_compile_parallel() {
        let compiler = Compiler::new();
        let mut files = HashMap::new();
        files.insert("test1.nargo".to_string(), "<div>Hello</div>".to_string());
        files.insert("test2.nargo".to_string(), "<div>World</div>".to_string());

        let options = CompileOptions::default();
        let result = compiler.compile_parallel(&files, options).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("test1.nargo"));
        assert!(result.contains_key("test2.nargo"));
    }

    #[test]
    fn test_compile_parallel_with_dependencies() {
        let mut compiler = Compiler::new();
        let mut modules = HashMap::new();
        modules.insert("Module1".to_string(), "<div>Module1</div>".to_string());
        modules.insert("Module2".to_string(), "<div>Module2</div>".to_string());

        let options = CompileOptions::default();
        let result = compiler.compile_parallel_with_dependencies(&modules, options).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("Module1"));
        assert!(result.contains_key("Module2"));
    }
}
