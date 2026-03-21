use nargo_ir::{JsExpr, JsProgram, JsStmt};
use nargo_types::Result;
use std::collections::HashSet;

use crate::{
    rule::RuleEngine,
    types::{AnalysisReport, ScriptMetadata},
};

/// 脚本分析器
#[derive(Default)]
pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    /// 创建新的脚本分析器
    pub fn new() -> Self {
        Self
    }

    /// 分析脚本并生成元数据
    pub fn analyze(&self, program: &JsProgram) -> Result<ScriptMetadata> {
        let mut meta = ScriptMetadata::default();

        // 收集所有声明的变量和函数，用于依赖分析
        let mut declared_vars = HashSet::new();
        let mut declared_functions = HashSet::new();

        // 第一次遍历：收集所有声明
        for stmt in &program.body {
            match stmt {
                JsStmt::VariableDecl { id, init, .. } => {
                    self.analyze_variable_decl(id, init.as_ref(), &mut meta);
                    // 收集声明的变量
                    if id.starts_with('[') && id.ends_with(']') {
                        let content = &id[1..id.len() - 1];
                        for part in content.split(',') {
                            let trimmed = part.trim();
                            if !trimmed.is_empty() {
                                declared_vars.insert(trimmed.to_string());
                            }
                        }
                    }
                    else {
                        declared_vars.insert(id.to_string());
                    }
                }
                JsStmt::FunctionDecl { id, params: _, body: _, .. } => {
                    meta.actions.insert(id.clone());
                    declared_functions.insert(id.clone());
                }
                JsStmt::Expr(expr, _, _) => {
                    self.analyze_expression(expr, &mut meta);
                }
                _ => {}
            }
        }

        // 第二次遍历：分析依赖关系
        for stmt in &program.body {
            match stmt {
                JsStmt::VariableDecl { id, init, .. } => {
                    if let Some(init_expr) = init {
                        let mut deps = HashSet::new();
                        self.find_dependencies(init_expr, &mut deps, &meta);
                        if !deps.is_empty() {
                            meta.dependencies.insert(id.clone(), deps);
                        }
                    }
                }
                JsStmt::FunctionDecl { id, body, .. } => {
                    let mut deps = HashSet::new();
                    for s in body {
                        self.find_dependencies_in_stmt(s, &mut deps, &meta);
                    }
                    if !deps.is_empty() {
                        meta.dependencies.insert(id.clone(), deps);
                    }
                }
                JsStmt::Expr(expr, _, _) => {
                    // Track effect dependencies
                    if let JsExpr::Call { callee, args, .. } = expr {
                        if let JsExpr::Identifier(name, _, _) = &**callee {
                            if name == "$effect" || name == "watchEffect" {
                                if let Some(first_arg) = args.get(0) {
                                    let mut deps = HashSet::new();
                                    self.find_dependencies(first_arg, &mut deps, &meta);
                                    if !deps.is_empty() {
                                        // Store effect dependencies with a special key or handle separately
                                        meta.dependencies.insert(format!("$effect_{}", meta.dependencies.len()), deps);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(meta)
    }

    /// 执行静态分析规则检查
    pub fn analyze_with_rules(&self, program: &JsProgram, file_path: Option<String>) -> Result<(ScriptMetadata, AnalysisReport)> {
        let start_time = std::time::Instant::now();

        // 首先进行常规分析
        let meta = self.analyze(program)?;

        // 创建分析报告
        let mut report = AnalysisReport::new(file_path);

        // 使用默认规则引擎
        let rule_engine = super::default_rule_engine();

        // 执行规则检查
        rule_engine.run(program, &meta, &mut report);

        // 计算分析耗时
        let duration = start_time.elapsed();
        report.duration_ms = duration.as_millis() as u64;

        // 排序问题
        report.sort_issues();

        Ok((meta, report))
    }

    /// 分析变量声明
    fn analyze_variable_decl(&self, id: &str, init: Option<&JsExpr>, meta: &mut ScriptMetadata) {
        let mut ids = Vec::new();
        if id.starts_with('[') && id.ends_with(']') {
            // Simple array pattern parsing: [a, b, ...]
            let content = &id[1..id.len() - 1];
            for part in content.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    ids.push(trimmed.to_string());
                }
            }
        }
        else {
            ids.push(id.to_string());
        }

        // Handle props: const props = defineProps(...)
        if id == "props" {
            if let Some(JsExpr::Call { callee, args, .. }) = init {
                if let JsExpr::Identifier(name, _, _) = &**callee {
                    if name == "defineProps" {
                        self.extract_keys_from_args(args, &mut meta.props);
                        return;
                    }
                }
            }
        }

        // Handle emits: const emit = defineEmits(...)
        if id == "emit" || id == "emits" {
            if let Some(JsExpr::Call { callee, args, .. }) = init {
                if let JsExpr::Identifier(name, _, _) = &**callee {
                    if name == "defineEmits" {
                        self.extract_keys_from_args(args, &mut meta.emits);
                        return;
                    }
                }
            }
        }

        // Handle signals/computed
        if let Some(init_expr) = init {
            match init_expr {
                JsExpr::Call { callee, .. } => {
                    if let JsExpr::Identifier(name, _, _) = &**callee {
                        if name == "signal" || name == "createSignal" || name == "ref" || name == "reactive" {
                            for (i, var_id) in ids.iter().enumerate() {
                                if i == 0 || name == "ref" || name == "reactive" {
                                    meta.signals.insert(var_id.clone());
                                }
                            }
                        }
                        else if name == "computed" || name == "$computed" || name == "createComputed" {
                            for var_id in &ids {
                                meta.computed.insert(var_id.clone());
                            }
                        }
                    }
                }
                JsExpr::ArrowFunction { .. } => {
                    for var_id in &ids {
                        meta.actions.insert(var_id.clone());
                    }
                }
                _ => {}
            }
        }
    }

    /// 分析表达式
    fn analyze_expression(&self, expr: &JsExpr, meta: &mut ScriptMetadata) {
        if let JsExpr::Call { callee, args, .. } = expr {
            if let JsExpr::Identifier(name, _, _) = &**callee {
                match name.as_str() {
                    "defineProps" => {
                        self.extract_keys_from_args(args, &mut meta.props);
                    }
                    "defineEmits" => {
                        self.extract_keys_from_args(args, &mut meta.emits);
                    }
                    _ => {}
                }
            }
        }
    }

    /// 查找依赖关系
    fn find_dependencies(&self, expr: &JsExpr, deps: &mut HashSet<String>, meta: &ScriptMetadata) {
        match expr {
            JsExpr::Identifier(name, _, _) => {
                if meta.signals.contains(name) || meta.computed.contains(name) || meta.props.contains(name) {
                    deps.insert(name.clone());
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.find_dependencies(left, deps, meta);
                self.find_dependencies(right, deps, meta);
            }
            JsExpr::Unary { argument, .. } => {
                self.find_dependencies(argument, deps, meta);
            }
            JsExpr::Call { callee, args, .. } => {
                self.find_dependencies(callee, deps, meta);
                for arg in args {
                    self.find_dependencies(arg, deps, meta);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.find_dependencies(object, deps, meta);
                if *computed {
                    self.find_dependencies(property, deps, meta);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.find_dependencies(el, deps, meta);
                }
            }
            JsExpr::Object(properties, _, _) => {
                for value in properties.values() {
                    self.find_dependencies(value, deps, meta);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.find_dependencies(body, deps, meta);
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                self.find_dependencies(test, deps, meta);
                self.find_dependencies(consequent, deps, meta);
                self.find_dependencies(alternate, deps, meta);
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for e in expressions {
                    self.find_dependencies(e, deps, meta);
                }
            }
            _ => {}
        }
    }

    /// 在语句中查找依赖关系
    fn find_dependencies_in_stmt(&self, stmt: &JsStmt, deps: &mut HashSet<String>, meta: &ScriptMetadata) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.find_dependencies(expr, deps, meta),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(e) = init {
                    self.find_dependencies(e, deps, meta);
                }
            }
            JsStmt::Return(expr, _, _) => {
                if let Some(e) = expr {
                    self.find_dependencies(e, deps, meta);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                self.find_dependencies(test, deps, meta);
                self.find_dependencies_in_stmt(consequent, deps, meta);
                if let Some(alt) = alternate {
                    self.find_dependencies_in_stmt(alt, deps, meta);
                }
            }
            JsStmt::While { test, body, .. } => {
                self.find_dependencies(test, deps, meta);
                self.find_dependencies_in_stmt(body, deps, meta);
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(i) = init {
                    self.find_dependencies_in_stmt(i, deps, meta);
                }
                if let Some(t) = test {
                    self.find_dependencies(t, deps, meta);
                }
                if let Some(u) = update {
                    self.find_dependencies(u, deps, meta);
                }
                self.find_dependencies_in_stmt(body, deps, meta);
            }
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.find_dependencies_in_stmt(s, deps, meta);
                }
            }
            _ => {}
        }
    }

    /// 从参数中提取键
    fn extract_keys_from_args(&self, args: &[JsExpr], set: &mut HashSet<String>) {
        if let Some(first_arg) = args.get(0) {
            match first_arg {
                JsExpr::Object(map, _, _) => {
                    for key in map.keys() {
                        set.insert(key.clone());
                    }
                }
                JsExpr::Array(arr, _, _) => {
                    for item in arr {
                        if let JsExpr::Literal(nargo_types::NargoValue::String(s), _, _) = item {
                            set.insert(s.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
