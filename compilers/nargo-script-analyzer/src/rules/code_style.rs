use nargo_ir::JsStmt;

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 代码风格检查规则
pub struct CodeStyleRule;

impl Rule for CodeStyleRule {
    fn code(&self) -> String {
        "code-style".to_string()
    }

    fn description(&self) -> String {
        "检查代码风格问题".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查变量命名规范
        fn check_naming_conventions(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::VariableDecl { id, span, .. } => {
                    // 检查变量命名
                    if id.starts_with('[') && id.ends_with(']') {
                        let content = &id[1..id.len() - 1];
                        for part in content.split(',') {
                            let trimmed = part.trim();
                            if !trimmed.is_empty() {
                                check_variable_name(trimmed, *span, report);
                            }
                        }
                    }
                    else {
                        check_variable_name(id, *span, report);
                    }
                }
                JsStmt::FunctionDecl { id, span, .. } => {
                    // 检查函数命名
                    if !id.starts_with(|c: char| c.is_ascii_lowercase()) {
                        report.add_issue(crate::types::IssueLevel::Warning, "code-style".to_string(), format!("Function name '{}' should use camelCase", id), Some(*span));
                    }
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_naming_conventions(s, report);
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_naming_conventions(consequent, report);
                    if let Some(alt) = alternate {
                        check_naming_conventions(alt, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_naming_conventions(body, report);
                }
                JsStmt::For { body, .. } => {
                    check_naming_conventions(body, report);
                }
                _ => {}
            }
        }

        fn check_variable_name(name: &str, span: nargo_types::Span, report: &mut AnalysisReport) {
            // 检查变量命名
            if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                report.add_issue(crate::types::IssueLevel::Warning, "code-style".to_string(), format!("Variable name '{}' should use camelCase", name), Some(span));
            }
            // 检查单字符变量名（除了 i, j, k 等循环变量）
            if name.len() == 1 && !"ijklmn".contains(name) {
                report.add_issue(crate::types::IssueLevel::Info, "code-style".to_string(), format!("Variable name '{}' is too short, consider using a more descriptive name", name), Some(span));
            }
        }

        // 检查代码长度
        fn check_code_length(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::FunctionDecl { body, span, .. } => {
                    let body_length = body.len();
                    if body_length > 50 {
                        report.add_issue(crate::types::IssueLevel::Warning, "code-style".to_string(), "Function is too long (over 50 statements), consider refactoring".to_string(), Some(*span));
                    }
                }
                JsStmt::Block(stmts, span, ..) => {
                    let block_length = stmts.len();
                    if block_length > 30 {
                        report.add_issue(crate::types::IssueLevel::Info, "code-style".to_string(), "Block is too long (over 30 statements), consider refactoring".to_string(), Some(*span));
                    }
                }
                JsStmt::If { consequent, alternate, .. } => {
                    check_code_length(consequent, report);
                    if let Some(alt) = alternate {
                        check_code_length(alt, report);
                    }
                }
                JsStmt::While { body, .. } => {
                    check_code_length(body, report);
                }
                JsStmt::For { body, .. } => {
                    check_code_length(body, report);
                }
                _ => {}
            }
        }

        // 检查空格和缩进
        fn check_whitespace(_stmt: &JsStmt, _report: &mut AnalysisReport) {
            // 这里可以添加空格和缩进的检查逻辑
            // 由于 AST 中可能没有包含空格信息，这里暂时只做简单检查
        }

        // 执行代码风格检查
        for stmt in &program.body {
            check_naming_conventions(stmt, report);
            check_code_length(stmt, report);
            check_whitespace(stmt, report);
        }
    }
}
