#![warn(missing_docs)]

use nargo_ir::IRModule;
use nargo_types::{CompileMode, Result};

pub mod analyzer;
pub mod hmr;
pub mod runtime;
pub mod split;
pub mod targets;
pub mod types;

pub use types::*;

use rayon::prelude::*;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use ws::Sender;

use crate::targets::{CssBackend, DtsBackend, HtmlBackend, JsBackend, WasmBackend};

use crate::{
    analyzer::{analyze_dependencies, analyze_module_into, is_route_component},
    hmr::{generate_hmr_client, send_hmr_update, start_hmr_server},
    runtime::generate_custom_runtime,
    split::{split_by_component, split_by_route, split_custom, split_single_file},
    types::{BuildOutput, BuildOutputs, FeatureSet, FileFingerprint, ModuleCache, ModuleSystem, OutputFormat, SplitStrategy},
};

/// Nargo 打包器，支持多格式代码生成、热模块替换和增量构建
pub struct Bundler {
    /// 特征集
    pub feature_set: FeatureSet,
    /// 运行时路径
    pub runtime_path: Option<PathBuf>,
    /// 是否启用热模块替换
    pub hmr: bool,
    /// HMR 端口
    pub hmr_port: Option<u16>,
    /// HMR 服务器运行状态
    hmr_running: bool,
    /// WebSocket 连接列表
    connections: Arc<Mutex<Vec<Sender>>>,
    /// 模块缓存，键为模块名
    module_cache: Arc<Mutex<HashMap<String, ModuleCache>>>,
    /// 是否启用增量构建
    incremental: bool,
    /// 代码分割策略
    split_strategy: SplitStrategy,
    /// 是否启用动态导入支持
    dynamic_import: bool,
    /// 是否启用懒加载支持
    lazy_loading: bool,
    /// 模块系统
    module_system: ModuleSystem,
}

impl Bundler {
    /// 创建新的打包器实例
    pub fn new(runtime_path: Option<PathBuf>) -> Self {
        Self { feature_set: FeatureSet::default(), runtime_path, hmr: false, hmr_port: None, hmr_running: false, connections: Arc::new(Mutex::new(vec![])), module_cache: Arc::new(Mutex::new(HashMap::new())), incremental: true, split_strategy: SplitStrategy::SingleFile, dynamic_import: false, lazy_loading: false, module_system: ModuleSystem::ESM }
    }

    /// 启用热模块替换
    pub fn with_hmr(mut self, port: Option<u16>) -> Self {
        self.hmr = true;
        self.hmr_port = port;
        self
    }

    /// 设置是否启用增量构建
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }

    /// 设置代码分割策略
    pub fn with_split_strategy(mut self, strategy: SplitStrategy) -> Self {
        self.split_strategy = strategy;
        self
    }

    /// 设置是否启用动态导入支持
    pub fn with_dynamic_import(mut self, dynamic_import: bool) -> Self {
        self.dynamic_import = dynamic_import;
        self
    }

    /// 设置是否启用懒加载支持
    pub fn with_lazy_loading(mut self, lazy_loading: bool) -> Self {
        self.lazy_loading = lazy_loading;
        self
    }

    /// 设置模块系统
    pub fn with_module_system(mut self, module_system: ModuleSystem) -> Self {
        self.module_system = module_system;
        self
    }

    /// 计算文件指纹
    fn compute_fingerprint(&self, module: &IRModule) -> FileFingerprint {
        // 简单实现：使用模块名称和内容的哈希作为指纹
        let content = format!("{}{:?}", module.name, module);
        let content_hash = content.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
        FileFingerprint { modified_time: SystemTime::now(), content_hash }
    }

    /// 并行分析所有模块的特征
    pub fn analyze_all(&mut self, modules: &[IRModule]) {
        let feature_set = Arc::new(Mutex::new(FeatureSet::default()));

        modules.par_iter().for_each(|module| {
            let mut local_features = FeatureSet::default();
            analyze_module_into(module, &mut local_features);
            let mut global_features = feature_set.lock().unwrap();
            global_features.merge(&local_features);
        });

        self.feature_set = Arc::try_unwrap(feature_set).unwrap().into_inner().unwrap();
    }

    /// 启动 HMR 服务器
    pub fn start_hmr_server(&mut self) -> Result<()> {
        if !self.hmr || self.hmr_running {
            return Ok(());
        }

        let port = self.hmr_port.unwrap_or(3000);
        let connections = self.connections.clone();

        start_hmr_server(connections, port)?;

        self.hmr_running = true;
        Ok(())
    }

    /// 发送热更新通知
    pub fn send_hmr_update(&self, module_name: &str) -> Result<()> {
        send_hmr_update(&self.connections, module_name)
    }

    /// 并行编译并链接所有模块，生成指定格式的代码
    pub fn bundle_all(&mut self, modules: &[IRModule], format: OutputFormat) -> Result<BuildOutputs> {
        match format {
            OutputFormat::JavaScript => self.bundle_javascript(modules),
            OutputFormat::CSS => self.bundle_css(modules),
            OutputFormat::HTML => self.bundle_html(modules),
            OutputFormat::TypeScript => self.bundle_typescript(modules),
            OutputFormat::WebAssembly => self.bundle_wasm(modules),
        }
    }

    /// 并行编译并链接所有模块，默认生成 JavaScript 代码
    pub fn bundle_all_default(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        self.bundle_all(modules, OutputFormat::JavaScript)
    }

    /// 生成 JavaScript 代码
    fn bundle_javascript(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        let updated_feature_set = Arc::new(Mutex::new(FeatureSet::default()));
        let need_full_rebuild = Arc::new(Mutex::new(false));

        // 1. 分析模块并检查缓存
        let backend = JsBackend::new(false, false, None, CompileMode::Vue2);
        let compiled_modules: Vec<Result<(String, String)>> = modules
            .par_iter()
            .map(|module| {
                let mut module_needs_rebuild = false;
                let mut module_features = FeatureSet::default();
                let mut processed_code = String::new();

                if self.incremental {
                    let fingerprint = self.compute_fingerprint(module);
                    let module_cache = self.module_cache.lock().unwrap();
                    if let Some(cache) = module_cache.get(&module.name) {
                        if cache.fingerprint == fingerprint {
                            // 使用缓存的编译结果
                            let mut features = updated_feature_set.lock().unwrap();
                            features.merge(&cache.features);
                            return Ok((module.name.clone(), cache.compiled_code.clone()));
                        }
                    }
                    // 缓存未命中，需要重新编译
                    module_needs_rebuild = true;
                }

                if module_needs_rebuild || !self.incremental {
                    // 重新编译模块
                    let (code, _) = backend.generate(module)?;

                    // 后处理：移除模块化的导入导出，改为内部变量
                    processed_code = code.replace("import {", "// import {").replace("} from '@nargo/core';", " } = runtime;").replace("} from '@nargo/dom';", " } = runtime;").replace("} from '@nargo/client';", " } = runtime;").replace("export default ", &format!("const {} = ", module.name));

                    // 分析模块特征和依赖
                    analyze_module_into(module, &mut module_features);
                    let dependencies = analyze_dependencies(module);

                    // 更新缓存
                    if self.incremental {
                        let fingerprint = self.compute_fingerprint(module);
                        let mut cache = self.module_cache.lock().unwrap();
                        cache.insert(module.name.clone(), ModuleCache { fingerprint, compiled_code: processed_code.clone(), dependencies, features: module_features.clone() });
                    }

                    // 更新全局特征集
                    let mut features = updated_feature_set.lock().unwrap();
                    features.merge(&module_features);

                    // 标记需要完全重建
                    let mut rebuild = need_full_rebuild.lock().unwrap();
                    *rebuild = true;
                }

                Ok((module.name.clone(), processed_code))
            })
            .collect();

        // 2. 更新全局特征集
        let needs_rebuild = *need_full_rebuild.lock().unwrap();
        if needs_rebuild || !self.incremental {
            self.feature_set = Arc::try_unwrap(updated_feature_set).unwrap().into_inner().unwrap();
        }

        // 3. 生成运行时代码
        let runtime_code = generate_custom_runtime(&self.feature_set);

        // 4. 生成 HMR 客户端代码
        let hmr_code = if self.hmr {
            self.start_hmr_server()?;
            let port = self.hmr_port.unwrap_or(3000);
            generate_hmr_client(port)
        }
        else {
            String::new()
        };

        // 5. 准备编译后的模块数据
        let compiled_modules: Vec<(String, String)> = compiled_modules.into_iter().map(|res| res.unwrap()).collect();

        // 6. 根据分割策略生成输出
        let mut outputs = match self.split_strategy {
            SplitStrategy::SingleFile => split_single_file(&runtime_code, &hmr_code, &compiled_modules, modules),
            SplitStrategy::ByComponent => split_by_component(&runtime_code, &hmr_code, &compiled_modules, modules),
            SplitStrategy::ByRoute => split_by_route(&runtime_code, &hmr_code, &compiled_modules, modules),
            SplitStrategy::Custom => split_custom(&runtime_code, &hmr_code, &compiled_modules, modules),
        };

        // 7. 根据模块系统包装输出
        self.wrap_with_module_system(&mut outputs);

        Ok(outputs)
    }

    /// 根据模块系统包装输出
    fn wrap_with_module_system(&self, outputs: &mut BuildOutputs) {
        for output in &mut outputs.outputs {
            if output.format == OutputFormat::JavaScript {
                let code = String::from_utf8_lossy(&output.code).to_string();
                let wrapped_code = match self.module_system {
                    ModuleSystem::ESM => self.wrap_esm(&code),
                    ModuleSystem::CommonJS => self.wrap_commonjs(&code),
                    ModuleSystem::UMD => self.wrap_umd(&code),
                    ModuleSystem::IIFE => self.wrap_iife(&code),
                };
                output.code = wrapped_code.into_bytes();
            }
        }
    }

    /// 包装为 ES 模块
    fn wrap_esm(&self, code: &str) -> String {
        format!("{}\nexport default App;", code)
    }

    /// 包装为 CommonJS 模块
    fn wrap_commonjs(&self, code: &str) -> String {
        format!("{}\nmodule.exports = App;", code)
    }

    /// 包装为 UMD 模块
    fn wrap_umd(&self, code: &str) -> String {
        format!(
            "(function(root, factory) {{
    if (typeof define === 'function' && define.amd) {{
        define([], factory);
    }} else if (typeof module === 'object' && module.exports) {{
        module.exports = factory();
    }} else {{
        root.App = factory();
    }}
}}(typeof self !== 'undefined' ? self : this, function() {{
    {}    return App;
}}));",
            code
        )
    }

    /// 包装为 IIFE 模块
    fn wrap_iife(&self, code: &str) -> String {
        format!(
            "(function() {{
    {}    window.App = App;
}})();",
            code
        )
    }

    /// 生成 CSS 代码
    fn bundle_css(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        let mut output = String::new();
        let backend = CssBackend::new(false);

        for module in modules {
            let css = backend.generate(module)?;
            if !css.is_empty() {
                output.push_str(&format!("/* Component: {} */\n", module.name));
                output.push_str(&css);
                output.push_str("\n");
            }
        }

        let build_output = BuildOutput { code: output.into_bytes(), format: OutputFormat::CSS, filename: "bundle.css".to_string() };
        Ok(BuildOutputs { outputs: vec![build_output], entry_file: "bundle.css".to_string() })
    }

    /// 生成 HTML 代码
    fn bundle_html(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        let mut output = String::new();
        let backend = HtmlBackend::new();

        for module in modules {
            let html = backend.generate(module)?;
            if !html.is_empty() {
                output.push_str(&format!("<!-- Component: {} -->\n", module.name));
                output.push_str(&html);
                output.push_str("\n");
            }
        }

        let build_output = BuildOutput { code: output.into_bytes(), format: OutputFormat::HTML, filename: "bundle.html".to_string() };
        Ok(BuildOutputs { outputs: vec![build_output], entry_file: "bundle.html".to_string() })
    }

    /// 生成 TypeScript 类型定义
    fn bundle_typescript(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        let mut output = String::new();
        let backend = DtsBackend::new();

        for module in modules {
            let dts = backend.generate(module)?;
            if !dts.is_empty() {
                output.push_str(&format!("// Component: {}\n", module.name));
                output.push_str(&dts);
                output.push_str("\n");
            }
        }

        let build_output = BuildOutput { code: output.into_bytes(), format: OutputFormat::TypeScript, filename: "bundle.d.ts".to_string() };
        Ok(BuildOutputs { outputs: vec![build_output], entry_file: "bundle.d.ts".to_string() })
    }

    /// 生成 WebAssembly 代码
    fn bundle_wasm(&mut self, modules: &[IRModule]) -> Result<BuildOutputs> {
        let mut output = Vec::new();
        let backend = WasmBackend::new(false);

        for module in modules {
            let wasm = backend.generate(module)?;
            if !wasm.is_empty() {
                output.extend(wasm);
            }
        }

        let build_output = BuildOutput { code: output, format: OutputFormat::WebAssembly, filename: "bundle.wasm".to_string() };
        Ok(BuildOutputs { outputs: vec![build_output], entry_file: "bundle.wasm".to_string() })
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new(None)
    }
}
