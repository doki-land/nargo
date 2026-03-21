//! No alert rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsExpr, JsStmt};

/// No alert rule.
pub struct NoAlert;

#[async_trait]
impl LintRule for NoAlert {
    fn name(&self) -> &'static str {
        "no-alert"
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

impl NoAlert {
    fn check_stmt(&self, stmt: &JsStmt, diagnostics: &mut Vec<Diagnostic>) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.check_expr(expr, diagnostics),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.check_expr(expr, diagnostics);
                }
            }
            JsStmt::Export { declaration, .. } => self.check_stmt(declaration, diagnostics),
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.check_stmt(stmt, diagnostics);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.check_stmt(stmt, diagnostics);
                }
            }
            _ => {}
        }
    }

    fn check_expr(&self, expr: &JsExpr, diagnostics: &mut Vec<Diagnostic>) {
        match expr {
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(name, _, _) = callee.as_ref() {
                    if name == "alert" {
                        diagnostics.push(Diagnostic { code: self.name().to_string(), message: "Unexpected 'alert' call".to_string(), severity: Severity::Warning, line: 1, column: 1 });
                    }
                }
                for arg in args {
                    self.check_expr(arg, diagnostics);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.check_expr(el, diagnostics);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.check_expr(val, diagnostics);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.check_expr(left, diagnostics);
                self.check_expr(right, diagnostics);
            }
            JsExpr::Unary { argument, .. } => {
                self.check_expr(argument, diagnostics);
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.check_expr(body, diagnostics);
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
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    if let Some(action) = self.fix_expr(expr, source) {
                        fix_actions.push(action);
                    }
                }
            }
            JsStmt::Export { declaration, .. } => self.fix_stmt(declaration, source, fix_actions),
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.fix_stmt(stmt, source, fix_actions);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.fix_stmt(stmt, source, fix_actions);
                }
            }
            _ => {}
        }
    }

    fn fix_expr(&self, expr: &JsExpr, source: &str) -> Option<FixAction> {
        match expr {
            JsExpr::Call { callee, args, span, .. } => {
                if let JsExpr::Identifier(name, _, _) = callee.as_ref() {
                    if name == "alert" {
                        let start = span.start.offset as usize;
                        let end = span.end.offset as usize;
                        if start < end && end <= source.len() {
                            return Some(FixAction { start, end, replacement: "".to_string(), description: "Remove alert call".to_string() });
                        }
                    }
                }
                for arg in args {
                    if let Some(action) = self.fix_expr(arg, source) {
                        return Some(action);
                    }
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    if let Some(action) = self.fix_expr(el, source) {
                        return Some(action);
                    }
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    if let Some(action) = self.fix_expr(val, source) {
                        return Some(action);
                    }
                }
            }
            JsExpr::Binary { left, right, .. } => {
                if let Some(action) = self.fix_expr(left, source) {
                    return Some(action);
                }
                if let Some(action) = self.fix_expr(right, source) {
                    return Some(action);
                }
            }
            JsExpr::Unary { argument, .. } => {
                if let Some(action) = self.fix_expr(argument, source) {
                    return Some(action);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                if let Some(action) = self.fix_expr(body, source) {
                    return Some(action);
                }
            }
            _ => {}
        }
        None
    }
}
