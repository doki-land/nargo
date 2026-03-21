//! No magic numbers rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsExpr, JsStmt};
use nargo_types::NargoValue;

/// No magic numbers rule.
pub struct NoMagicNumbers;

#[async_trait]
impl LintRule for NoMagicNumbers {
    fn name(&self) -> &'static str {
        "no-magic-numbers"
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

impl NoMagicNumbers {
    fn check_stmt(&self, stmt: &JsStmt, diagnostics: &mut Vec<Diagnostic>) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.check_expr(expr, diagnostics),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.check_expr(expr, diagnostics);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.check_stmt(declaration, diagnostics);
            }
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
            JsExpr::Literal(lit, _, _) => {
                if let NargoValue::Number(_) = lit {
                    diagnostics.push(Diagnostic { code: self.name().to_string(), message: "Magic number detected".to_string(), severity: Severity::Warning, line: 1, column: 1 });
                }
            }
            JsExpr::Call { callee, args, .. } => {
                self.check_expr(callee, diagnostics);
                for arg in args {
                    self.check_expr(arg, diagnostics);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.check_expr(object, diagnostics);
                if *computed {
                    self.check_expr(property, diagnostics);
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
            JsStmt::Expr(expr, _, _) => {
                self.fix_expr(expr, source, fix_actions);
            }
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.fix_expr(expr, source, fix_actions);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.fix_stmt(declaration, source, fix_actions);
            }
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

    fn fix_expr(&self, expr: &JsExpr, source: &str, fix_actions: &mut Vec<FixAction>) {
        match expr {
            JsExpr::Literal(lit, span, _) => {
                if let NargoValue::Number(num) = lit {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        // Generate a constant name based on the number
                        let constant_name = format!("CONSTANT_{}", num.to_string().replace('.', "_")).to_uppercase();
                        let constant_name_clone = constant_name.clone();
                        fix_actions.push(FixAction { start, end, replacement: constant_name, description: format!("Replace magic number {} with constant {}", num, constant_name_clone) });
                    }
                }
            }
            JsExpr::Call { callee, args, .. } => {
                self.fix_expr(callee, source, fix_actions);
                for arg in args {
                    self.fix_expr(arg, source, fix_actions);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.fix_expr(object, source, fix_actions);
                if *computed {
                    self.fix_expr(property, source, fix_actions);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.fix_expr(el, source, fix_actions);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.fix_expr(val, source, fix_actions);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.fix_expr(left, source, fix_actions);
                self.fix_expr(right, source, fix_actions);
            }
            JsExpr::Unary { argument, .. } => {
                self.fix_expr(argument, source, fix_actions);
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.fix_expr(body, source, fix_actions);
            }
            _ => {}
        }
    }
}
