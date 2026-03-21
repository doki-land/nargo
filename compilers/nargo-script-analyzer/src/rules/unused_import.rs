use nargo_ir::{JsExpr, JsStmt};
use std::collections::HashSet;

use crate::{
    rule::Rule,
    types::{AnalysisReport, ScriptMetadata},
};

/// 未使用的导入检查规则
pub struct UnusedImportRule;

impl Rule for UnusedImportRule {
    fn code(&self) -> String {
        "unused-import".to_string()
    }

    fn description(&self) -> String {
        "检查未使用的导入".to_string()
    }

    fn check(&self, program: &nargo_ir::JsProgram, _meta: &ScriptMetadata, report: &mut AnalysisReport) {
        let mut imports = HashSet::new();
        let mut used_imports = HashSet::new();

        // 收集所有导入
        for stmt in &program.body {
            match stmt {
                JsStmt::Import { specifiers, source: _, span: _, trivia: _ } => {
                    for specifier in specifiers {
                        imports.insert(specifier.clone());
                    }
                }
                _ => {}
            }
        }

        // 收集所有使用的标识符
        fn collect_used_identifiers(expr: &JsExpr, used: &mut HashSet<String>) {
            match expr {
                JsExpr::Identifier(name, _, _) => {
                    used.insert(name.clone());
                }
                JsExpr::Binary { left, right, .. } => {
                    collect_used_identifiers(left, used);
                    collect_used_identifiers(right, used);
                }
                JsExpr::Unary { argument, .. } => {
                    collect_used_identifiers(argument, used);
                }
                JsExpr::Call { callee, args, .. } => {
                    collect_used_identifiers(callee, used);
                    for arg in args {
                        collect_used_identifiers(arg, used);
                    }
                }
                JsExpr::Member { object, property, computed, .. } => {
                    collect_used_identifiers(object, used);
                    if *computed {
                        collect_used_identifiers(property, used);
                    }
                }
                JsExpr::Array(elements, _, _) => {
                    for el in elements {
                        collect_used_identifiers(el, used);
                    }
                }
                JsExpr::Object(properties, _, _) => {
                    for value in properties.values() {
                        collect_used_identifiers(value, used);
                    }
                }
                JsExpr::ArrowFunction { body, .. } => {
                    collect_used_identifiers(body, used);
                }
                JsExpr::Conditional { test, consequent, alternate, .. } => {
                    collect_used_identifiers(test, used);
                    collect_used_identifiers(consequent, used);
                    collect_used_identifiers(alternate, used);
                }
                JsExpr::TemplateLiteral { expressions, .. } => {
                    for e in expressions {
                        collect_used_identifiers(e, used);
                    }
                }
                _ => {}
            }
        }

        fn collect_used_identifiers_in_stmt(stmt: &JsStmt, used: &mut HashSet<String>) {
            match stmt {
                JsStmt::Expr(expr, _, _) => collect_used_identifiers(expr, used),
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(e) = init {
                        collect_used_identifiers(e, used);
                    }
                }
                JsStmt::Return(expr, _, _) => {
                    if let Some(e) = expr {
                        collect_used_identifiers(e, used);
                    }
                }
                JsStmt::If { test, consequent, alternate, .. } => {
                    collect_used_identifiers(test, used);
                    collect_used_identifiers_in_stmt(consequent, used);
                    if let Some(alt) = alternate {
                        collect_used_identifiers_in_stmt(alt, used);
                    }
                }
                JsStmt::While { test, body, .. } => {
                    collect_used_identifiers(test, used);
                    collect_used_identifiers_in_stmt(body, used);
                }
                JsStmt::For { init, test, update, body, .. } => {
                    if let Some(i) = init {
                        collect_used_identifiers_in_stmt(i, used);
                    }
                    if let Some(t) = test {
                        collect_used_identifiers(t, used);
                    }
                    if let Some(u) = update {
                        collect_used_identifiers(u, used);
                    }
                    collect_used_identifiers_in_stmt(body, used);
                }
                JsStmt::Block(stmts, _, _) => {
                    for s in stmts {
                        collect_used_identifiers_in_stmt(s, used);
                    }
                }
                _ => {}
            }
        }

        for stmt in &program.body {
            collect_used_identifiers_in_stmt(stmt, &mut used_imports);
        }

        // 检查未使用的导入
        for import in &imports {
            if !used_imports.contains(import) {
                report.add_issue(crate::types::IssueLevel::Warning, self.code(), format!("Unused import: {}", import), None);
            }
        }
    }
}
