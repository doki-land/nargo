use nargo_ir::{JsExpr, JsStmt};
use std::collections::HashSet;

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 未使用变量检查规则
pub struct UnusedVariableRule;

impl Rule for UnusedVariableRule {
    fn code(&self) -> String {
        "unused-variable".to_string()
    }

    fn description(&self) -> String {
        "检查未使用的变量".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, meta: &ScriptMetadata, report: &mut AnalysisReport) {
        let mut declared_vars = HashSet::new();
        let mut used_vars = HashSet::new();

        // 收集所有声明的变量
        for stmt in &program.body {
            match stmt {
                JsStmt::VariableDecl { id, .. } => {
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
                _ => {}
            }
        }

        // 收集所有使用的变量
        fn collect_used_vars(expr: &JsExpr, used_vars: &mut HashSet<String>) {
            match expr {
                JsExpr::Identifier(name, _, _) => {
                    used_vars.insert(name.clone());
                }
                JsExpr::Binary { left, right, .. } => {
                    collect_used_vars(left, used_vars);
                    collect_used_vars(right, used_vars);
                }
                JsExpr::Unary { argument, .. } => {
                    collect_used_vars(argument, used_vars);
                }
                JsExpr::Call { callee, args, .. } => {
                    collect_used_vars(callee, used_vars);
                    for arg in args {
                        collect_used_vars(arg, used_vars);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    collect_used_vars(object, used_vars);
                    if *computed {
                        collect_used_vars(property, used_vars);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        collect_used_vars(el, used_vars);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        collect_used_vars(value, used_vars);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    collect_used_vars(body, used_vars);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    collect_used_vars(test, used_vars);
                    collect_used_vars(consequent, used_vars);
                    collect_used_vars(alternate, used_vars);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        collect_used_vars(e, used_vars);
                    }
                }
                _ => {}
            }
        }

        fn collect_used_vars_in_stmt(stmt: &JsStmt, used_vars: &mut HashSet<String>) {
            match stmt {
                JsStmt::Expr(expr, _, _) => collect_used_vars(expr, used_vars),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        collect_used_vars(e, used_vars);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        collect_used_vars(e, used_vars);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    collect_used_vars(test, used_vars);
                    collect_used_vars_in_stmt(consequent, used_vars);
                    if let Some(alt) = alternate {
                        collect_used_vars_in_stmt(alt, used_vars);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    collect_used_vars(test, used_vars);
                    collect_used_vars_in_stmt(body, used_vars);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        collect_used_vars_in_stmt(i, used_vars);
                    }
                    if let Some(t) = test {
                        collect_used_vars(t, used_vars);
                    }
                    if let Some(u) = update {
                        collect_used_vars(u, used_vars);
                    }
                    collect_used_vars_in_stmt(body, used_vars);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        collect_used_vars_in_stmt(s, used_vars);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            collect_used_vars_in_stmt(stmt, &mut used_vars);
        }

        // 检查未使用的变量
        for var in &declared_vars {
            if !used_vars.contains(var) && !meta.signals.contains(var) && !meta.computed.contains(var) && !meta.actions.contains(var) {
                report.add_issue(crate::types::IssueLevel::Warning, self.code(), format!("Unused variable: {}", var), None);
            }
        }
    }
}
