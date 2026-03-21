//! Build and compile commands.
//!
//! This module provides production build and compiler functionality.

use color_eyre::eyre::Result;
use nargo_bundler::{
    Bundler,
    types::{BuildOutputs, OutputFormat},
};
use nargo_compiler::Compiler;
use nargo_config::ConfigLoader;
use nargo_ir::IRModule;
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// 构建配置
///
/// 包含构建过程中所需的所有配置选项
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// 项目根目录
    pub root: PathBuf,
    /// 输出目录
    pub out_dir: PathBuf,
    /// 是否启用生产模式
    pub is_production: bool,
    /// 是否启用源码映射
    pub source_map: bool,
    /// 是否启用增量构建
    pub incremental: bool,
}

impl BuildConfig {
    /// 创建新的构建配置
    ///
    /// # Arguments
    ///
    /// * `root` - 项目根目录
    /// * `out_dir` - 输出目录
    ///
    /// # Returns
    ///
    /// 构建配置实例
    pub fn new(root: PathBuf, out_dir: PathBuf) -> Self {
        Self { root, out_dir, is_production: true, source_map: true, incremental: true }
    }

    /// 设置是否为生产模式
    ///
    /// # Arguments
    ///
    /// * `is_production` - 是否启用生产模式
    ///
    /// # Returns
    ///
    /// 更新后的构建配置
    pub fn with_production(mut self, is_production: bool) -> Self {
        self.is_production = is_production;
        self
    }

    /// 设置是否启用源码映射
    ///
    /// # Arguments
    ///
    /// * `source_map` - 是否启用源码映射
    ///
    /// # Returns
    ///
    /// 更新后的构建配置
    pub fn with_source_map(mut self, source_map: bool) -> Self {
        self.source_map = source_map;
        self
    }

    /// 设置是否启用增量构建
    ///
    /// # Arguments
    ///
    /// * `incremental` - 是否启用增量构建
    ///
    /// # Returns
    ///
    /// 更新后的构建配置
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }
}

/// 构建上下文
///
/// 包含构建过程中的状态和工具
pub struct BuildContext {
    /// 构建配置
    pub config: BuildConfig,
    /// Nargo 配置
    pub nargo_config: nargo_config::NargoConfig,
    /// 编译器实例
    pub compiler: Compiler,
    /// 打包器实例
    pub bundler: Bundler,
}

impl BuildContext {
    /// 创建新的构建上下文
    ///
    /// # Arguments
    ///
    /// * `config` - 构建配置
    ///
    /// # Returns
    ///
    /// 构建上下文实例
    pub async fn new(config: BuildConfig) -> Result<Self> {
        let config_path = nargo_config::ConfigLoader::find_config_file(&config.root).unwrap_or_else(|| config.root.join(nargo_config::DEFAULT_CONFIG_FILE));

        let config_loader = nargo_config::ConfigLoader::new(config_path);
        let nargo_config = config_loader.load()?;

        let compiler = Compiler::new();
        let bundler = Bundler::new(None);

        Ok(Self { config, nargo_config, compiler, bundler })
    }

    /// 加载并编译所有源文件
    ///
    /// # Returns
    ///
    /// 编译后的 IR 模块列表
    pub fn load_and_compile_files(&mut self) -> Result<Vec<IRModule>> {
        let source_files = self.find_source_files()?;
        let mut modules = Vec::with_capacity(source_files.len());

        for file_path in source_files {
            let module = self.compile_file(&file_path)?;
            modules.push(module);
        }

        Ok(modules)
    }

    /// 查找项目中的所有源文件
    ///
    /// # Returns
    ///
    /// 源文件路径列表
    fn find_source_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let src_dir = self.config.root.join("src");

        if !src_dir.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()).map(|ext| matches!(ext, "nargo" | "vmz" | "ts" | "tsx")).unwrap_or(false) {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// 编译单个文件为 IR 模块
    ///
    /// # Arguments
    ///
    /// * `file_path` - 文件路径
    ///
    /// # Returns
    ///
    /// 编译后的 IR 模块
    fn compile_file(&mut self, file_path: &Path) -> Result<IRModule> {
        let content = fs::read_to_string(file_path)?;
        let file_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        let mut options = nargo_compiler::CompileOptions::default();
        options.is_prod = self.config.is_production;

        let ir = self.compiler.compile_to_ir(file_name, &content, &mut options)?;
        Ok(ir)
    }

    /// 执行构建流程
    ///
    /// # Returns
    ///
    /// 构建输出
    pub fn build(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        // 打包步骤：使用 nargo 打包器打包
        self.bundler.analyze_all(modules);
        let outputs = self.bundler.bundle_all(modules, OutputFormat::JavaScript)?;

        Ok(outputs)
    }

    /// 写入构建输出到文件系统
    ///
    /// # Arguments
    ///
    /// * `outputs` - 构建输出
    pub fn write_outputs(&self, outputs: &BuildOutputs) -> Result<()> {
        fs::create_dir_all(&self.config.out_dir)?;

        for output in &outputs.outputs {
            let output_path = self.config.out_dir.join(&output.filename);
            fs::write(&output_path, &output.code)?;
        }

        Ok(())
    }
}

/// 执行构建命令
///
/// # Arguments
///
/// * `root` - 项目根目录
/// * `out_dir` - 输出目录
///
/// # Returns
///
/// 执行结果
pub async fn execute_build(root: &PathBuf, out_dir: &PathBuf) -> Result<()> {
    println!("🏗️ Building project in {} to {}", root.display(), out_dir.display());

    let config = BuildConfig::new(root.clone(), out_dir.clone());
    let mut context = BuildContext::new(config).await?;

    println!("📦 Loading and compiling source files...");
    let modules = context.load_and_compile_files()?;

    if modules.is_empty() {
        println!("⚠️ No source files found");
        return Ok(());
    }

    println!("🔗 Bundling modules...");
    let outputs = context.build(&modules)?;

    println!("💾 Writing output files...");
    context.write_outputs(&outputs)?;

    println!("✅ Build completed! Generated {} files", outputs.outputs.len());
    Ok(())
}

/// 执行编译命令
///
/// # Arguments
///
/// * `root` - 项目根目录
/// * `watch` - 是否启用监听模式
///
/// # Returns
///
/// 执行结果
pub async fn execute_compile(root: &PathBuf, watch: bool) -> Result<()> {
    println!("🔄 Compiling project in {}", root.display());
    if watch {
        println!("👁️  Watch mode enabled");
    }

    let out_dir = root.join("dist");
    let config = BuildConfig::new(root.clone(), out_dir);
    let mut context = BuildContext::new(config).await?;

    let modules = context.load_and_compile_files()?;

    if modules.is_empty() {
        println!("⚠️ No source files found");
        return Ok(());
    }

    println!("✅ Compilation completed! Compiled {} modules", modules.len());
    Ok(())
}
