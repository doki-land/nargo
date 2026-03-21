use nargo_ir::{JsExpr, JsStmt};

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 安全规则
pub struct SecurityRule;

impl Rule for SecurityRule {
    fn code(&self) -> String {
        "security".to_string()
    }

    fn description(&self) -> String {
        "检查安全问题".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 检查 SQL 注入
        fn check_sql_injection(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Binary { left, right, op, span, .. } => {
                    // 检查字符串拼接可能导致的 SQL 注入
                    if *op == "+" {
                        // 检查是否有 SQL 相关的关键词
                        let mut has_sql_keyword = false;
                        let mut has_user_input = false;

                        fn check_sql_keywords(expr: &JsExpr) -> bool {
                            match expr {
                                JsExpr::Literal(nargo_types::NargoValue::String(s), _, _) => {
                                    let s_lower = s.to_lowercase();
                                    s_lower.contains("select") || s_lower.contains("insert") || s_lower.contains("update") || s_lower.contains("delete") || s_lower.contains("from") || s_lower.contains("where")
                                }
                                JsExpr::Binary { left, right, .. } => check_sql_keywords(left) || check_sql_keywords(right),
                                JsExpr::Identifier(_, _, _) => true, // 可能是用户输入变量
                                _ => false,
                            }
                        }

                        has_sql_keyword = check_sql_keywords(left) || check_sql_keywords(right);
                        has_user_input = matches!(&**left, JsExpr::Identifier(_, _, _)) || matches!(&**right, JsExpr::Identifier(_, _, _));

                        if has_sql_keyword && has_user_input {
                            report.add_issue(crate::types::IssueLevel::Error, "security".to_string(), "Potential SQL injection vulnerability: avoid string concatenation for SQL queries".to_string(), Some(*span));
                        }
                    }
                    check_sql_injection(left, report);
                    check_sql_injection(right, report);
                }
                JsExpr::Call { callee, args, .. } => {
                    check_sql_injection(callee, report);
                    for arg in args {
                        check_sql_injection(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_sql_injection(object, report);
                    if *computed {
                        check_sql_injection(property, report);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_sql_injection(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_sql_injection(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_sql_injection(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_sql_injection(test, report);
                    check_sql_injection(consequent, report);
                    check_sql_injection(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_sql_injection(e, report);
                    }
                }
                _ => {}
            }
        }

        // 检查 XSS 攻击
        fn check_xss(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Member { object, property, computed: false, span, .. } => {
                    if let JsExpr::Identifier(prop_name, _, _) = &**property {
                        // 检查 innerHTML 赋值
                        if prop_name == "innerHTML" {
                            report.add_issue(crate::types::IssueLevel::Warning, "security".to_string(), "Potential XSS vulnerability: avoid using innerHTML with untrusted data".to_string(), Some(*span));
                        }
                    }
                    check_xss(object, report);
                }
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查 document.write
                    if let JsExpr::Member { property, computed: false, .. } = &**callee {
                        if let JsExpr::Identifier(prop_name, _, _) = &**property {
                            if prop_name == "write" {
                                report.add_issue(crate::types::IssueLevel::Warning, "security".to_string(), "Potential XSS vulnerability: avoid using document.write".to_string(), Some(*span));
                            }
                        }
                    }
                    check_xss(callee, report);
                    for arg in args {
                        check_xss(arg, report);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_xss(left, report);
                    check_xss(right, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_xss(argument, report);
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_xss(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_xss(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_xss(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_xss(test, report);
                    check_xss(consequent, report);
                    check_xss(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_xss(e, report);
                    }
                }
                _ => {}
            }
        }

        // 检查不安全的密码存储
        fn check_password_storage(expr: &JsExpr, report: &mut AnalysisReport) {
            match expr {
                JsExpr::Call { callee, args, span, .. } => {
                    // 检查密码相关的存储
                    if let JsExpr::Identifier(name, _, _) = &**callee {
                        if name == "localStorage" || name == "sessionStorage" {
                            // 检查是否存储密码
                            for arg in args {
                                if let JsExpr::Literal(nargo_types::NargoValue::String(s), _, _) = arg {
                                    let s_lower = s.to_lowercase();
                                    if s_lower.contains("password") || s_lower.contains("pwd") {
                                        report.add_issue(crate::types::IssueLevel::Error, "security".to_string(), "Insecure password storage: avoid storing passwords in localStorage/sessionStorage".to_string(), Some(*span));
                                    }
                                }
                            }
                        }
                    }
                    check_password_storage(callee, report);
                    for arg in args {
                        check_password_storage(arg, report);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    check_password_storage(object, report);
                    if *computed {
                        check_password_storage(property, report);
                    }
                }
                JsExpr::Binary { left, right, .. } => {
                    check_password_storage(left, report);
                    check_password_storage(right, report);
                }
                JsExpr::Unary { argument, .. } => {
                    check_password_storage(argument, report);
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        check_password_storage(el, report);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        check_password_storage(value, report);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    check_password_storage(body, report);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    check_password_storage(test, report);
                    check_password_storage(consequent, report);
                    check_password_storage(alternate, report);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        check_password_storage(e, report);
                    }
                }
                _ => {}
            }
        }

        fn check_security_in_stmt(stmt: &JsStmt, report: &mut AnalysisReport) {
            match stmt {
                JsStmt::Expr(expr, _, _) => {
                    check_sql_injection(expr, report);
                    check_xss(expr, report);
                    check_password_storage(expr, report);
                }
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        check_sql_injection(e, report);
                        check_xss(e, report);
                        check_password_storage(e, report);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        check_sql_injection(e, report);
                        check_xss(e, report);
                        check_password_storage(e, report);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    check_sql_injection(test, report);
                    check_xss(test, report);
                    check_password_storage(test, report);
                    check_security_in_stmt(consequent, report);
                    if let Some(alt) = alternate {
                        check_security_in_stmt(alt, report);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    check_sql_injection(test, report);
                    check_xss(test, report);
                    check_password_storage(test, report);
                    check_security_in_stmt(body, report);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        check_security_in_stmt(i, report);
                    }
                    if let Some(t) = test {
                        check_sql_injection(t, report);
                        check_xss(t, report);
                        check_password_storage(t, report);
                    }
                    if let Some(u) = update {
                        check_sql_injection(u, report);
                        check_xss(u, report);
                        check_password_storage(u, report);
                    }
                    check_security_in_stmt(body, report);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        check_security_in_stmt(s, report);
                    }
                }
                _ => {}
            }
        }

        // 执行安全检查
        for stmt in &program.body {
            check_security_in_stmt(stmt, report);
        }
    }
}
