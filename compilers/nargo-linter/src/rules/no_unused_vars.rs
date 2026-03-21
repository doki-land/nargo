//! No unused variables rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsExpr, JsStmt};
use std::collections::HashSet;

/// No unused variables rule.
pub struct NoUnusedVars;

#[async_trait]
impl LintRule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(script) = &module.script {
            let mut declared_vars = HashSet::new();
            let mut used_vars = HashSet::new();

            // First pass: collect all declared variables
            for stmt in &script.body {
                self.collect_declared_vars(stmt, &mut declared_vars);
            }

            // Second pass: collect all used variables
            for stmt in &script.body {
                self.collect_used_vars(stmt, &mut used_vars);
            }

            // Find unused variables
            for var in declared_vars {
                if !used_vars.contains(&var) {
                    diagnostics.push(Diagnostic { code: self.name().to_string(), message: format!("Unused variable '{}'", var), severity: Severity::Warning, line: 1, column: 1 });
                }
            }
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(script) = &module.script {
            let mut declared_vars = HashSet::new();
            let mut used_vars = HashSet::new();

            // First pass: collect all declared variables
            for stmt in &script.body {
                self.collect_declared_vars(stmt, &mut declared_vars);
            }

            // Second pass: collect all used variables
            for stmt in &script.body {
                self.collect_used_vars(stmt, &mut used_vars);
            }

            // Find unused variables and generate fix actions
            for stmt in &script.body {
                self.fix_stmt(stmt, source, &declared_vars, &used_vars, &mut fix_actions);
            }
        }
        Ok(fix_actions)
    }
}
impl NoUnusedVars {
    fn collect_declared_vars(&self, stmt: &JsStmt, declared: &mut HashSet<String>) {
        match stmt {
            JsStmt::VariableDecl { id, .. } => {
                declared.insert(id.clone());
            }
            JsStmt::Export { declaration, .. } => {
                self.collect_declared_vars(declaration, declared);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.collect_declared_vars(stmt, declared);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.collect_declared_vars(stmt, declared);
                }
            }
            _ => {}
        }
    }

    fn collect_used_vars(&self, stmt: &JsStmt, used: &mut HashSet<String>) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.collect_used_vars_expr(expr, used),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.collect_used_vars_expr(expr, used);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.collect_used_vars(declaration, used);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.collect_used_vars(stmt, used);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.collect_used_vars(stmt, used);
                }
            }
            _ => {}
        }
    }

    fn collect_used_vars_expr(&self, expr: &JsExpr, used: &mut HashSet<String>) {
        match expr {
            JsExpr::Identifier(name, _, _) => {
                used.insert(name.clone());
            }
            JsExpr::Call { callee, args, .. } => {
                self.collect_used_vars_expr(callee, used);
                for arg in args {
                    self.collect_used_vars_expr(arg, used);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.collect_used_vars_expr(object, used);
                if *computed {
                    self.collect_used_vars_expr(property, used);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.collect_used_vars_expr(el, used);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.collect_used_vars_expr(val, used);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.collect_used_vars_expr(left, used);
                self.collect_used_vars_expr(right, used);
            }
            JsExpr::Unary { argument, .. } => {
                self.collect_used_vars_expr(argument, used);
            }
            JsExpr::ArrowFunction { body, .. } => {
                self.collect_used_vars_expr(body, used);
            }
            _ => {}
        }
    }

    fn fix_stmt(&self, stmt: &JsStmt, source: &str, declared_vars: &HashSet<String>, used_vars: &HashSet<String>, fix_actions: &mut Vec<FixAction>) {
        match stmt {
            JsStmt::VariableDecl { id, span, .. } => {
                if declared_vars.contains(id) && !used_vars.contains(id) {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        fix_actions.push(FixAction { start, end, replacement: "".to_string(), description: format!("Remove unused variable '{}'", id) });
                    }
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.fix_stmt(s, source, declared_vars, used_vars, fix_actions);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.fix_stmt(s, source, declared_vars, used_vars, fix_actions);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.fix_stmt(declaration, source, declared_vars, used_vars, fix_actions);
            }
            _ => {}
        }
    }
}
