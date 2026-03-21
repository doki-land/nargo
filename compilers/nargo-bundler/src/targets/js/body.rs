use nargo_ir::{IRModule, JsExpr, JsStmt};
use std::collections::HashMap;

/// 收集常量绑定
pub fn collect_const_bindings(ir: &IRModule) -> HashMap<String, JsExpr> {
    let mut bindings = HashMap::new();

    // 收集主脚本中的常量绑定
    if let Some(script) = &ir.script {
        for stmt in &script.body {
            if let JsStmt::VariableDecl { id, init, kind, .. } = stmt {
                if kind == &"const" && init.is_some() {
                    bindings.insert(id.clone(), init.as_ref().unwrap().clone());
                }
            }
        }
    }

    // 收集客户端脚本中的常量绑定
    if let Some(script) = &ir.script_client {
        for stmt in &script.body {
            if let JsStmt::VariableDecl { id, init, kind, .. } = stmt {
                if kind == &"const" && init.is_some() {
                    bindings.insert(id.clone(), init.as_ref().unwrap().clone());
                }
            }
        }
    }

    // 收集服务器脚本中的常量绑定
    if let Some(script) = &ir.script_server {
        for stmt in &script.body {
            if let JsStmt::VariableDecl { id, init, kind, .. } = stmt {
                if kind == &"const" && init.is_some() {
                    bindings.insert(id.clone(), init.as_ref().unwrap().clone());
                }
            }
        }
    }

    bindings
}
