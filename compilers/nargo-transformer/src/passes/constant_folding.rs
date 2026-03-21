use nargo_ir::*;
use nargo_types::{NargoValue, Result};

/// 常量折叠优化插件
pub struct ConstantFoldingPass;

impl ConstantFoldingPass {
    /// 创建新的常量折叠优化插件
    pub fn new() -> Self {
        Self
    }

    fn fold_expr(&self, expr: &mut JsExpr) {
        match expr {
            JsExpr::Binary { left, op, right, span, trivia } => {
                self.fold_expr(left);
                self.fold_expr(right);

                if let (JsExpr::Literal(NargoValue::Number(l), _, _), JsExpr::Literal(NargoValue::Number(r), _, _)) = (&**left, &**right) {
                    match op.as_str() {
                        "+" => *expr = JsExpr::Literal(NargoValue::Number(l + r), *span, trivia.clone()),
                        "-" => *expr = JsExpr::Literal(NargoValue::Number(l - r), *span, trivia.clone()),
                        "*" => *expr = JsExpr::Literal(NargoValue::Number(l * r), *span, trivia.clone()),
                        "/" => {
                            if *r != 0.0 {
                                *expr = JsExpr::Literal(NargoValue::Number(l / r), *span, trivia.clone())
                            }
                        }
                        "%" => {
                            if *r != 0.0 {
                                *expr = JsExpr::Literal(NargoValue::Number(l % r), *span, trivia.clone())
                            }
                        }
                        "==" => *expr = JsExpr::Literal(NargoValue::Bool(l == r), *span, trivia.clone()),
                        "!=" => *expr = JsExpr::Literal(NargoValue::Bool(l != r), *span, trivia.clone()),
                        "<" => *expr = JsExpr::Literal(NargoValue::Bool(l < r), *span, trivia.clone()),
                        "<=" => *expr = JsExpr::Literal(NargoValue::Bool(l <= r), *span, trivia.clone()),
                        ">" => *expr = JsExpr::Literal(NargoValue::Bool(l > r), *span, trivia.clone()),
                        ">=" => *expr = JsExpr::Literal(NargoValue::Bool(l >= r), *span, trivia.clone()),
                        _ => {}
                    }
                }
                else if let (JsExpr::Literal(NargoValue::String(l), _, _), JsExpr::Literal(NargoValue::String(r), _, _)) = (&**left, &**right) {
                    match op.as_str() {
                        "+" => *expr = JsExpr::Literal(NargoValue::String(format!("{}{}", l, r)), *span, trivia.clone()),
                        "==" => *expr = JsExpr::Literal(NargoValue::Bool(l == r), *span, trivia.clone()),
                        "!=" => *expr = JsExpr::Literal(NargoValue::Bool(l != r), *span, trivia.clone()),
                        _ => {}
                    }
                }
                else if let (JsExpr::Literal(NargoValue::Bool(l), _, _), JsExpr::Literal(NargoValue::Bool(r), _, _)) = (&**left, &**right) {
                    match op.as_str() {
                        "==" => *expr = JsExpr::Literal(NargoValue::Bool(l == r), *span, trivia.clone()),
                        "!=" => *expr = JsExpr::Literal(NargoValue::Bool(l != r), *span, trivia.clone()),
                        "&&" => *expr = JsExpr::Literal(NargoValue::Bool(*l && *r), *span, trivia.clone()),
                        "||" => *expr = JsExpr::Literal(NargoValue::Bool(*l || *r), *span, trivia.clone()),
                        _ => {}
                    }
                }
            }
            JsExpr::Unary { op, argument, span, trivia } => {
                self.fold_expr(argument);
                if let JsExpr::Literal(val, _, _) = &**argument {
                    match (op.as_str(), val) {
                        ("-", NargoValue::Number(n)) => *expr = JsExpr::Literal(NargoValue::Number(-n), *span, trivia.clone()),
                        ("!", NargoValue::Bool(b)) => *expr = JsExpr::Literal(NargoValue::Bool(!b), *span, trivia.clone()),
                        ("typeof", _) => {
                            let type_str = match val {
                                NargoValue::Number(_) => "number",
                                NargoValue::String(_) => "string",
                                NargoValue::Bool(_) => "boolean",
                                _ => "object",
                            };
                            *expr = JsExpr::Literal(NargoValue::String(type_str.to_string()), *span, trivia.clone());
                        }
                        _ => {}
                    }
                }
            }
            JsExpr::Conditional { test, consequent, alternate, span, trivia } => {
                self.fold_expr(test);
                self.fold_expr(consequent);
                self.fold_expr(alternate);

                if let JsExpr::Literal(NargoValue::Bool(b), _, _) = &**test {
                    if *b {
                        *expr = *std::mem::replace(consequent, Box::new(JsExpr::Literal(NargoValue::Null, *span, trivia.clone())));
                    }
                    else {
                        *expr = *std::mem::replace(alternate, Box::new(JsExpr::Literal(NargoValue::Null, *span, trivia.clone())));
                    }
                }
            }
            JsExpr::Array(elements, _, _) => {
                elements.iter_mut().for_each(|el| {
                    self.fold_expr(el);
                });
            }
            JsExpr::Object(props, _, _) => {
                props.values_mut().for_each(|val| {
                    self.fold_expr(val);
                });
            }
            JsExpr::Call { callee, args, .. } => {
                self.fold_expr(callee);
                args.iter_mut().for_each(|arg| {
                    self.fold_expr(arg);
                });
            }
            _ => {}
        }
    }

    fn fold_stmt(&self, stmt: &mut JsStmt) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.fold_expr(expr),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                self.fold_expr(expr);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.fold_stmt(s);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                self.fold_expr(test);
                self.fold_stmt(consequent);
                if let Some(alt) = alternate {
                    self.fold_stmt(alt);
                }
            }
            JsStmt::Return(Some(expr), _, _) => self.fold_expr(expr),
            JsStmt::Block(stmts, _, _) => {
                for s in stmts {
                    self.fold_stmt(s);
                }
            }
            _ => {}
        }
    }

    fn fold_node(&self, node: &mut TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &mut el.attributes {
                    if let Some(ast) = &mut attr.value_ast {
                        self.fold_expr(ast);
                    }
                }
                for child in &mut el.children {
                    self.fold_node(child);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &mut expr.ast {
                    self.fold_expr(ast);
                }
            }
            _ => {}
        }
    }
}

impl crate::TransformPass for ConstantFoldingPass {
    fn name(&self) -> String {
        "ConstantFoldingPass".to_string()
    }

    fn description(&self) -> String {
        "Evaluates constant expressions at compile time".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &mut ir.script {
            // 处理脚本语句
            script.body.iter_mut().for_each(|stmt| {
                self.fold_stmt(stmt);
            });
        }
        if let Some(template) = &mut ir.template {
            // 处理模板节点
            template.nodes.iter_mut().for_each(|node| {
                self.fold_node(node);
            });
        }
        Ok(())
    }
}
