//! No unused imports rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsStmt};
use std::collections::HashSet;

/// No unused imports rule.
pub struct NoUnusedImports;

#[async_trait]
impl LintRule for NoUnusedImports {
    fn name(&self) -> &'static str {
        "no-unused-imports"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(script) = &module.script {
            let mut imports = Vec::new();
            let mut used_vars = HashSet::new();

            // First pass: collect all imports
            for stmt in &script.body {
                if let JsStmt::Import { specifiers, .. } = stmt {
                    for specifier in specifiers {
                        imports.push((specifier.clone(), stmt));
                    }
                }
            }

            // Second pass: collect all used variables
            for stmt in &script.body {
                self.collect_used_vars(stmt, &mut used_vars);
            }

            // Find unused imports
            for (specifier, stmt) in &imports {
                if !used_vars.contains(specifier) {
                    let line = 1; // TODO: Get actual line number from span
                    let column = 1; // TODO: Get actual column number from span
                    diagnostics.push(Diagnostic { code: self.name().to_string(), message: format!("Unused import '{}'", specifier), severity: Severity::Warning, line, column });
                }
            }
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(script) = &module.script {
            let mut imports = Vec::new();
            let mut used_vars = HashSet::new();

            // First pass: collect all imports
            for stmt in &script.body {
                if let JsStmt::Import { specifiers, span, .. } = stmt {
                    imports.push((specifiers.clone(), span));
                }
            }

            // Second pass: collect all used variables
            for stmt in &script.body {
                self.collect_used_vars(stmt, &mut used_vars);
            }

            // Find unused imports and generate fix actions
            for (specifiers, span) in &imports {
                let mut unused_specifiers = specifiers.iter().filter(|spec| !used_vars.contains(*spec)).collect::<Vec<_>>();

                if !unused_specifiers.is_empty() {
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        fix_actions.push(FixAction { start, end, replacement: "".to_string(), description: "Remove unused imports".to_string() });
                    }
                }
            }
        }
        Ok(fix_actions)
    }
}

impl NoUnusedImports {
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

    fn collect_used_vars_expr(&self, expr: &nargo_ir::JsExpr, used: &mut HashSet<String>) {
        match expr {
            nargo_ir::JsExpr::Identifier(name, _, _) => {
                used.insert(name.clone());
            }
            nargo_ir::JsExpr::Call { callee, args, .. } => {
                self.collect_used_vars_expr(callee, used);
                for arg in args {
                    self.collect_used_vars_expr(arg, used);
                }
            }
            nargo_ir::JsExpr::Member { object, property, computed, .. } => {
                self.collect_used_vars_expr(object, used);
                if *computed {
                    self.collect_used_vars_expr(property, used);
                }
            }
            nargo_ir::JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.collect_used_vars_expr(el, used);
                }
            }
            nargo_ir::JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.collect_used_vars_expr(val, used);
                }
            }
            nargo_ir::JsExpr::Binary { left, right, .. } => {
                self.collect_used_vars_expr(left, used);
                self.collect_used_vars_expr(right, used);
            }
            nargo_ir::JsExpr::Unary { argument, .. } => {
                self.collect_used_vars_expr(argument, used);
            }
            nargo_ir::JsExpr::ArrowFunction { body, .. } => {
                self.collect_used_vars_expr(body, used);
            }
            _ => {}
        }
    }
}
