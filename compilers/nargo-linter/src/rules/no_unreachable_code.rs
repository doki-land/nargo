//! No unreachable code rule.

use crate::{Diagnostic, FixAction, LintRule, Severity};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, JsStmt};

/// No unreachable code rule.
pub struct NoUnreachableCode;

#[async_trait]
impl LintRule for NoUnreachableCode {
    fn name(&self) -> &'static str {
        "no-unreachable-code"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(script) = &module.script {
            self.check_block(&script.body, &mut diagnostics);
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(script) = &module.script {
            self.fix_block(&script.body, source, &mut fix_actions);
        }
        Ok(fix_actions)
    }
}

impl NoUnreachableCode {
    fn is_terminal_stmt(&self, stmt: &JsStmt) -> bool {
        match stmt {
            JsStmt::Return { .. } => true,
            JsStmt::Break { .. } => true,
            JsStmt::Continue { .. } => true,
            JsStmt::Block(stmts, _, _) => {
                // Check if block ends with terminal statement
                let mut is_terminal = false;
                for s in stmts {
                    is_terminal = self.is_terminal_stmt(s);
                }
                is_terminal
            }
            _ => false,
        }
    }

    fn check_block(&self, stmts: &Vec<JsStmt>, diagnostics: &mut Vec<Diagnostic>) {
        let mut is_unreachable = false;
        for stmt in stmts {
            if is_unreachable {
                diagnostics.push(Diagnostic { code: self.name().to_string(), message: "Unreachable code".to_string(), severity: Severity::Warning, line: 1, column: 1 });
            }
            else {
                // Check function bodies
                if let JsStmt::FunctionDecl { body, .. } = stmt {
                    self.check_block(body, diagnostics);
                }
                else if let JsStmt::Block(block_stmts, _, _) = stmt {
                    self.check_block(block_stmts, diagnostics);
                }
            }
            is_unreachable = self.is_terminal_stmt(stmt);
        }
    }

    fn fix_block(&self, stmts: &Vec<JsStmt>, source: &str, fix_actions: &mut Vec<FixAction>) {
        let mut is_unreachable = false;
        for stmt in stmts {
            if is_unreachable {
                // Generate fix action to remove unreachable code
                let span = stmt.span();
                let start = span.start.offset as usize;
                let end = span.end.offset as usize;
                if start < end && end <= source.len() {
                    fix_actions.push(FixAction { start, end, replacement: "".to_string(), description: "Remove unreachable code".to_string() });
                }
            }
            else {
                // Check function bodies
                if let JsStmt::FunctionDecl { body, .. } = stmt {
                    self.fix_block(body, source, fix_actions);
                }
                else if let JsStmt::Block(block_stmts, _, _) = stmt {
                    self.fix_block(block_stmts, source, fix_actions);
                }
            }
            is_unreachable = self.is_terminal_stmt(stmt);
        }
    }
}
