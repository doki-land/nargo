use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 代码重构建议规则
pub struct RefactoringSuggestionRule;

impl Rule for RefactoringSuggestionRule {
    fn code(&self) -> String {
        "refactoring-suggestion".to_string()
    }

    fn description(&self) -> String {
        "提供代码重构建议".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查过长的函数
        fn check_long_functions(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::FunctionDecl { id, body, span, .. } => {
                    let body_length = body.len();
                    if body_length > 30 {
                        report.add_issue(crate::types::IssueLevel::Info, "refactoring-suggestion".to_string(), format!("Function '{}' is too long ({} statements), consider refactoring into smaller functions", id, body_length), Some(*span));
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_long_functions(s, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_long_functions(consequent, report);
                    if let Some(alt) = alternate {
                        check_long_functions(alt, report);
                    }
                }
                _ => {}
            }
        }

        // 检查重复代码
        fn check_duplicate_code(stmt: &JsStmt, report: &mut AnalysisReport) {
            // 这里可以实现更复杂的重复代码检测逻辑
            // 目前我们检查简单的重复模式
            match stmt {
                JsStmt::Block(stmts, span, _) => {
                    let mut previous_stmt: Option<&JsStmt> = None;
                    for (_i, s) in stmts.iter().enumerate() {
                        if let Some(prev) = previous_stmt {
                            if are_statements_similar(prev, s) {
                                report.add_issue(crate::types::IssueLevel::Info, "refactoring-suggestion".to_string(), "Duplicate code detected, consider refactoring into a function".to_string(), Some(*span));
                                break;
                            }
                        }
                        previous_stmt = Some(s);
                    }
                    for s in stmts {
                        check_duplicate_code(s, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_duplicate_code(consequent, report);
                    if let Some(alt) = alternate {
                        check_duplicate_code(alt, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_duplicate_code(body, report);
                }
                JsStmt::For { body, .. } => {
                    check_duplicate_code(body, report);
                }
                _ => {}
            }
        }

        // 检查两个语句是否相似
        fn are_statements_similar(stmt1: &JsStmt, stmt2: &JsStmt) -> bool {
            // 这里实现简单的相似性检查
            // 实际应用中可能需要更复杂的算法
            match (stmt1, stmt2) {
                (JsStmt::Expr(expr1, _, _), JsStmt::Expr(expr2, _, _)) => are_expressions_similar(expr1, expr2),
                (JsStmt::VariableDecl { id: id1, init: init1, .. }, JsStmt::VariableDecl { id: id2, init: init2, .. }) => id1 != id2 && are_option_expressions_similar(init1, init2),
                _ => false,
            }
        }

        // 检查两个表达式是否相似
        fn are_expressions_similar(expr1: &JsExpr, expr2: &JsExpr) -> bool {
            match (expr1, expr2) {
                (JsExpr::Call { callee: callee1, args: args1, .. }, JsExpr::Call { callee: callee2, args: args2, .. }) => args1.len() == args2.len() && are_expressions_similar(callee1, callee2),
                (JsExpr::Member { object: obj1, property: _prop1, computed: comp1, .. }, JsExpr::Member { object: obj2, property: _prop2, computed: comp2, .. }) => *comp1 == *comp2 && are_expressions_similar(obj1, obj2),
                _ => false,
            }
        }

        // 检查两个可选表达式是否相似
        fn are_option_expressions_similar(opt1: &Option<JsExpr>, opt2: &Option<JsExpr>) -> bool {
            match (opt1, opt2) {
                (Some(expr1), Some(expr2)) => are_expressions_similar(expr1, expr2),
                (None, None) => true,
                _ => false,
            }
        }

        // 检查嵌套的条件语句
        fn check_nested_conditions(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::If { test: _test, consequent, alternate, span, .. } => {
                    // 检查嵌套的 if 语句
                    if let JsStmt::If { .. } = **consequent {
                        report.add_issue(crate::types::IssueLevel::Info, "refactoring-suggestion".to_string(), "Nested if statements detected, consider using early return or combining conditions".to_string(), Some(*span));
                    }
                    check_nested_conditions(consequent, report);
                    if let Some(alt) = alternate {
                        check_nested_conditions(alt, report);
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_nested_conditions(s, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_nested_conditions(body, report);
                }
                JsStmt::For { body, .. } => {
                    check_nested_conditions(body, report);
                }
                _ => {}
            }
        }

        // 检查魔法数字
        fn check_magic_numbers(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Literal(nargo_types::NargoValue::Number(num), span, _) => {
                    // 检查是否为魔法数字（不是 0, 1, 2 等常见数字）
                    let num_f64 = *num;
                    let num_i64 = num_f64 as i64;
                    if num_f64 == num_i64 as f64 && ![-1, 0, 1, 2, 3, 4, 5, 10, 100].contains(&num_i64) {
                        report.add_issue(crate::types::IssueLevel::Info, "refactoring-suggestion".to_string(), format!("Magic number {} detected, consider defining it as a constant", num_f64), Some(*span));
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_magic_numbers(left, report);
                    check_magic_numbers(right, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_magic_numbers(argument, report);
                }
                JsExpr::Call { callee, args, .. } => {
                    check_magic_numbers(callee, report);
                    for arg in args {
                        check_magic_numbers(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_magic_numbers(object, report);
                    if *computed {
                        check_magic_numbers(property, report);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_magic_numbers(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_magic_numbers(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_magic_numbers(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_magic_numbers(test, report);
                    check_magic_numbers(consequent, report);
                    check_magic_numbers(alternate, report);
                }
                _ => {}
            }
        }

        // 检查语句中的魔法数字
        fn check_magic_numbers_in_stmt(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => check_magic_numbers(expr, report),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_magic_numbers(e, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_magic_numbers(e, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_magic_numbers(test, report);
                    check_magic_numbers_in_stmt(consequent, report);
                    if let Some(alt) = alternate {
                        check_magic_numbers_in_stmt(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_magic_numbers(test, report);
                    check_magic_numbers_in_stmt(body, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_magic_numbers_in_stmt(i, report);
                    }
                    if let Some(t) = test {
                        check_magic_numbers(t, report);
                    }
                    if let Some(u) = update {
                        check_magic_numbers(u, report);
                    }
                    check_magic_numbers_in_stmt(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_magic_numbers_in_stmt(s, report);
                    }
                }
                _ => {}
            }
        }

        // 检查函数参数过多
        fn check_excessive_parameters(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::FunctionDecl { id, params, span, .. } => {
                    if params.len() > 5 {
                        report.add_issue(crate::types::IssueLevel::Info, "refactoring-suggestion".to_string(), format!("Function '{}' has too many parameters ({}), consider using an options object", id, params.len()), Some(*span));
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_excessive_parameters(s, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_excessive_parameters(consequent, report);
                    if let Some(alt) = alternate {
                        check_excessive_parameters(alt, report);
                    }
                }
                _ => {}
            }
        }

        // 执行重构建议检查
        for stmt in &program.body {
            check_long_functions(stmt, report);
            check_duplicate_code(stmt, report);
            check_nested_conditions(stmt, report);
            check_magic_numbers_in_stmt(stmt, report);
            check_excessive_parameters(stmt, report);
        }
    }
}
