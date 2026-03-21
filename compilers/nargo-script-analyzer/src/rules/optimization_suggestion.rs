use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 代码优化建议规则
pub struct OptimizationSuggestionRule;

impl Rule for OptimizationSuggestionRule {
    fn code(&self) -> String {
        "optimization-suggestion".to_string()
    }

    fn description(&self) -> String {
        "提供代码优化建议".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查重复计算
        fn check_redundant_computations(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::VariableDecl { id, init, span, .. } => {
                    // 检查复杂表达式的重复计算
                    if let Some(init_expr) = init {
                        if is_complex_expression(init_expr) {
                            report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), format!("Consider caching the result of this complex expression in variable '{}'", id), Some(*span));
                        }
                    }
                }
                JsStmt::Expr(expr, span, _) => {
                    // 检查复杂表达式的直接使用
                    if is_complex_expression(expr) {
                        report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Consider caching the result of this complex expression in a variable".to_string(), Some(*span));
                    }
                }
                JsStmt::Return(expr, span, _) => {
                    if let Some(expr) = expr {
                        if is_complex_expression(expr) {
                            report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Consider caching the result of this complex expression before returning".to_string(), Some(*span));
                        }
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_redundant_computations(s, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    if is_complex_expression(test) {
                        report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Consider caching the result of this complex condition in a variable".to_string(), Some(test.span()));
                    }
                    check_redundant_computations(consequent, report);
                    if let Some(alt) = alternate {
                        check_redundant_computations(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    if is_complex_expression(test) {
                        report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Consider caching the result of this complex condition outside the loop".to_string(), Some(test.span()));
                    }
                    check_redundant_computations(body, report);
                }
                JsStmt::For { test, body, .. } => {
                    if let Some(test) = test {
                        if is_complex_expression(test) {
                            report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Consider caching the result of this complex condition outside the loop".to_string(), Some(test.span()));
                        }
                    }
                    check_redundant_computations(body, report);
                }
                _ => {}
            }
        }

        // 检查是否为复杂表达式
        fn is_complex_expression(expr: &JsExpr) -> bool {
            match expr {
                JsExpr::Binary { left, right, .. } => is_complex_expression(left) || is_complex_expression(right),
                JsExpr::Call { callee, args, .. } => {
                    // 检查函数调用的复杂度
                    args.len() > 3 || is_complex_expression(callee)
                }
                JsExpr::Member { object, property, computed, .. } => is_complex_expression(object) || (*computed && is_complex_expression(property)),
                JsExpr::Array(elements, _, _) => elements.len() > 5 || elements.iter().any(is_complex_expression),
                JsExpr::Object(properties, _, _) => properties.len() > 5 || properties.values().any(is_complex_expression),
                JsExpr::Conditional { test, consequent, alternate, .. } => is_complex_expression(test) || is_complex_expression(consequent) || is_complex_expression(alternate),
                JsExpr::TemplateLiteral { expressions, .. } => expressions.len() > 3 || expressions.iter().any(is_complex_expression),
                _ => false,
            }
        }

        // 检查内存优化机会
        fn check_memory_optimization(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::VariableDecl { id, init, span, .. } => {
                    // 检查大型对象或数组的创建
                    if let Some(init_expr) = init {
                        match init_expr {
                            JsExpr::Array(elements, _, _) => {
                                if elements.len() > 100 {
                                    report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), format!("Large array created in variable '{}', consider lazy initialization or chunking", id), Some(*span));
                                }
                            }
                            JsExpr::Object(properties, _, _) => {
                                if properties.len() > 50 {
                                    report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), format!("Large object created in variable '{}', consider breaking it into smaller objects", id), Some(*span));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                JsStmt::Expr(expr, span, _) => {
                    // 检查大型对象或数组的直接使用
                    match expr {
                        JsExpr::Array(elements, _, _) => {
                            if elements.len() > 100 {
                                report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Large array created inline, consider moving it to a variable or using lazy initialization".to_string(), Some(*span));
                            }
                        }
                        JsExpr::Object(properties, _, _) => {
                            if properties.len() > 50 {
                                report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Large object created inline, consider moving it to a variable".to_string(), Some(*span));
                            }
                        }
                        _ => {}
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_memory_optimization(s, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_memory_optimization(consequent, report);
                    if let Some(alt) = alternate {
                        check_memory_optimization(alt, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_memory_optimization(body, report);
                }
                JsStmt::For { body, .. } => {
                    check_memory_optimization(body, report);
                }
                _ => {}
            }
        }

        // 检查循环优化机会
        fn check_loop_optimization(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::For { body, .. } => {
                    // 检查循环中的重复计算
                    check_loop_body_optimization(body, report);
                }
                JsStmt::While { body, .. } => {
                    // 检查循环中的重复计算
                    check_loop_body_optimization(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_loop_optimization(s, report);
                    }
                }
                _ => {}
            }
        }

        // 检查循环体中的优化机会
        fn check_loop_body_optimization(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_loop_body_optimization(s, report);
                    }
                }
                JsStmt::Expr(expr, span, _) => {
                    // 检查循环中的 DOM 操作
                    if is_dom_operation(expr) {
                        report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "DOM operation inside loop, consider moving it outside or batching".to_string(), Some(*span));
                    }
                    // 检查循环中的复杂计算
                    if is_complex_expression(expr) {
                        report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Complex computation inside loop, consider moving it outside".to_string(), Some(*span));
                    }
                }
                JsStmt::VariableDecl { init, span, .. } => {
                    if let Some(init_expr) = init {
                        if is_complex_expression(init_expr) {
                            report.add_issue(crate::types::IssueLevel::Info, "optimization-suggestion".to_string(), "Complex initialization inside loop, consider moving it outside".to_string(), Some(*span));
                        }
                    }
                }
                _ => {}
            }
        }

        // 检查是否为 DOM 操作
        fn is_dom_operation(expr: &JsExpr) -> bool {
            match expr {
                JsExpr::Call { callee, .. } => {
                    if let JsExpr::Member { property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            return ["getElementById", "querySelector", "querySelectorAll", "createElement", "appendChild", "removeChild", "innerHTML", "outerHTML"].contains(&prop_name.as_str());
                        }
                    }
                    is_dom_operation(callee)
                }
                JsExpr::Member { object, property, computed, .. } => is_dom_operation(object) || (*computed && is_dom_operation(property)),
                _ => false,
            }
        }

        // 执行优化建议检查
        for stmt in &program.body {
            check_redundant_computations(stmt, report);
            check_memory_optimization(stmt, report);
            check_loop_optimization(stmt, report);
        }
    }
}
