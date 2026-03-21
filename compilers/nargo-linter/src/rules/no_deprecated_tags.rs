//! No deprecated tags rule.

use crate::{Diagnostic, FixAction, LintRule};
use async_trait::async_trait;
use nargo_types::Result;

use nargo_ir::{IRModule, TemplateNodeIR};

/// No deprecated tags rule.
pub struct NoDeprecatedTags;

#[async_trait]
impl LintRule for NoDeprecatedTags {
    fn name(&self) -> &'static str {
        "no-deprecated-tags"
    }

    async fn check(&self, _source: &str, module: &IRModule) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if let Some(template) = &module.template {
            for node in &template.nodes {
                self.check_node(node, &mut diagnostics);
            }
        }
        Ok(diagnostics)
    }

    async fn fix(&self, source: &str, module: &IRModule) -> Result<Vec<FixAction>> {
        let mut fix_actions = Vec::new();
        if let Some(template) = &module.template {
            for node in &template.nodes {
                self.fix_node(node, source, &mut fix_actions);
            }
        }
        Ok(fix_actions)
    }
}

impl NoDeprecatedTags {
    fn check_node(&self, node: &TemplateNodeIR, diagnostics: &mut Vec<Diagnostic>) {
        match node {
            TemplateNodeIR::Element(element) => {
                // Check if the tag is deprecated
                let deprecated_tags = vec!["center", "font", "basefont", "big", "small", "strike", "tt", "u", "s"];
                if deprecated_tags.contains(&element.tag.as_str()) {
                    diagnostics.push(Diagnostic { code: self.name().to_string(), message: format!("Deprecated tag '{}' found", element.tag), severity: crate::Severity::Warning, line: 1, column: 1 });
                }
                // Check child nodes
                for child in &element.children {
                    self.check_node(child, diagnostics);
                }
            }
            TemplateNodeIR::If(if_node) => {
                for child in &if_node.consequent {
                    self.check_node(child, diagnostics);
                }
                if let Some(alternate) = &if_node.alternate {
                    for child in alternate {
                        self.check_node(child, diagnostics);
                    }
                }
                for (_, children) in &if_node.else_ifs {
                    for child in children {
                        self.check_node(child, diagnostics);
                    }
                }
            }
            TemplateNodeIR::For(for_node) => {
                for child in &for_node.body {
                    self.check_node(child, diagnostics);
                }
            }
            _ => {}
        }
    }

    fn fix_node(&self, node: &TemplateNodeIR, source: &str, fix_actions: &mut Vec<FixAction>) {
        match node {
            TemplateNodeIR::Element(element) => {
                // Map deprecated tags to modern alternatives
                let tag_map = std::collections::HashMap::from([("center", "div"), ("font", "span"), ("basefont", "span"), ("big", "span"), ("small", "span"), ("strike", "span"), ("tt", "span"), ("u", "span"), ("s", "span")]);

                if let Some(replacement_tag) = tag_map.get(element.tag.as_str()) {
                    // Generate fix action to replace deprecated tag
                    let span = element.span;
                    let start = span.start.offset as usize;
                    let end = span.end.offset as usize;
                    if start < end && end <= source.len() {
                        // This is a simplified fix - in a real implementation you would need to
                        // parse the tag and replace just the tag name, preserving attributes
                        fix_actions.push(FixAction { start, end, replacement: format!("<{}>{}</{}", replacement_tag, "", replacement_tag), description: format!("Replace deprecated tag '{}' with '{}'", element.tag, replacement_tag) });
                    }
                }
                // Check child nodes
                for child in &element.children {
                    self.fix_node(child, source, fix_actions);
                }
            }
            TemplateNodeIR::If(if_node) => {
                for child in &if_node.consequent {
                    self.fix_node(child, source, fix_actions);
                }
                if let Some(alternate) = &if_node.alternate {
                    for child in alternate {
                        self.fix_node(child, source, fix_actions);
                    }
                }
                for (_, children) in &if_node.else_ifs {
                    for child in children {
                        self.fix_node(child, source, fix_actions);
                    }
                }
            }
            TemplateNodeIR::For(for_node) => {
                for child in &for_node.body {
                    self.fix_node(child, source, fix_actions);
                }
            }
            _ => {}
        }
    }
}
