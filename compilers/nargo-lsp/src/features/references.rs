use crate::utils::{is_in_span, span_to_range};
use nargo_types::{JsExpr, JsProgram, JsStmt, TemplateNodeIR};
use oak_lsp::types::LocationRange;
use oak_core::Arc;

/// Finds all references to a symbol in an IR module.
///
/// Searches through script and template sections to find all
/// occurrences of the symbol at the given offset.
///
/// # Arguments
///
/// * `ir` - The IR module to search.
/// * `uri` - The URI of the document.
/// * `offset` - The offset of the symbol to find references for.
///
/// # Returns
///
/// A vector of location ranges for all references.
pub fn find_references_in_ir(
    ir: &nargo_types::IRModule,
    uri: &str,
    offset: usize,
) -> Vec<LocationRange> {
    let mut locations = Vec::new();

    let symbol_name = find_symbol_name_at_offset(ir, offset);

    if let Some(name) = symbol_name {
        if let Some(script) = &ir.script {
            find_references_in_script(script, &name, uri, &mut locations);
        }

        if let Some(template) = &ir.template {
            for node in &template.nodes {
                find_references_in_template_node(node, &name, uri, &mut locations);
            }
        }
    }

    locations
}

/// Finds the symbol name at a given offset in the IR module.
///
/// Searches through script statements and template nodes to locate
/// the symbol at the specified position.
///
/// # Arguments
///
/// * `ir` - The IR module to search.
/// * `offset` - The character offset to look up.
///
/// # Returns
///
/// The symbol name if found at the offset.
fn find_symbol_name_at_offset(ir: &nargo_types::IRModule, offset: usize) -> Option<String> {
    if let Some(script) = &ir.script {
        for stmt in &script.body {
            if is_in_span(stmt.span(), offset) {
                return find_symbol_name_in_stmt(stmt, offset);
            }
        }
    }

    if let Some(template) = &ir.template {
        for node in &template.nodes {
            if let Some(name) = find_symbol_name_in_template_node(node, offset) {
                return Some(name);
            }
        }
    }

    None
}

/// Finds the symbol name in a statement.
///
/// # Arguments
///
/// * `stmt` - The statement to search.
/// * `offset` - The character offset.
///
/// # Returns
///
/// The symbol name if found.
fn find_symbol_name_in_stmt(stmt: &JsStmt, offset: usize) -> Option<String> {
    match stmt {
        JsStmt::VariableDecl { id, init, .. } => {
            if let Some(init_expr) = init {
                if is_in_span(init_expr.span(), offset) {
                    return find_symbol_name_in_expr(init_expr, offset);
                }
            }
            Some(id.clone())
        }
        JsStmt::FunctionDecl { id, .. } => Some(id.clone()),
        JsStmt::Expr(expr, ..) => find_symbol_name_in_expr(expr, offset),
        _ => None,
    }
}

/// Finds the symbol name in an expression.
///
/// # Arguments
///
/// * `expr` - The expression to search.
/// * `offset` - The character offset.
///
/// # Returns
///
/// The symbol name if found.
fn find_symbol_name_in_expr(expr: &JsExpr, offset: usize) -> Option<String> {
    match expr {
        JsExpr::Identifier(name, ..) => Some(name.clone()),
        JsExpr::Binary { left, right, .. } => {
            if is_in_span(left.span(), offset) {
                return find_symbol_name_in_expr(left, offset);
            }
            if is_in_span(right.span(), offset) {
                return find_symbol_name_in_expr(right, offset);
            }
            None
        }
        JsExpr::Call { callee, args, .. } => {
            if is_in_span(callee.span(), offset) {
                return find_symbol_name_in_expr(callee, offset);
            }
            for arg in args {
                if is_in_span(arg.span(), offset) {
                    return find_symbol_name_in_expr(arg, offset);
                }
            }
            None
        }
        _ => None,
    }
}

/// Finds the symbol name in a template node.
///
/// # Arguments
///
/// * `node` - The template node to search.
/// * `offset` - The character offset.
///
/// # Returns
///
/// The symbol name if found.
fn find_symbol_name_in_template_node(node: &TemplateNodeIR, offset: usize) -> Option<String> {
    match node {
        TemplateNodeIR::Interpolation(expr) => {
            if is_in_span(expr.span, offset) {
                return Some(expr.code.trim().to_string());
            }
        }
        TemplateNodeIR::Element(el) => {
            for attr in &el.attributes {
                if is_in_span(attr.span, offset) {
                    return Some(attr.name.clone());
                }
            }
            for child in &el.children {
                if let Some(name) = find_symbol_name_in_template_node(child, offset) {
                    return Some(name);
                }
            }
        }
        _ => {}
    }
    None
}

/// Finds references in a script section.
///
/// # Arguments
///
/// * `script` - The script to search.
/// * `name` - The symbol name to find.
/// * `uri` - The document URI.
/// * `locations` - The output vector for found locations.
fn find_references_in_script(
    script: &JsProgram,
    name: &str,
    uri: &str,
    locations: &mut Vec<LocationRange>,
) {
    for stmt in &script.body {
        find_references_in_stmt(stmt, name, uri, locations);
    }
}

/// Finds references in a statement.
///
/// # Arguments
///
/// * `stmt` - The statement to search.
/// * `name` - The symbol name to find.
/// * `uri` - The document URI.
/// * `locations` - The output vector for found locations.
fn find_references_in_stmt(
    stmt: &JsStmt,
    name: &str,
    uri: &str,
    locations: &mut Vec<LocationRange>,
) {
    match stmt {
        JsStmt::VariableDecl {
            id, init, span, ..
        } => {
            if id == name {
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: span_to_range(*span),
                });
            }
            if let Some(init_expr) = init {
                find_references_in_expr(init_expr, name, uri, locations);
            }
        }
        JsStmt::FunctionDecl {
            id, body, span, ..
        } => {
            if id == name {
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: span_to_range(*span),
                });
            }
            for s in body {
                find_references_in_stmt(s, name, uri, locations);
            }
        }
        JsStmt::Expr(expr, ..) => {
            find_references_in_expr(expr, name, uri, locations)
        }
        JsStmt::If {
            test,
            consequent,
            alternate,
            ..
        } => {
            find_references_in_expr(test, name, uri, locations);
            for s in consequent {
                find_references_in_stmt(s, name, uri, locations);
            }
            if let Some(alt) = alternate {
                for s in alt {
                    find_references_in_stmt(s, name, uri, locations);
                }
            }
        }
        JsStmt::Block(body, ..) => {
            for s in body {
                find_references_in_stmt(s, name, uri, locations);
            }
        }
        JsStmt::Return(expr, ..) => {
            find_references_in_expr(expr, name, uri, locations);
        }
        JsStmt::For {
            init,
            test,
            update,
            body,
            ..
        } => {
            find_references_in_stmt(init, name, uri, locations);
            if let Some(test) = test {
                find_references_in_expr(test, name, uri, locations);
            }
            if let Some(update) = update {
                find_references_in_expr(update, name, uri, locations);
            }
            for s in body {
                find_references_in_stmt(s, name, uri, locations);
            }
        }
        JsStmt::While { test, body, .. } => {
            find_references_in_expr(test, name, uri, locations);
            for s in body {
                find_references_in_stmt(s, name, uri, locations);
            }
        }
        _ => {}
    }
}

/// Finds references in an expression.
///
/// # Arguments
///
/// * `expr` - The expression to search.
/// * `name` - The symbol name to find.
/// * `uri` - The document URI.
/// * `locations` - The output vector for found locations.
fn find_references_in_expr(
    expr: &JsExpr,
    name: &str,
    uri: &str,
    locations: &mut Vec<LocationRange>,
) {
    match expr {
        JsExpr::Identifier(n, span, _) => {
            if n == name {
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: span_to_range(*span),
                });
            }
        }
        JsExpr::Binary { left, right, .. } => {
            find_references_in_expr(left, name, uri, locations);
            find_references_in_expr(right, name, uri, locations);
        }
        JsExpr::Call { callee, args, .. } => {
            find_references_in_expr(callee, name, uri, locations);
            for arg in args {
                find_references_in_expr(arg, name, uri, locations);
            }
        }
        JsExpr::Member {
            object, ..
        } => {
            find_references_in_expr(object, name, uri, locations);
        }
        JsExpr::Object(props, ..) => {
            for val in props.values() {
                find_references_in_expr(val, name, uri, locations);
            }
        }
        JsExpr::Array(elements, ..) => {
            for el in elements {
                find_references_in_expr(el, name, uri, locations);
            }
        }
        JsExpr::Unary { argument, .. } => {
            find_references_in_expr(argument, name, uri, locations);
        }
        JsExpr::Conditional {
            test,
            consequent,
            alternate,
            ..
        } => {
            find_references_in_expr(test, name, uri, locations);
            find_references_in_expr(consequent, name, uri, locations);
            find_references_in_expr(alternate, name, uri, locations);
        }
        _ => {}
    }
}

/// Finds references in a template node.
///
/// # Arguments
///
/// * `node` - The template node to search.
/// * `name` - The symbol name to find.
/// * `uri` - The document URI.
/// * `locations` - The output vector for found locations.
fn find_references_in_template_node(
    node: &TemplateNodeIR,
    name: &str,
    uri: &str,
    locations: &mut Vec<LocationRange>,
) {
    match node {
        TemplateNodeIR::Interpolation(expr) => {
            if let Some(ast) = &expr.ast {
                find_references_in_expr(ast, name, uri, locations);
            } else if expr.code.contains(name) {
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: span_to_range(expr.span),
                });
            }
        }
        TemplateNodeIR::Element(el) => {
            if el.tag == name {
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: span_to_range(el.span),
                });
            }

            for attr in &el.attributes {
                if attr.name == name {
                    locations.push(LocationRange {
                        uri: Arc::from(uri),
                        range: span_to_range(attr.span),
                    });
                }

                if let Some(ast) = &attr.value_ast {
                    find_references_in_expr(ast, name, uri, locations);
                } else if let Some(val) = &attr.value {
                    if val.contains(name) {
                        locations.push(LocationRange {
                            uri: Arc::from(uri),
                            range: span_to_range(attr.span),
                        });
                    }
                }
            }

            for child in &el.children {
                find_references_in_template_node(child, name, uri, locations);
            }
        }
        _ => {}
    }
}
