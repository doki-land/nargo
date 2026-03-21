use nargo_ir::*;
use nargo_types::Result;
use std::collections::{HashMap, HashSet};

/// 树摇优化插件
///
/// 基于静态分析识别并移除未使用的代码
pub struct TreeShakingPass {
    /// 已使用的标识符集合
    used_identifiers: HashSet<String>,
    /// 函数定义映射
    function_defs: HashMap<String, JsStmt>,
    /// 变量定义映射
    variable_defs: HashMap<String, JsStmt>,
}

impl TreeShakingPass {
    /// 创建新的树摇优化插件
    pub fn new() -> Self {
        Self { used_identifiers: HashSet::new(), function_defs: HashMap::new(), variable_defs: HashMap::new() }
    }

    /// 收集所有定义的函数和变量
    fn collect_definitions(&mut self, stmts: &Vec<JsStmt>) {
        for stmt in stmts {
            match stmt {
                JsStmt::FunctionDecl { id, .. } => {
                    self.function_defs.insert(id.clone(), stmt.clone());
                }
                JsStmt::VariableDecl { id, .. } => {
                    self.variable_defs.insert(id.clone(), stmt.clone());
                }
                JsStmt::Block(body, _, _) => {
                    self.collect_definitions(body);
                }
                JsStmt::If { consequent, alternate, .. } => {
                    match **consequent {
                        JsStmt::Block(ref body, _, _) => {
                            self.collect_definitions(body);
                        }
                        _ => {}
                    }
                    if let Some(alt) = alternate {
                        match **alt {
                            JsStmt::Block(ref body, _, _) => {
                                self.collect_definitions(body);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// 标记使用的标识符
    fn mark_used_identifiers(&mut self, expr: &JsExpr) {
        match expr {
            JsExpr::Identifier(id, _, _) => {
                self.used_identifiers.insert(id.clone());
            }
            JsExpr::Unary { argument, .. } => {
                self.mark_used_identifiers(argument);
            }
            JsExpr::Binary { left, right, .. } => {
                self.mark_used_identifiers(left);
                self.mark_used_identifiers(right);
            }
            JsExpr::Call { callee, args, .. } => {
                self.mark_used_identifiers(callee);
                for arg in args {
                    self.mark_used_identifiers(arg);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.mark_used_identifiers(object);
                if *computed {
                    self.mark_used_identifiers(property);
                }
            }
            JsExpr::Array(items, ..) => {
                for item in items {
                    self.mark_used_identifiers(item);
                }
            }
            JsExpr::Object(props, ..) => {
                for (_, value) in props {
                    self.mark_used_identifiers(value);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.mark_used_identifiers(body);
            }
            JsExpr::TseElement { attributes, children, .. } => {
                for attr in attributes {
                    if let Some(value) = &attr.value {
                        self.mark_used_identifiers(value);
                    }
                }
                for child in children {
                    self.mark_used_identifiers(child);
                }
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                self.mark_used_identifiers(test);
                self.mark_used_identifiers(consequent);
                self.mark_used_identifiers(alternate);
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    self.mark_used_identifiers(expr);
                }
            }
            JsExpr::Spread(expr, ..) => {
                self.mark_used_identifiers(expr);
            }
            _ => {}
        }
    }

    /// 标记语句中使用的标识符
    fn mark_used_in_stmt(&mut self, stmt: &JsStmt) {
        match stmt {
            JsStmt::Expr(expr, _, _) => {
                self.mark_used_identifiers(expr);
            }
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.mark_used_identifiers(expr);
                }
            }
            JsStmt::Return(expr, _, _) => {
                if let Some(expr) = expr {
                    self.mark_used_identifiers(expr);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                self.mark_used_identifiers(test);
                self.mark_used_in_stmt(consequent);
                if let Some(alt) = alternate {
                    self.mark_used_in_stmt(alt);
                }
            }
            JsStmt::While { test, body, .. } => {
                self.mark_used_identifiers(test);
                self.mark_used_in_stmt(body);
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    self.mark_used_in_stmt(init_stmt);
                }
                if let Some(test_expr) = test {
                    self.mark_used_identifiers(test_expr);
                }
                if let Some(update_expr) = update {
                    self.mark_used_identifiers(update_expr);
                }
                self.mark_used_in_stmt(body);
            }
            JsStmt::Block(body, _, _) => {
                for stmt in body {
                    self.mark_used_in_stmt(stmt);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.mark_used_in_stmt(stmt);
                }
            }
            _ => {}
        }
    }

    /// 过滤未使用的语句
    fn filter_unused_stmts(&mut self, stmts: &mut Vec<JsStmt>) {
        let mut filtered = Vec::new();

        for stmt in &mut *stmts {
            let keep = match stmt {
                JsStmt::FunctionDecl { id, .. } => self.used_identifiers.contains(id),
                JsStmt::VariableDecl { id, .. } => self.used_identifiers.contains(id),
                JsStmt::Import { specifiers, .. } => {
                    // 检查是否有导入的标识符被使用
                    specifiers.iter().any(|spec| self.used_identifiers.contains(spec))
                }
                _ => true, // 其他语句保留
            };

            if keep {
                // 递归处理嵌套语句
                match stmt {
                    JsStmt::Block(body, _, _) => {
                        self.filter_unused_stmts(body);
                    }
                    JsStmt::If { consequent, alternate, .. } => {
                        if let JsStmt::Block(body, _, _) = &mut **consequent {
                            self.filter_unused_stmts(body);
                        }
                        if let Some(alt) = alternate {
                            if let JsStmt::Block(body, _, _) = &mut **alt {
                                self.filter_unused_stmts(body);
                            }
                        }
                    }
                    JsStmt::FunctionDecl { body, .. } => {
                        self.filter_unused_stmts(body);
                    }
                    _ => {}
                }
                filtered.push(stmt.clone());
            }
        }

        *stmts = filtered;
    }

    /// 处理脚本
    fn process_script(&mut self, script: &mut JsProgram) {
        // 1. 收集所有定义
        self.collect_definitions(&script.body);

        // 2. 标记使用的标识符
        for stmt in &script.body {
            self.mark_used_in_stmt(stmt);
        }

        // 3. 过滤未使用的语句
        self.filter_unused_stmts(&mut script.body);
    }
}

impl crate::TransformPass for TreeShakingPass {
    fn name(&self) -> String {
        "TreeShakingPass".to_string()
    }

    fn description(&self) -> String {
        "Removes unused code based on static analysis".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &mut ir.script {
            self.process_script(script);
        }
        if let Some(script_server) = &mut ir.script_server {
            self.process_script(script_server);
        }
        if let Some(script_client) = &mut ir.script_client {
            self.process_script(script_client);
        }
        Ok(())
    }
}
