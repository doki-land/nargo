use nargo_ir::*;
use nargo_types::{NargoValue, Result};
use std::collections::HashMap;

/// 调用计数分析插件
pub struct CallCountPass {
    pub counts: HashMap<String, usize>,
}

impl CallCountPass {
    /// 创建新的调用计数分析插件
    pub fn new() -> Self {
        Self { counts: HashMap::new() }
    }

    fn track_stmt(&mut self, stmt: &JsStmt) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.track_expr(expr),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                self.track_expr(expr);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.track_stmt(s);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.track_stmt(declaration);
            }
            _ => {}
        }
    }

    fn track_expr(&mut self, expr: &JsExpr) {
        match expr {
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(id, _, _) = &**callee {
                    *self.counts.entry(id.clone()).or_insert(0) += 1;
                }
                self.track_expr(callee);
                for arg in args {
                    self.track_expr(arg);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.track_expr(left);
                self.track_expr(right);
            }
            JsExpr::Unary { argument, .. } => {
                self.track_expr(argument);
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.track_expr(el);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values() {
                    self.track_expr(val);
                }
            }
            _ => {}
        }
    }

    fn track_node(&mut self, node: &TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &el.attributes {
                    if let Some(expr) = &attr.value_ast {
                        self.track_expr(expr);
                    }
                }
                for child in &el.children {
                    self.track_node(child);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &expr.ast {
                    self.track_expr(ast);
                }
            }
            _ => {}
        }
    }
}

impl crate::TransformPass for CallCountPass {
    fn name(&self) -> String {
        "CallCountPass".to_string()
    }

    fn description(&self) -> String {
        "Analyzes call frequencies for identifiers across script and template".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &ir.script {
            for stmt in &script.body {
                self.track_stmt(stmt);
            }
        }
        if let Some(template) = &ir.template {
            for node in &template.nodes {
                self.track_node(node);
            }
        }

        // 将分析结果存储在 IR 的 metadata 中，以便后续 Pass 使用
        for (id, count) in &self.counts {
            ir.metadata.insert(format!("call_count:{}", id), NargoValue::String(count.to_string()));
        }

        Ok(())
    }
}
