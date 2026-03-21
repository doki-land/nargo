#![warn(missing_docs)]

use nargo_ir::{IRModule, JsExpr, JsStmt, TemplateNodeIR};
use std::collections::HashSet;

use crate::types::FeatureSet;

/// 分析模块的依赖关系
pub fn analyze_dependencies(module: &IRModule) -> HashSet<String> {
    let mut dependencies = HashSet::new();

    // 分析脚本中的导入
    if let Some(script) = &module.script {
        for stmt in &script.body {
            analyze_stmt_for_dependencies(stmt, &mut dependencies);
        }
    }

    if let Some(script) = &module.script_client {
        for stmt in &script.body {
            analyze_stmt_for_dependencies(stmt, &mut dependencies);
        }
    }

    if let Some(script) = &module.script_server {
        for stmt in &script.body {
            analyze_stmt_for_dependencies(stmt, &mut dependencies);
        }
    }

    dependencies
}

/// 分析语句中的依赖关系
fn analyze_stmt_for_dependencies(stmt: &JsStmt, dependencies: &mut HashSet<String>) {
    // 这里需要实现具体的依赖分析逻辑
    // 暂时返回空集合
}

/// 分析单个模块的特征
pub fn analyze_module_into(module: &IRModule, features: &mut FeatureSet) {
    // 1. 分析模板中的 VDOM 和特定指令
    if let Some(template) = &module.template {
        if !template.nodes.is_empty() {
            features.has_vdom = true;
            features.used_dom_functions.insert("h".to_string());
        }
        for node in &template.nodes {
            analyze_node_into(node, features);
        }
    }

    // 2. 分析脚本中的响应式和核心函数
    if let Some(script) = &module.script {
        for stmt in &script.body {
            analyze_stmt_into(stmt, features);
        }
    }

    if let Some(script) = &module.script_client {
        for stmt in &script.body {
            analyze_stmt_into(stmt, features);
        }
    }

    if let Some(script) = &module.script_server {
        for stmt in &script.body {
            analyze_stmt_into(stmt, features);
            features.has_ssr = true;
        }
    }
}

/// 分析语句中的特征
fn analyze_stmt_into(stmt: &JsStmt, features: &mut FeatureSet) {
    match stmt {
        JsStmt::Expr(expr, _, _) => analyze_expr_into(expr, features),
        JsStmt::VariableDecl { init, .. } => {
            if let Some(expr) = init {
                analyze_expr_into(expr, features);
            }
        }
        JsStmt::Return(expr, _, _) => {
            if let Some(expr) = expr {
                analyze_expr_into(expr, features);
            }
        }
        JsStmt::If { test, consequent, alternate, .. } => {
            analyze_expr_into(test, features);
            analyze_stmt_into(consequent, features);
            if let Some(alt) = alternate {
                analyze_stmt_into(alt, features);
            }
        }
        JsStmt::While { test, body, .. } => {
            analyze_expr_into(test, features);
            analyze_stmt_into(body, features);
        }
        JsStmt::For { init, test, update, body, .. } => {
            if let Some(init) = init {
                analyze_stmt_into(init, features);
            }
            if let Some(test) = test {
                analyze_expr_into(test, features);
            }
            if let Some(update) = update {
                analyze_expr_into(update, features);
            }
            analyze_stmt_into(body, features);
        }
        JsStmt::Block(stmts, _, _) => {
            for s in stmts {
                analyze_stmt_into(s, features);
            }
        }
        JsStmt::FunctionDecl { body, .. } => {
            for s in body {
                analyze_stmt_into(s, features);
            }
        }
        JsStmt::Export { declaration, .. } => analyze_stmt_into(declaration, features),
        _ => {}
    }
}

/// 分析表达式中的特征
fn analyze_expr_into(expr: &JsExpr, features: &mut FeatureSet) {
    match expr {
        JsExpr::Call { callee, args, .. } => {
            if let JsExpr::Identifier(name, _, _) = &**callee {
                match name.as_str() {
                    "signal" | "createSignal" => {
                        features.has_signals = true;
                        features.used_core_functions.insert(name.clone());
                    }
                    "effect" | "createEffect" => {
                        features.has_effects = true;
                        features.used_core_functions.insert(name.clone());
                    }
                    "computed" | "createComputed" => {
                        features.has_signals = true;
                        features.used_core_functions.insert(name.clone());
                    }
                    _ => {
                        features.used_core_functions.insert(name.clone());
                    }
                }
            }
            for arg in args {
                analyze_expr_into(arg, features);
            }
        }
        JsExpr::Binary { left, right, .. } => {
            analyze_expr_into(left, features);
            analyze_expr_into(right, features);
        }
        JsExpr::Unary { argument, .. } => {
            analyze_expr_into(argument, features);
        }
        JsExpr::Array(exprs, _, _) => {
            for e in exprs {
                analyze_expr_into(e, features);
            }
        }
        JsExpr::Object(map, _, _) => {
            for e in map.values() {
                analyze_expr_into(e, features);
            }
        }
        JsExpr::ArrowFunction { body, .. } => {
            analyze_expr_into(body, features);
        }
        JsExpr::Conditional { test, consequent, alternate, .. } => {
            analyze_expr_into(test, features);
            analyze_expr_into(consequent, features);
            analyze_expr_into(alternate, features);
        }
        JsExpr::TemplateLiteral { expressions, .. } => {
            for e in expressions {
                analyze_expr_into(e, features);
            }
        }
        _ => {}
    }
}

/// 分析节点中的特征
fn analyze_node_into(node: &TemplateNodeIR, features: &mut FeatureSet) {
    match node {
        TemplateNodeIR::Element(el) => {
            for attr in &el.attributes {
                if let Some(ast) = &attr.value_ast {
                    analyze_expr_into(ast, features);
                }
                // 检查指令
                if attr.is_directive {
                    features.has_effects = true;
                    features.used_core_functions.insert("createEffect".to_string());
                }
            }
            for child in &el.children {
                analyze_node_into(child, features);
            }
        }
        TemplateNodeIR::Interpolation(expr) => {
            features.has_effects = true;
            features.used_core_functions.insert("createEffect".to_string());
            if let Some(ast) = &expr.ast {
                analyze_expr_into(ast, features);
            }
        }
        _ => {}
    }
}

/// 分析模块是否为路由组件
pub fn is_route_component(module: &IRModule) -> bool {
    // 检查模块是否包含路由相关的特征
    if let Some(script) = &module.script {
        for stmt in &script.body {
            if analyze_stmt_for_route(stmt) {
                return true;
            }
        }
    }
    false
}

/// 分析语句是否包含路由相关代码
fn analyze_stmt_for_route(stmt: &JsStmt) -> bool {
    match stmt {
        JsStmt::Expr(expr, _, _) => analyze_expr_for_route(expr),
        JsStmt::VariableDecl { init, .. } => {
            if let Some(expr) = init {
                analyze_expr_for_route(expr)
            }
            else {
                false
            }
        }
        JsStmt::Return(expr, _, _) => {
            if let Some(expr) = expr {
                analyze_expr_for_route(expr)
            }
            else {
                false
            }
        }
        JsStmt::Block(stmts, _, _) => stmts.iter().any(|s| analyze_stmt_for_route(s)),
        _ => false,
    }
}

/// 分析表达式是否包含路由相关代码
fn analyze_expr_for_route(expr: &JsExpr) -> bool {
    match expr {
        JsExpr::Call { callee, .. } => {
            if let JsExpr::Identifier(name, _, _) = &**callee {
                // 检查是否使用了路由相关函数
                matches!(name.as_str(), "useRoute" | "useRouter" | "createRouter" | "createWebHistory" | "createWebHashHistory")
            }
            else {
                false
            }
        }
        JsExpr::Object(map, _, _) => {
            // 检查是否包含路由配置对象
            map.contains_key("path") || map.contains_key("component") || map.contains_key("children")
        }
        _ => false,
    }
}
