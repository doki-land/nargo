use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 性能瓶颈检测规则
pub struct PerformanceBottleneckRule;

impl Rule for PerformanceBottleneckRule {
    fn code(&self) -> String {
        "performance-bottleneck".to_string()
    }

    fn description(&self) -> String {
        "检查可能的性能瓶颈".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查嵌套循环
        fn check_nested_loops(stmt: &JsStmt, loop_depth: u32, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::For { body, span, .. } => {
                    let new_depth = loop_depth + 1;
                    if new_depth >= 3 {
                        report.add_issue(crate::types::IssueLevel::Warning, "performance-bottleneck".to_string(), "Deeply nested loops (3+ levels) may cause performance issues".to_string(), Some(*span));
                    }
                    check_nested_loops(body, new_depth, report);
                }
                JsStmt::While { body, span, .. } => {
                    let new_depth = loop_depth + 1;
                    if new_depth >= 3 {
                        report.add_issue(crate::types::IssueLevel::Warning, "performance-bottleneck".to_string(), "Deeply nested loops (3+ levels) may cause performance issues".to_string(), Some(*span));
                    }
                    check_nested_loops(body, new_depth, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_nested_loops(s, loop_depth, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_nested_loops(consequent, loop_depth, report);
                    if let Some(alt) = alternate {
                        check_nested_loops(alt, loop_depth, report);
                    }
                }
                _ => {}
            }
        }

        // 检查 DOM 操作
        fn check_dom_operations(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查频繁的 DOM 查询
                    if let JsExpr::Member { property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            if prop_name == "getElementById" || prop_name == "querySelector" || prop_name == "querySelectorAll" {
                                report.add_issue(crate::types::IssueLevel::Info, "performance-bottleneck".to_string(), "Frequent DOM queries may cause performance issues, consider caching results".to_string(), Some(*span));
                            }
                        }
                    }
                    check_dom_operations(callee, report);
                    for arg in args {
                        check_dom_operations(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_dom_operations(object, report);
                    if *computed {
                        check_dom_operations(property, report);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_dom_operations(left, report);
                    check_dom_operations(right, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_dom_operations(argument, report);
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_dom_operations(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_dom_operations(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_dom_operations(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_dom_operations(test, report);
                    check_dom_operations(consequent, report);
                    check_dom_operations(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_dom_operations(e, report);
                    }
                }
                _ => {}
            }
        }

        fn check_dom_operations_in_stmt(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_dom_operations(expr, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_dom_operations(e, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_dom_operations(e, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_dom_operations(test, report);
                    check_dom_operations_in_stmt(consequent, report);
                    if let Some(alt) = alternate {
                        check_dom_operations_in_stmt(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_dom_operations(test, report);
                    check_dom_operations_in_stmt(body, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_dom_operations_in_stmt(i, report);
                    }
                    if let Some(t) = test {
                        check_dom_operations(t, report);
                    }
                    if let Some(u) = update {
                        check_dom_operations(u, report);
                    }
                    check_dom_operations_in_stmt(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_dom_operations_in_stmt(s, report);
                    }
                }
                _ => {}
            }
        }

        // 检查大数组操作
        fn check_large_array_operations(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查可能的大数组操作
                    if let JsExpr::Member { property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            if prop_name == "map" || prop_name == "filter" || prop_name == "reduce" || prop_name == "forEach" {
                                report.add_issue(crate::types::IssueLevel::Info, "performance-bottleneck".to_string(), "Large array operations may cause performance issues, consider using more efficient methods".to_string(), Some(*span));
                            }
                        }
                    }
                    check_large_array_operations(callee, report);
                    for arg in args {
                        check_large_array_operations(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_large_array_operations(object, report);
                    if *computed {
                        check_large_array_operations(property, report);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_large_array_operations(left, report);
                    check_large_array_operations(right, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_large_array_operations(argument, report);
                }
                JsExpr::Array(elements, span, _) => {
                    if elements.len() > 100 {
                        report.add_issue(crate::types::IssueLevel::Warning, "performance-bottleneck".to_string(), "Large array literals may cause performance issues".to_string(), Some(*span));
                    }
                    for el in elements {
                        check_large_array_operations(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_large_array_operations(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_large_array_operations(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_large_array_operations(test, report);
                    check_large_array_operations(consequent, report);
                    check_large_array_operations(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_large_array_operations(e, report);
                    }
                }
                _ => {}
            }
        }

        fn check_large_array_operations_in_stmt(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_large_array_operations(expr, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_large_array_operations(e, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_large_array_operations(e, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_large_array_operations(test, report);
                    check_large_array_operations_in_stmt(consequent, report);
                    if let Some(alt) = alternate {
                        check_large_array_operations_in_stmt(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_large_array_operations(test, report);
                    check_large_array_operations_in_stmt(body, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_large_array_operations_in_stmt(i, report);
                    }
                    if let Some(t) = test {
                        check_large_array_operations(t, report);
                    }
                    if let Some(u) = update {
                        check_large_array_operations(u, report);
                    }
                    check_large_array_operations_in_stmt(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_large_array_operations_in_stmt(s, report);
                    }
                }
                _ => {}
            }
        }

        // 执行检查
        for stmt in &program.body {
            check_nested_loops(stmt, 0, report);
            check_dom_operations_in_stmt(stmt, report);
            check_large_array_operations_in_stmt(stmt, report);
        }
    }
}
