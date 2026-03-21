use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 代码复杂度检查规则
pub struct CodeComplexityRule;

impl Rule for CodeComplexityRule {
    fn code(&self) -> String {
        "code-complexity".to_string()
    }

    fn description(&self) -> String {
        "检查代码复杂度问题".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查圈复杂度
        fn check_cyclomatic_complexity(stmt: &JsStmt, complexity: u32, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::FunctionDecl { id, body, span, .. } => {
                    // 创建一个临时的 Block 语句来计算复杂度
                    let block_stmt = JsStmt::Block(body.clone(), *span, Default::default());
                    let func_complexity = calculate_complexity(&block_stmt, 1);
                    if func_complexity > 10 {
                        report.add_issue(crate::types::IssueLevel::Warning, "code-complexity".to_string(), format!("Function '{}' has high cyclomatic complexity ({}), consider refactoring", id, func_complexity), Some(*span));
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_cyclomatic_complexity(s, complexity, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_cyclomatic_complexity(consequent, complexity + 1, report);
                    if let Some(alt) = alternate {
                        check_cyclomatic_complexity(alt, complexity + 1, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_cyclomatic_complexity(body, complexity + 1, report);
                }
                JsStmt::For { body, .. } => {
                    check_cyclomatic_complexity(body, complexity + 1, report);
                }
                _ => {}
            }
        }

        // 计算代码复杂度
        fn calculate_complexity(stmt: &JsStmt, base: u32) -> u32 {
            match stmt {
                JsStmt::If { consequent, alternate, .. } => {
                    let cons_complexity = calculate_complexity(consequent, base + 1);
                    let alt_complexity = if let Some(alt) = alternate { calculate_complexity(alt, base + 1) } else { base };
                    cons_complexity.max(alt_complexity)
                }
                JsStmt::While { body, .. } => calculate_complexity(body, base + 1),
                JsStmt::For { body, .. } => calculate_complexity(body, base + 1),
                JsStmt::Block(stmts, _, _) => stmts.iter().fold(base, |max_complexity, s| max_complexity.max(calculate_complexity(s, base))),
                _ => base,
            }
        }

        // 检查嵌套深度
        fn check_nesting_depth(stmt: &JsStmt, depth: u32, report: &mut AnalysisReport) {
            const MAX_NESTING_DEPTH: u32 = 5;

            if depth > MAX_NESTING_DEPTH {
                if let Some(span) = get_stmt_span(stmt) {
                    report.add_issue(crate::types::IssueLevel::Warning, "code-complexity".to_string(), format!("Code has excessive nesting depth ({}), consider refactoring", depth), Some(span));
                }
            }

            match stmt {
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_nesting_depth(s, depth + 1, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_nesting_depth(consequent, depth + 1, report);
                    if let Some(alt) = alternate {
                        check_nesting_depth(alt, depth + 1, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_nesting_depth(body, depth + 1, report);
                }
                JsStmt::For { body, .. } => {
                    check_nesting_depth(body, depth + 1, report);
                }
                _ => {}
            }
        }

        // 获取语句的 span
        fn get_stmt_span(stmt: &JsStmt) -> Option<nargo_types::Span> {
            match stmt {
                JsStmt::VariableDecl { span, .. } => Some(*span),
                JsStmt::FunctionDecl { span, .. } => Some(*span),
                JsStmt::Expr(_, span, _) => Some(*span),
                JsStmt::Return(_, span, _) => Some(*span),
                JsStmt::If { span, .. } => Some(*span),
                JsStmt::While { span, .. } => Some(*span),
                JsStmt::For { span, .. } => Some(*span),
                JsStmt::Block(_, span, _) => Some(*span),
                _ => None,
            }
        }

        // 执行复杂度检查
        for stmt in &program.body {
            check_cyclomatic_complexity(stmt, 1, report);
            check_nesting_depth(stmt, 0, report);
        }
    }
}
