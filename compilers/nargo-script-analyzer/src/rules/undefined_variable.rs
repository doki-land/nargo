use nargo_ir::{JsExpr, JsStmt};
use std::collections::HashSet;

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 未定义变量使用检查规则
pub struct UndefinedVariableRule;

impl Rule for UndefinedVariableRule {
    fn code(&self) -> String {
        "undefined-variable".to_string()
    }

    fn description(&self) -> String {
        "检查使用未定义的变量".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, meta: &ScriptMetadata, report: &mut AnalysisReport) {
        let mut declared_vars = HashSet::new();

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
                JsStmt::FunctionDecl { id, .. } => {
                    declared_vars.insert(id.clone());
                }
                _ => {}
            }
        }

        // 检查未定义变量的使用
        fn check_undefined_vars(expr: &JsExpr, declared_vars: &HashSet<String>, meta: &ScriptMetadata, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Identifier(name, span, _) => {
                    if !declared_vars.contains(name) && !meta.signals.contains(name) && !meta.computed.contains(name) && !meta.props.contains(name) && name != "props" && name != "emit" && name != "emits" && !name.starts_with('$') {
                        report.add_issue(crate::types::IssueLevel::Error, "undefined-variable".to_string(), format!("Undefined variable: {}", name), Some(*span));
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_undefined_vars(left, declared_vars, meta, report);
                    check_undefined_vars(right, declared_vars, meta, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_undefined_vars(argument, declared_vars, meta, report);
                }
                JsExpr::Call { callee, args, .. } => {
                    check_undefined_vars(callee, declared_vars, meta, report);
                    for arg in args {
                        check_undefined_vars(arg, declared_vars, meta, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_undefined_vars(object, declared_vars, meta, report);
                    if *computed {
                        check_undefined_vars(property, declared_vars, meta, report);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_undefined_vars(el, declared_vars, meta, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_undefined_vars(value, declared_vars, meta, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_undefined_vars(body, declared_vars, meta, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_undefined_vars(test, declared_vars, meta, report);
                    check_undefined_vars(consequent, declared_vars, meta, report);
                    check_undefined_vars(alternate, declared_vars, meta, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_undefined_vars(e, declared_vars, meta, report);
                    }
                }
                _ => {}
            }
        }

        fn check_undefined_vars_in_stmt(stmt: &JsStmt, declared_vars: &HashSet<String>, meta: &ScriptMetadata, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_undefined_vars(expr, declared_vars, meta, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_undefined_vars(e, declared_vars, meta, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_undefined_vars(e, declared_vars, meta, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_undefined_vars(test, declared_vars, meta, report);
                    check_undefined_vars_in_stmt(consequent, declared_vars, meta, report);
                    if let Some(alt) = alternate {
                        check_undefined_vars_in_stmt(alt, declared_vars, meta, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_undefined_vars(test, declared_vars, meta, report);
                    check_undefined_vars_in_stmt(body, declared_vars, meta, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_undefined_vars_in_stmt(i, declared_vars, meta, report);
                    }
                    if let Some(t) = test {
                        check_undefined_vars(t, declared_vars, meta, report);
                    }
                    if let Some(u) = update {
                        check_undefined_vars(u, declared_vars, meta, report);
                    }
                    check_undefined_vars_in_stmt(body, declared_vars, meta, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_undefined_vars_in_stmt(s, declared_vars, meta, report);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            check_undefined_vars_in_stmt(stmt, &declared_vars, meta, report);
        }
    }
}
