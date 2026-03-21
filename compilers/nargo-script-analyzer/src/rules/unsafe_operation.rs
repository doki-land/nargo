use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 不安全操作检查规则
pub struct UnsafeOperationRule;

impl Rule for UnsafeOperationRule {
    fn code(&self) -> String {
        "unsafe-operation".to_string()
    }

    fn description(&self) -> String {
        "检查不安全的操作".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查不安全操作
        fn check_unsafe_operations(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Binary { left, right, op, span, .. } => {
                    // 检查除以零
                    if *op == "/" {
                        if let JsExpr::Literal(nargo_types::NargoValue::Number(0.0), _, _) = &**right {
                            report.add_issue(crate::types::IssueLevel::Error, "unsafe-operation".to_string(), "Division by zero".to_string(), Some(*span));
                        }
                    }
                    // 检查类型不安全的比较
                    if *op == "==" || *op == "!=" {
                        report.add_issue(crate::types::IssueLevel::Warning, "unsafe-operation".to_string(), "Use of loose equality operator (==/!=) may cause type coercion issues, consider using strict equality (===/!==)".to_string(), Some(*span));
                    }
                    check_unsafe_operations(left, report);
                    check_unsafe_operations(right, report);
                }
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查 eval 调用
                    if let JsExpr::Identifier(name, _, _) = &**callee {
                        if name == "eval" {
                            report.add_issue(crate::types::IssueLevel::Warning, "unsafe-operation".to_string(), "Use of eval is potentially unsafe".to_string(), Some(*span));
                        }
                        // 检查 Function 构造函数
                        else if name == "Function" {
                            report.add_issue(crate::types::IssueLevel::Warning, "unsafe-operation".to_string(), "Use of Function constructor is potentially unsafe".to_string(), Some(*span));
                        }
                    }
                    check_unsafe_operations(callee, report);
                    for arg in args {
                        check_unsafe_operations(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_unsafe_operations(object, report);
                    if *computed {
                        check_unsafe_operations(property, report);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_unsafe_operations(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_unsafe_operations(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_unsafe_operations(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_unsafe_operations(test, report);
                    check_unsafe_operations(consequent, report);
                    check_unsafe_operations(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_unsafe_operations(e, report);
                    }
                }
                _ => {}
            }
        }

        fn check_unsafe_operations_in_stmt(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_unsafe_operations(expr, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_unsafe_operations(e, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_unsafe_operations(e, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_unsafe_operations(test, report);
                    check_unsafe_operations_in_stmt(consequent, report);
                    if let Some(alt) = alternate {
                        check_unsafe_operations_in_stmt(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_unsafe_operations(test, report);
                    check_unsafe_operations_in_stmt(body, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_unsafe_operations_in_stmt(i, report);
                    }
                    if let Some(t) = test {
                        check_unsafe_operations(t, report);
                    }
                    if let Some(u) = update {
                        check_unsafe_operations(u, report);
                    }
                    check_unsafe_operations_in_stmt(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_unsafe_operations_in_stmt(s, report);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            check_unsafe_operations_in_stmt(stmt, report);
        }
    }
}
