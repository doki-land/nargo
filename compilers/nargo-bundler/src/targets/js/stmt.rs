use nargo_ir::JsStmt;

/// 语句生成器
pub fn generate_stmt(stmt: &JsStmt) -> String {
    match stmt {
        JsStmt::Expr(expr, _, _) => format!("{};", super::expr::generate_expr(expr)),
        _ => String::new(),
    }
}
