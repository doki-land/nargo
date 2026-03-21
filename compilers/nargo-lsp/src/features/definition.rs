use crate::utils::{is_in_span, span_to_range};
use nargo_types::{JsProgram, JsStmt, TemplateNodeIR, Span};
use oak_lsp::types::LocationRange;
use oak_core::Arc;

/// Finds the definition location for a symbol at a given offset.
///
/// Navigates from usage sites to their definitions in the script section.
///
/// # Arguments
///
/// * `ir` - The IR module to analyze.
/// * `uri` - The document URI.
/// * `offset` - The character offset of the symbol.
///
/// # Returns
///
/// A vector of location ranges for definitions.
pub fn find_definition_in_ir(
    ir: &nargo_types::IRModule,
    uri: &str,
    offset: usize,
) -> Vec<LocationRange> {
    let mut locations = Vec::new();

    if let Some(template) = &ir.template {
        if is_in_span(template.span, offset) {
            for node in &template.nodes {
                if let Some(loc) = find_definition_in_template_node(node, ir, uri, offset) {
                    locations.push(loc);
                    return locations;
                }
            }
        }
    }

    locations
}

/// Finds the definition location in a template node.
///
/// # Arguments
///
/// * `node` - The template node to search.
/// * `ir` - The parent IR module.
/// * `uri` - The document URI.
/// * `offset` - The character offset.
///
/// # Returns
///
/// An optional location range for the definition.
fn find_definition_in_template_node(
    node: &TemplateNodeIR,
    ir: &nargo_types::IRModule,
    uri: &str,
    offset: usize,
) -> Option<LocationRange> {
    match node {
        TemplateNodeIR::Element(el) => {
            if is_in_span(el.span, offset) {
                for attr in &el.attributes {
                    if is_in_span(attr.span, offset) {
                        if let Some(script) = &ir.script {
                            if let Some(def_span) =
                                find_variable_definition_in_script(script, &attr.name)
                            {
                                return Some(LocationRange {
                                    uri: Arc::from(uri),
                                    range: span_to_range(def_span),
                                });
                            }
                        }
                    }
                }

                for child in &el.children {
                    if let Some(loc) = find_definition_in_template_node(child, ir, uri, offset) {
                        return Some(loc);
                    }
                }
            }
        }
        TemplateNodeIR::Interpolation(expr) => {
            if is_in_span(expr.span, offset) {
                if let Some(script) = &ir.script {
                    if let Some(def_span) =
                        find_variable_definition_in_script(script, expr.code.trim())
                    {
                        return Some(LocationRange {
                            uri: Arc::from(uri),
                            range: span_to_range(def_span),
                        });
                    }
                }
            }
        }
        _ => {}
    }
    None
}

/// Finds a variable definition in a script.
///
/// # Arguments
///
/// * `script` - The script to search.
/// * `name` - The variable name to find.
///
/// # Returns
///
/// The span of the definition if found.
pub fn find_variable_definition_in_script(script: &JsProgram, name: &str) -> Option<Span> {
    for stmt in &script.body {
        match stmt {
            JsStmt::VariableDecl { id, span, .. } if id == name => {
                return Some(*span);
            }
            JsStmt::FunctionDecl { id, span, .. } if id == name => {
                return Some(*span);
            }
            _ => {}
        }
    }
    None
}
