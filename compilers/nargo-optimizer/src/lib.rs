#![warn(missing_docs)]

use nargo_ir::IRModule;
use nargo_transformer::{CallCountPass, ConstantFoldingPass, DeadCodeEliminationPass, I18nPass, ScopedCssPass, StaticHoistingPass, StyleAnalysisPass, Transformer, TreeShakingPass};
use nargo_types::{NargoValue, Result};

/// 目标环境
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetEnvironment {
    /// 浏览器环境
    Browser,
    /// 服务器端渲染环境
    SSR,
    /// 移动应用环境
    Mobile,
    /// 桌面应用环境
    Desktop,
    /// 嵌入式环境
    Embedded,
}

impl Default for TargetEnvironment {
    fn default() -> Self {
        Self::Browser
    }
}

/// 环境特定优化配置
#[derive(Debug, Clone, Default)]
pub struct EnvironmentOptimizationConfig {
    /// 目标环境
    pub target: TargetEnvironment,
    /// 是否启用内存优化
    pub memory_optimization: bool,
    /// 是否启用性能优化
    pub performance_optimization: bool,
    /// 是否启用大小优化
    pub size_optimization: bool,
    /// 特定环境的额外配置
    pub extra_config: std::collections::HashMap<String, String>,
}

/// 优化级别
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 基本优化
    Basic,
    /// 标准优化
    Standard,
    /// 激进优化
    Aggressive,
    /// 极致优化（针对性能关键场景）
    Extreme,
}

/// Nargo 优化器
///
/// 负责对 IR 模块进行各种优化，包括常量折叠、死代码消除、静态提升等。
pub struct Optimizer {
    /// 上次生成的 CSS
    pub last_css: String,
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    /// 环境优化配置
    pub environment_config: EnvironmentOptimizationConfig,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    /// 创建新的优化器实例
    pub fn new() -> Self {
        Self { last_css: String::new(), optimization_level: OptimizationLevel::Standard, environment_config: EnvironmentOptimizationConfig::default() }
    }

    /// 设置优化级别
    ///
    /// # Arguments
    ///
    /// * `level` - 优化级别
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }

    /// 设置环境优化配置
    ///
    /// # Arguments
    ///
    /// * `config` - 环境优化配置
    pub fn set_environment_config(&mut self, config: EnvironmentOptimizationConfig) {
        self.environment_config = config;
    }

    /// 设置目标环境
    ///
    /// # Arguments
    ///
    /// * `target` - 目标环境
    pub fn set_target_environment(&mut self, target: TargetEnvironment) {
        self.environment_config.target = target;
    }

    /// 优化 IR 模块
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    /// * `locale` - 国际化语言环境
    /// * `is_prod` - 是否为生产环境
    pub fn optimize(&mut self, ir: &mut IRModule, locale: Option<&str>, is_prod: bool) {
        match self.optimization_level {
            OptimizationLevel::None => {
                // 无优化
                return;
            }
            OptimizationLevel::Basic => {
                // 基本优化
                self.apply_basic_optimizations(ir, locale);
            }
            OptimizationLevel::Standard => {
                // 标准优化
                self.apply_basic_optimizations(ir, locale);
                self.apply_standard_optimizations(ir);
            }
            OptimizationLevel::Aggressive => {
                // 激进优化
                self.apply_basic_optimizations(ir, locale);
                self.apply_standard_optimizations(ir);
                self.apply_aggressive_optimizations(ir);
            }
            OptimizationLevel::Extreme => {
                // 极致优化
                self.apply_basic_optimizations(ir, locale);
                self.apply_standard_optimizations(ir);
                self.apply_aggressive_optimizations(ir);
                self.apply_extreme_optimizations(ir);
            }
        }

        // 应用环境特定优化
        self.apply_environment_specific_optimizations(ir);

        if is_prod {
            // 生产环境的额外优化
            self.apply_production_optimizations(ir);
        }
    }

    /// 应用基本优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    /// * `locale` - 国际化语言环境
    fn apply_basic_optimizations(&self, ir: &mut IRModule, locale: Option<&str>) {
        let mut transformer = Transformer::new();

        // 1. 常量折叠
        let mut folding_pass = ConstantFoldingPass::new();
        let _ = transformer.apply(ir, &mut folding_pass);

        // 2. 死代码消除
        let mut dce_pass = DeadCodeEliminationPass::new();
        let _ = transformer.apply(ir, &mut dce_pass);

        // 3. 国际化优化
        if let Some(locale_val) = locale {
            if let Some(i18n_map) = &ir.i18n {
                if let Some(messages) = i18n_map.get(locale_val) {
                    let mut i18n_pass = I18nPass::new(messages.clone());
                    let _ = transformer.apply(ir, &mut i18n_pass);
                }
            }
        }
    }

    /// 应用标准优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_standard_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 静态提升
        let mut hoisting_pass = StaticHoistingPass::new();
        let _ = transformer.apply(ir, &mut hoisting_pass);

        // 2. 样式分析和提取
        let mut style_pass = StyleAnalysisPass::new();
        let _ = transformer.apply(ir, &mut style_pass);

        // 3. 调用计数分析（用于内联决策）
        let mut call_count_pass = CallCountPass::new();
        let _ = transformer.apply(ir, &mut call_count_pass);
    }

    /// 应用激进优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_aggressive_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 树摇优化
        let mut tree_shaking_pass = TreeShakingPass::new();
        let _ = transformer.apply(ir, &mut tree_shaking_pass);
    }

    /// 应用极致优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_extreme_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 函数内联（针对小函数）
        // 这里需要实现函数内联逻辑

        // 2. 循环展开（针对小循环）
        // 这里需要实现循环展开逻辑

        // 3. 更激进的常量折叠
        let mut folding_pass = ConstantFoldingPass::new();
        let _ = transformer.apply(ir, &mut folding_pass);

        // 4. 内存访问优化
        // 这里需要实现内存访问优化逻辑

        // 5. 指令重排序
        // 这里需要实现指令重排序逻辑

        // 6. 再次运行树摇（确保所有优化后没有新的未使用代码）
        let mut tree_shaking_pass = TreeShakingPass::new();
        let _ = transformer.apply(ir, &mut tree_shaking_pass);
    }

    /// 应用生产环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_production_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 再次运行死代码消除（确保所有优化后没有新的死代码）
        let mut dce_pass = DeadCodeEliminationPass::new();
        let _ = transformer.apply(ir, &mut dce_pass);

        // 2. 再次运行树摇（确保所有优化后没有新的未使用代码）
        let mut tree_shaking_pass = TreeShakingPass::new();
        let _ = transformer.apply(ir, &mut tree_shaking_pass);

        // 3. 清理 IR 模块中的冗余信息
        ir.cleanup();
    }

    /// 应用环境特定优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_environment_specific_optimizations(&self, ir: &mut IRModule) {
        match self.environment_config.target {
            TargetEnvironment::Browser => {
                self.apply_browser_optimizations(ir);
            }
            TargetEnvironment::SSR => {
                self.apply_ssr_optimizations(ir);
            }
            TargetEnvironment::Mobile => {
                self.apply_mobile_optimizations(ir);
            }
            TargetEnvironment::Desktop => {
                self.apply_desktop_optimizations(ir);
            }
            TargetEnvironment::Embedded => {
                self.apply_embedded_optimizations(ir);
            }
        }
    }

    /// 应用浏览器环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_browser_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 针对浏览器的大小优化
        if self.environment_config.size_optimization {
            // 额外的树摇优化
            let mut tree_shaking_pass = TreeShakingPass::new();
            let _ = transformer.apply(ir, &mut tree_shaking_pass);
        }

        // 2. 针对浏览器的性能优化
        if self.environment_config.performance_optimization {
            // 静态提升优化
            let mut hoisting_pass = StaticHoistingPass::new();
            let _ = transformer.apply(ir, &mut hoisting_pass);
        }
    }

    /// 应用服务器端渲染环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_ssr_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 针对 SSR 的内存优化
        if self.environment_config.memory_optimization {
            // 死代码消除
            let mut dce_pass = DeadCodeEliminationPass::new();
            let _ = transformer.apply(ir, &mut dce_pass);
        }

        // 2. 针对 SSR 的性能优化
        if self.environment_config.performance_optimization {
            // 常量折叠
            let mut folding_pass = ConstantFoldingPass::new();
            let _ = transformer.apply(ir, &mut folding_pass);
        }
    }

    /// 应用移动应用环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_mobile_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 针对移动应用的大小优化
        if self.environment_config.size_optimization {
            // 树摇优化
            let mut tree_shaking_pass = TreeShakingPass::new();
            let _ = transformer.apply(ir, &mut tree_shaking_pass);
        }

        // 2. 针对移动应用的内存优化
        if self.environment_config.memory_optimization {
            // 死代码消除
            let mut dce_pass = DeadCodeEliminationPass::new();
            let _ = transformer.apply(ir, &mut dce_pass);
        }
    }

    /// 应用桌面应用环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_desktop_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 针对桌面应用的性能优化
        if self.environment_config.performance_optimization {
            // 静态提升
            let mut hoisting_pass = StaticHoistingPass::new();
            let _ = transformer.apply(ir, &mut hoisting_pass);

            // 调用计数分析
            let mut call_count_pass = CallCountPass::new();
            let _ = transformer.apply(ir, &mut call_count_pass);
        }
    }

    /// 应用嵌入式环境优化
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    fn apply_embedded_optimizations(&self, ir: &mut IRModule) {
        let mut transformer = Transformer::new();

        // 1. 针对嵌入式环境的大小优化（最优先）
        if self.environment_config.size_optimization {
            // 树摇优化
            let mut tree_shaking_pass = TreeShakingPass::new();
            let _ = transformer.apply(ir, &mut tree_shaking_pass);
        }

        // 2. 针对嵌入式环境的内存优化
        if self.environment_config.memory_optimization {
            // 死代码消除
            let mut dce_pass = DeadCodeEliminationPass::new();
            let _ = transformer.apply(ir, &mut dce_pass);
        }

        // 3. 针对嵌入式环境的性能优化
        if self.environment_config.performance_optimization {
            // 常量折叠
            let mut folding_pass = ConstantFoldingPass::new();
            let _ = transformer.apply(ir, &mut folding_pass);
        }
    }

    /// 处理样式
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    ///
    /// # Returns
    ///
    /// 处理结果
    pub fn process_styles(&mut self, ir: &IRModule) -> Result<()> {
        let mut final_css = String::new();

        // 1. 处理常规 CSS 样式
        for style in &ir.styles {
            final_css.push_str(&style.code);
            final_css.push('\n');
        }

        // 2. 处理收集的类（Tailwind）
        if let Some(NargoValue::String(classes)) = ir.metadata.get("collected_styles") {
            // 生成 Tailwind CSS 代码
            final_css.push_str(&self.generate_tailwind_css(classes));
        }

        self.last_css = final_css;
        Ok(())
    }

    /// 生成 Tailwind CSS 代码
    ///
    /// # Arguments
    ///
    /// * `classes` - 类名列表
    ///
    /// # Returns
    ///
    /// CSS 代码
    fn generate_tailwind_css(&self, classes: &str) -> String {
        let mut css = String::new();

        // 简单的 Tailwind 类名到 CSS 的映射
        let class_map = std::collections::HashMap::from([("p-4", "padding: 1rem;"), ("p-6", "padding: 1.5rem;"), ("m-2", "margin: 0.5rem;"), ("m-4", "margin: 1rem;"), ("flex", "display: flex;"), ("items-center", "align-items: center;"), ("bg-blue", "background-color: #0000ff;"), ("text-white", "color: #ffffff;"), ("text-blue", "color: #0000ff;"), ("rounded-lg", "border-radius: 0.5rem;"), ("shadow-sm", "box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);"), ("font-bold", "font-weight: 700;"), ("text-2xl", "font-size: 1.5rem;"), ("text-center", "text-align: center;"), ("mx-4", "margin-left: 1rem; margin-right: 1rem;")]);

        for class in classes.split_whitespace() {
            if let Some(style) = class_map.get(class) {
                css.push_str(&format!(".{} {{ {} }}\n", class, style));
            }
        }

        css
    }

    /// 获取生成的 CSS
    ///
    /// # Returns
    ///
    /// CSS 代码
    pub fn get_css(&self) -> String {
        self.last_css.clone()
    }

    /// 应用作用域 ID
    ///
    /// # Arguments
    ///
    /// * `ir` - IR 模块
    /// * `scope_id` - 作用域 ID
    pub fn apply_scope_id(&mut self, ir: &mut IRModule, scope_id: &str) {
        let mut transformer = Transformer::new();
        let mut pass = ScopedCssPass::new(scope_id.to_string());
        let _ = transformer.apply(ir, &mut pass);
    }

    /// 生成作用域 ID
    ///
    /// # Arguments
    ///
    /// * `name` - 组件名称
    ///
    /// # Returns
    ///
    /// 作用域 ID
    pub fn generate_scope_id(&self, name: &str) -> String {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        format!("data-h-{:x}", hash)
    }

    /// 设置是否启用详细优化（兼容旧接口）
    ///
    /// # Arguments
    ///
    /// * `enabled` - 是否启用
    pub fn set_detailed_optimizations(&mut self, enabled: bool) {
        if enabled {
            self.optimization_level = OptimizationLevel::Aggressive;
        }
        else {
            self.optimization_level = OptimizationLevel::Basic;
        }
    }
}
