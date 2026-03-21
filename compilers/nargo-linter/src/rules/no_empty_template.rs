//! No empty template rule.

use crate::{Diagnostic, FixAction, LintRule};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::IRModule;

/// No empty template rule.
pub struct NoEmptyTemplate;

#[async_trait]
impl LintRule for NoEmptyTemplate {
    fn name(&self) -> &'static str {
        "no-empty-template"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(template) = &module.template {
            if template.nodes.is_empty() {
                diagnostics.push(Diagnostic { code: self.name().to_string(), message: "Empty template found".to_string(), severity: crate::Severity::Warning, line: 1, column: 1 });
            }
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(template) = &module.template {
            if template.nodes.is_empty() {
                // For empty template, we can add a comment or a default element
                // This is a simple fix, in a real implementation you might want to be more sophisticated
                let span = template.span;
                let start = span.start.offset as usize;
                let end = span.end.offset as usize;
                if start < end && end <= source.len() {
                    fix_actions.push(FixAction { start, end, replacement: "<!-- Empty template -->".to_string(), description: "Add comment to empty template".to_string() });
                }
            }
        }
        Ok(fix_actions)
    }
}
