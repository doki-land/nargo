//! No debugger rule.

use crate::{Diagnostic, FixAction, LintRule};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsExpr, JsStmt};

/// No debugger rule.
pub struct NoDebugger;

#[async_trait]
impl LintRule for NoDebugger {
    fn name(&self) -> &'static str {
        "no-debugger"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(script) = &module.script {
            for stmt in &script.body {
                self.check_stmt(stmt, &mut diagnostics);
            }
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(script) = &module.script {
            for stmt in &script.body {
                self.fix_stmt(stmt, source, &mut fix_actions);
            }
        }
        Ok(fix_actions)
    }
}

impl NoDebugger {
    fn check_stmt(&self, stmt: &JsStmt, diagnostics: &mut Vec<Diagnostic>) {
        match stmt {
            JsStmt::Break(_, _) | JsStmt::Continue(_, _) => {
                // Handle break and continue statements
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.check_stmt(s, diagnostics);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.check_stmt(s, diagnostics);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.check_stmt(declaration, diagnostics);
            }
            JsStmt::Expr(expr, _, _) => {
                self.check_expr(expr, diagnostics);
            }
            JsStmt::Other(code, _, _) => {
                if code.contains("debugger") {
                    diagnostics.push(crate::Diagnostic { code: self.name().to_string(), message: "Unexpected 'debugger' statement".to_string(), severity: crate::Severity::Error, line: 1, column: 1 });
                }
            }
            _ => {}
        }
    }

    fn check_expr(&self, expr: &JsExpr, diagnostics: &mut Vec<Diagnostic>) {
        match expr {
            JsExpr::Identifier(name, _, _) => {
                if name == "debugger" {
                    diagnostics.push(crate::Diagnostic { code: self.name().to_string(), message: "Unexpected 'debugger' statement".to_string(), severity: crate::Severity::Error, line: 1, column: 1 });
                }
            }
            JsExpr::Other(code, _, _) => {
                if code.contains("debugger") {
                    diagnostics.push(crate::Diagnostic { code: self.name().to_string(), message: "Unexpected 'debugger' statement".to_string(), severity: crate::Severity::Error, line: 1, column: 1 });
                }
            }
            _ => {}
        }
    }

    fn fix_stmt(&self, stmt: &JsStmt, source: &str, fix_actions: &mut Vec<FixAction>) {
        match stmt {
            JsStmt::Expr(expr, span, _) => {
                if let Some(action) = self.fix_expr(expr, source) {
                    fix_actions.push(action);
                }
            }
            JsStmt::Other(code, span, _) => {
                if code.contains("debugger") {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        fix_actions.push(FixAction { start, end, replacement: "".to_string(), description: "Remove debugger statement".to_string() });
                    }
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.fix_stmt(s, source, fix_actions);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.fix_stmt(s, source, fix_actions);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.fix_stmt(declaration, source, fix_actions);
            }
            _ => {}
        }
    }

    fn fix_expr(&self, expr: &JsExpr, source: &str) -> Option<FixAction> {
        match expr {
            JsExpr::Identifier(name, span, _) => {
                if name == "debugger" {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        return Some(FixAction { start, end, replacement: "".to_string(), description: "Remove debugger statement".to_string() });
                    }
                }
            }
            JsExpr::Other(code, span, _) => {
                if code.contains("debugger") {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        return Some(FixAction { start, end, replacement: "".to_string(), description: "Remove debugger statement".to_string() });
                    }
                }
            }
            _ => {}
        }
        None
    }
}
