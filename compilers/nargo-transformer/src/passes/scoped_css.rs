use nargo_ir::*;
use nargo_types::Result;

/// Scoped CSS 变换 Pass
pub struct ScopedCssPass {
    pub scope_id: String,
}

impl ScopedCssPass {
    /// 创建新的 Scoped CSS 变换 Pass
    pub fn new(scope_id: String) -> Self {
        Self { scope_id }
    }

    fn apply_scope_id_to_nodes(&self, nodes: &mut [nargo_ir::TemplateNodeIR]) {
        for node in nodes {
            match node {
                nargo_ir::TemplateNodeIR::Element(el) => {
                    let el_span = el.span;
                    el.attributes.push(nargo_ir::AttributeIR { name: self.scope_id.clone(), value: None, value_ast: None, argument: None, modifiers: Vec::new(), is_directive: false, is_dynamic: false, span: el_span, trivia: nargo_ir::Trivia::default() });
                    self.apply_scope_id_to_nodes(&mut el.children);
                }
                nargo_ir::TemplateNodeIR::If(if_node) => {
                    self.apply_scope_id_to_nodes(&mut if_node.consequent);
                    if let Some(alt) = &mut if_node.alternate {
                        self.apply_scope_id_to_nodes(alt);
                    }
                    for (_, cond_nodes) in &mut if_node.else_ifs {
                        self.apply_scope_id_to_nodes(cond_nodes);
                    }
                }
                nargo_ir::TemplateNodeIR::For(for_node) => {
                    self.apply_scope_id_to_nodes(&mut for_node.body);
                }
                _ => {}
            }
        }
    }

    fn transform_scoped_css(&self, css: &str) -> String {
        let mut result = String::new();
        for line in css.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.contains('{') {
                let parts: Vec<&str> = line.split('{').collect();
                let selectors: Vec<&str> = parts[0].split(',').collect();
                let scoped_selectors: Vec<String> = selectors.iter().map(|s| format!("{}[{}]", s.trim(), self.scope_id)).collect();
                result.push_str(&scoped_selectors.join(", "));
                result.push_str(" {");
                if parts.len() > 1 {
                    result.push_str(parts[1]);
                }
                result.push('\n');
            }
            else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }
}

impl crate::TransformPass for ScopedCssPass {
    fn name(&self) -> String {
        "ScopedCssPass".to_string()
    }

    fn description(&self) -> String {
        format!("Injects scope ID ({}) into template and styles", self.scope_id)
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(template) = &mut ir.template {
            self.apply_scope_id_to_nodes(&mut template.nodes);
        }

        // 2. Transform styles
        for style in &mut ir.styles {
            if style.scoped {
                style.code = self.transform_scoped_css(&style.code);
            }
        }

        Ok(())
    }
}
