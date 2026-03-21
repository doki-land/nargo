use nargo_ir::TemplateNodeIR;

/// 模板节点生成器
pub fn generate_node(node: &TemplateNodeIR) -> String {
    match node {
        TemplateNodeIR::Text(text, ..) => format!("'{}'", text),
        _ => String::new(),
    }
}
