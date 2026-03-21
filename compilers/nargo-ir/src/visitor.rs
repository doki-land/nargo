#![warn(missing_docs)]

//! 访问者模块
//!
//! 提供默认访问者实现，用于遍历和处理 IR 结构。

use crate::{
    expr::{JsExpr, JsExprVisitor, TseAttribute},
    stmt::{JsStmt, JsStmtVisitor},
    template::{ElementIR, ExpressionIR, ForNodeIR, IfNodeIR, TemplateNodeIR, TemplateNodeVisitor},
    types::Trivia,
};
use nargo_types::{NargoValue, Span};

/// 简单的默认访问者实现
pub struct DefaultVisitor;

impl JsExprVisitor<()> for DefaultVisitor {
    fn visit_identifier(&mut self, _id: &String, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_literal(&mut self, _value: &NargoValue, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_unary(&mut self, _op: &String, argument: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        argument.accept(self);
    }
    fn visit_binary(&mut self, left: &JsExpr, _op: &String, right: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
    }
    fn visit_call(&mut self, callee: &JsExpr, args: &Vec<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        callee.accept(self);
        for arg in args {
            arg.accept(self);
        }
    }
    fn visit_member(&mut self, object: &JsExpr, property: &JsExpr, _computed: bool, _span: &Span, _trivia: &Trivia) -> () {
        object.accept(self);
        property.accept(self);
    }
    fn visit_optional_member(&mut self, object: &JsExpr, property: &JsExpr, _computed: bool, _span: &Span, _trivia: &Trivia) -> () {
        object.accept(self);
        property.accept(self);
    }
    fn visit_optional_call(&mut self, callee: &JsExpr, args: &Vec<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        callee.accept(self);
        for arg in args {
            arg.accept(self);
        }
    }
    fn visit_nullish_coalescing(&mut self, left: &JsExpr, right: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
    }
    fn visit_logical_assignment(&mut self, _op: &String, left: &JsExpr, right: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
    }
    fn visit_array(&mut self, items: &Vec<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        for item in items {
            item.accept(self);
        }
    }
    fn visit_object(&mut self, props: &std::collections::HashMap<String, JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        for (_key, value) in props {
            value.accept(self);
        }
    }
    fn visit_arrow_function(&mut self, _params: &Vec<String>, body: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        body.accept(self);
    }
    fn visit_tse_element(&mut self, _tag: &String, _attributes: &Vec<TseAttribute>, children: &Vec<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        for child in children {
            child.accept(self);
        }
    }
    fn visit_conditional(&mut self, test: &JsExpr, consequent: &JsExpr, alternate: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        test.accept(self);
        consequent.accept(self);
        alternate.accept(self);
    }
    fn visit_template_literal(&mut self, _quasis: &Vec<String>, expressions: &Vec<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        for expr in expressions {
            expr.accept(self);
        }
    }
    fn visit_spread(&mut self, expr: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        expr.accept(self);
    }
    fn visit_type_of(&mut self, expr: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        expr.accept(self);
    }
    fn visit_instance_of(&mut self, left: &JsExpr, right: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
    }
    fn visit_other(&mut self, _code: &String, _span: &Span, _trivia: &Trivia) -> () {}
}

impl JsStmtVisitor<()> for DefaultVisitor {
    fn visit_expr(&mut self, expr: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        expr.accept(self);
    }
    fn visit_variable_decl(&mut self, _kind: &String, _id: &String, init: &Option<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        if let Some(expr) = init {
            expr.accept(self);
        }
    }
    fn visit_import(&mut self, _source: &String, _specifiers: &Vec<String>, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_export(&mut self, declaration: &JsStmt, _span: &Span, _trivia: &Trivia) -> () {
        declaration.accept(self);
    }
    fn visit_export_all(&mut self, _source: &String, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_export_named(&mut self, _source: &Option<String>, _specifiers: &Vec<String>, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_function_decl(&mut self, _id: &String, _params: &Vec<String>, body: &Vec<JsStmt>, _is_async: bool, _span: &Span, _trivia: &Trivia) -> () {
        for stmt in body {
            stmt.accept(self);
        }
    }
    fn visit_return(&mut self, expr: &Option<JsExpr>, _span: &Span, _trivia: &Trivia) -> () {
        if let Some(expr) = expr {
            expr.accept(self);
        }
    }
    fn visit_if(&mut self, test: &JsExpr, consequent: &JsStmt, alternate: &Option<Box<JsStmt>>, _span: &Span, _trivia: &Trivia) -> () {
        test.accept(self);
        consequent.accept(self);
        if let Some(alt) = alternate {
            alt.accept(self);
        }
    }
    fn visit_while(&mut self, test: &JsExpr, body: &JsStmt, _span: &Span, _trivia: &Trivia) -> () {
        test.accept(self);
        body.accept(self);
    }
    fn visit_for(&mut self, init: &Option<Box<JsStmt>>, test: &Option<JsExpr>, update: &Option<JsExpr>, body: &JsStmt, _span: &Span, _trivia: &Trivia) -> () {
        if let Some(init_stmt) = init {
            init_stmt.accept(self);
        }
        if let Some(test_expr) = test {
            test_expr.accept(self);
        }
        if let Some(update_expr) = update {
            update_expr.accept(self);
        }
        body.accept(self);
    }
    fn visit_for_in(&mut self, left: &JsStmt, right: &JsExpr, body: &JsStmt, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
        body.accept(self);
    }
    fn visit_for_of(&mut self, left: &JsStmt, right: &JsExpr, body: &JsStmt, _span: &Span, _trivia: &Trivia) -> () {
        left.accept(self);
        right.accept(self);
        body.accept(self);
    }
    fn visit_try(&mut self, block: &JsStmt, handler: &Option<(String, Box<JsStmt>)>, finalizer: &Option<Box<JsStmt>>, _span: &Span, _trivia: &Trivia) -> () {
        block.accept(self);
        if let Some((_id, body)) = handler {
            body.accept(self);
        }
        if let Some(body) = finalizer {
            body.accept(self);
        }
    }
    fn visit_switch(&mut self, discriminant: &JsExpr, cases: &Vec<(Option<JsExpr>, Vec<JsStmt>)>, _span: &Span, _trivia: &Trivia) -> () {
        discriminant.accept(self);
        for (test, stmts) in cases {
            if let Some(test_expr) = test {
                test_expr.accept(self);
            }
            for stmt in stmts {
                stmt.accept(self);
            }
        }
    }
    fn visit_throw(&mut self, expr: &JsExpr, _span: &Span, _trivia: &Trivia) -> () {
        expr.accept(self);
    }
    fn visit_block(&mut self, stmts: &Vec<JsStmt>, _span: &Span, _trivia: &Trivia) -> () {
        for stmt in stmts {
            stmt.accept(self);
        }
    }
    fn visit_break(&mut self, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_continue(&mut self, _span: &Span, _trivia: &Trivia) -> () {}
    fn visit_other(&mut self, _code: &String, _span: &Span, _trivia: &Trivia) -> () {}
}

impl TemplateNodeVisitor<()> for DefaultVisitor {
    fn visit_element(&mut self, element: &ElementIR, depth: usize) -> () {
        for child in &element.children {
            child.accept(self, depth + 1);
        }
    }
    fn visit_if(&mut self, if_node: &IfNodeIR, depth: usize) -> () {
        for node in &if_node.consequent {
            node.accept(self, depth + 1);
        }
        if let Some(alternate) = &if_node.alternate {
            for node in alternate {
                node.accept(self, depth + 1);
            }
        }
        for (_condition, body) in &if_node.else_ifs {
            for node in body {
                node.accept(self, depth + 1);
            }
        }
    }
    fn visit_for(&mut self, for_node: &ForNodeIR, depth: usize) -> () {
        for node in &for_node.body {
            node.accept(self, depth + 1);
        }
    }
    fn visit_text(&mut self, _text: &String, _span: &Span, _trivia: &Trivia, _depth: usize) -> () {}
    fn visit_interpolation(&mut self, _expr: &ExpressionIR, _depth: usize) -> () {}
    fn visit_comment(&mut self, _comment: &String, _span: &Span, _trivia: &Trivia, _depth: usize) -> () {}
    fn visit_hoisted(&mut self, _key: &String, _depth: usize) -> () {}
}
