use nargo_ir::*;
use nargo_types::{NargoValue, Result};
use std::collections::HashMap;

/// i18n 优化变换插件
pub struct I18nPass {
    pub messages: HashMap<String, String>,
}

impl I18nPass {
    /// 创建新的 i18n 优化变换插件
    pub fn new(messages: HashMap<String, String>) -> Self {
        Self { messages }
    }

    fn transform_node(&self, node: &mut TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &mut el.attributes {
                    if let Some(ast) = &mut attr.value_ast {
                        self.transform_expr(ast);
                        if let JsExpr::Literal(NargoValue::String(s), _, _) = ast {
                            attr.value = Some(format!("'{}", s));
                        }
                        // 如果 AST 被替换成了字面量，且原本是动态属性，现在可以标记为非动态
                        if let JsExpr::Literal(_, _, _) = ast {
                            attr.is_dynamic = false;
                        }
                    }
                }
                for child in &mut el.children {
                    self.transform_node(child);
                }
            }
            TemplateNodeIR::If(if_node) => {
                self.transform_expr_ir(&mut if_node.condition);
                for child in &mut if_node.consequent {
                    self.transform_node(child);
                }
                if let Some(alt) = &mut if_node.alternate {
                    for child in alt {
                        self.transform_node(child);
                    }
                }
                for (cond, nodes) in &mut if_node.else_ifs {
                    self.transform_expr_ir(cond);
                    for node in nodes {
                        self.transform_node(node);
                    }
                }
            }
            TemplateNodeIR::For(for_node) => {
                self.transform_expr_ir(&mut for_node.iterator.collection);
                for child in &mut for_node.body {
                    self.transform_node(child);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &mut expr.ast {
                    self.transform_expr(ast);
                    if let JsExpr::Literal(NargoValue::String(s), _, _) = ast {
                        expr.code = format!("'{}", s);
                    }
                }
            }
            _ => {}
        }
    }

    fn transform_expr_ir(&self, expr: &mut nargo_ir::ExpressionIR) {
        if let Some(ast) = &mut expr.ast {
            self.transform_expr(ast);
        }
    }

    fn transform_stmt(&self, stmt: &mut JsStmt) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.transform_expr(expr),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                self.transform_expr(expr);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.transform_stmt(s);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.transform_stmt(declaration);
            }
            _ => {}
        }
    }

    fn transform_expr(&self, expr: &mut JsExpr) {
        match expr {
            JsExpr::Call { callee, args, span, trivia } => {
                if let JsExpr::Identifier(id, _, _) = &**callee {
                    if id == "$t" && args.len() == 1 {
                        if let JsExpr::Literal(NargoValue::String(key), _, _) = &args[0] {
                            if let Some(translated) = self.messages.get(key) {
                                // 保持原始 Span 以支持 Source Map
                                *expr = JsExpr::Literal(NargoValue::String(translated.clone()), *span, trivia.clone());
                                return;
                            }
                        }
                    }
                }
                for arg in args {
                    self.transform_expr(arg);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.transform_expr(left);
                self.transform_expr(right);
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.transform_expr(el);
                }
            }
            JsExpr::Object(props, _, _) => {
                for val in props.values_mut() {
                    self.transform_expr(val);
                }
            }
            _ => {}
        }
    }
}

impl crate::TransformPass for I18nPass {
    fn name(&self) -> String {
        "I18nPass".to_string()
    }

    fn description(&self) -> String {
        "Optimizes i18n calls by inlining translated strings".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(template) = &mut ir.template {
            for node in &mut template.nodes {
                self.transform_node(node);
            }
        }
        if let Some(script) = &mut ir.script {
            for stmt in &mut script.body {
                self.transform_stmt(stmt);
            }
        }
        Ok(())
    }
}
