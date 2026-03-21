use nargo_ir::JsExpr;

/// 表达式生成器
pub fn generate_expr(expr: &JsExpr) -> String {
    match expr {
        JsExpr::Identifier(name, _, _) => name.clone(),
        JsExpr::Literal(value, _, _) => value.to_string(),
        _ => String::new(),
    }
}
