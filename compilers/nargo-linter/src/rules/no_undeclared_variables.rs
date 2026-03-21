//! No undeclared variables rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;
use std::collections::HashSet;

use nargo_ir::{IRModule, JsExpr, JsStmt};

/// No undeclared variables rule.
pub struct NoUndeclaredVariables;

#[async_trait]
impl LintRule for NoUndeclaredVariables {
    fn name(&self) -> &'static str {
        "no-undeclared-variables"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(script) = &module.script {
            let mut declared_vars = HashSet::new();
            self.collect_declared_variables(script, &mut declared_vars);
            self.check_undeclared_variables(script, &declared_vars, &mut diagnostics);
        }
        Ok(diagnostics)
    }

    async fn fix(&self, _source: &str, _module: &IRModule) -> Result<Vec<FixAction>> {
        Ok(Vec::new())
    }
}

impl NoUndeclaredVariables {
    fn collect_declared_variables(&self, program: &nargo_ir::JsProgram, declared_vars: &mut HashSet<String>) {
        for stmt in &program.body {
            self.collect_declared_variables_in_stmt(stmt, declared_vars);
        }
    }

    fn collect_declared_variables_in_stmt(&self, stmt: &JsStmt, declared_vars: &mut HashSet<String>) {
        match stmt {
            JsStmt::VariableDecl { id, .. } => {
                declared_vars.insert(id.clone());
            }
            JsStmt::FunctionDecl { id, params, body, .. } => {
                declared_vars.insert(id.clone());
                for param in params {
                    declared_vars.insert(param.clone());
                }
                for stmt in body {
                    self.collect_declared_variables_in_stmt(stmt, declared_vars);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.collect_declared_variables_in_stmt(stmt, declared_vars);
                }
            }
            _ => {}
        }
    }

    fn check_undeclared_variables(&self, program: &nargo_ir::JsProgram, declared_vars: &HashSet<String>, diagnostics: &mut Vec<Diagnostic>) {
        for stmt in &program.body {
            self.check_undeclared_variables_in_stmt(stmt, declared_vars, diagnostics);
        }
    }

    fn check_undeclared_variables_in_stmt(&self, stmt: &JsStmt, declared_vars: &HashSet<String>, diagnostics: &mut Vec<Diagnostic>) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.check_undeclared_variables_in_expr(expr, declared_vars, diagnostics),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.check_undeclared_variables_in_expr(expr, declared_vars, diagnostics);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    self.check_undeclared_variables_in_stmt(stmt, declared_vars, diagnostics);
                }
            }
            JsStmt::Block(stmts, _, _) => {
                for stmt in stmts {
                    self.check_undeclared_variables_in_stmt(stmt, declared_vars, diagnostics);
                }
            }
            _ => {}
        }
    }

    fn check_undeclared_variables_in_expr(&self, expr: &JsExpr, declared_vars: &HashSet<String>, diagnostics: &mut Vec<Diagnostic>) {
        match expr {
            JsExpr::Identifier(name, _, _) => {
                if !declared_vars.contains(name) && !self.is_global_variable(name) {
                    diagnostics.push(Diagnostic { code: self.name().to_string(), message: format!("Undeclared variable '{}'", name), severity: Severity::Error, line: 1, column: 1 });
                }
            }
            JsExpr::Call { callee, args, .. } => {
                self.check_undeclared_variables_in_expr(callee, declared_vars, diagnostics);
                for arg in args {
                    self.check_undeclared_variables_in_expr(arg, declared_vars, diagnostics);
                }
            }
            JsExpr::Member { object, property, computed, .. } => {
                self.check_undeclared_variables_in_expr(object, declared_vars, diagnostics);
                if *computed {
                    self.check_undeclared_variables_in_expr(property, declared_vars, diagnostics);
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.check_undeclared_variables_in_expr(el, declared_vars, diagnostics);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.check_undeclared_variables_in_expr(val, declared_vars, diagnostics);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.check_undeclared_variables_in_expr(left, declared_vars, diagnostics);
                self.check_undeclared_variables_in_expr(right, declared_vars, diagnostics);
            }
            JsExpr::Unary { argument, .. } => {
                self.check_undeclared_variables_in_expr(argument, declared_vars, diagnostics);
            }
            JsExpr::ArrowFunction { params, body, .. } => {
                let mut local_vars = declared_vars.clone();
                for param in params {
                    local_vars.insert(param.clone());
                }
                self.check_undeclared_variables_in_expr(body, &local_vars, diagnostics);
            }
            _ => {}
        }
    }

    fn is_global_variable(&self, name: &str) -> bool {
        // Common global variables
        matches!(name, "window" | "document" | "console" | "alert" | "confirm" | "prompt" | "navigator" | "location")
    }
}
