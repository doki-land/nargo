use nargo_ir::*;
use nargo_types::{NargoValue, Result};
use std::collections::HashMap;

/// 样式提取与分析插件
pub struct StyleAnalysisPass {
    pub collected_styles: Vec<String>,
}

impl StyleAnalysisPass {
    /// 创建新的样式提取与分析插件
    pub fn new() -> Self {
        Self { collected_styles: Vec::new() }
    }

    fn collect_from_stmt(&mut self, stmt: &JsStmt) {
        match stmt {
            JsStmt::Expr(expr, _, _) => self.collect_from_expr(expr),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                self.collect_from_expr(expr);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.collect_from_stmt(declaration);
            }
            _ => {}
        }
    }

    fn collect_from_expr(&mut self, expr: &JsExpr) {
        match expr {
            JsExpr::Literal(NargoValue::String(s), _, _) => {
                // 简单的样式字符串收集 (如 tailwind 类名)
                let s = s.trim_matches(|c| c == '\'' || c == '"');
                for class in s.split_whitespace() {
                    self.collected_styles.push(class.to_string());
                }
            }
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(id, _, _) = &**callee {
                    if id == "addStyle" {
                        for arg in args {
                            self.collect_from_expr(arg);
                        }
                    }
                }
                self.collect_from_expr(callee);
                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.collect_from_expr(left);
                self.collect_from_expr(right);
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                self.collect_from_expr(test);
                self.collect_from_expr(consequent);
                self.collect_from_expr(alternate);
            }
            JsExpr::TemplateLiteral { expressions, quasis, .. } => {
                for expr in expressions {
                    self.collect_from_expr(expr);
                }
                for quasi in quasis {
                    for class in quasi.split_whitespace() {
                        self.collected_styles.push(class.to_string());
                    }
                }
            }
            JsExpr::Array(elements, _, _) => {
                for el in elements {
                    self.collect_from_expr(el);
                }
            }
            JsExpr::Object(props, _, _) => {
                for (key, val) in props {
                    // 在 Vue/Nargo 中，:class="{ 'font-bold': true }" 的类名是 key
                    for class in key.split_whitespace() {
                        self.collected_styles.push(class.to_string());
                    }
                    self.collect_from_expr(val);
                }
            }
            _ => {}
        }
    }

    fn collect_from_template_node(&mut self, node: &TemplateNodeIR, hoisted_nodes: &HashMap<String, TemplateNodeIR>) {
        match node {
            TemplateNodeIR::Element(el) => {
                // 收集 class 属性
                for attr in &el.attributes {
                    if attr.name == "class" {
                        if let Some(val) = &attr.value {
                            for class in val.split_whitespace() {
                                self.collected_styles.push(class.to_string());
                            }
                        }
                        if let Some(ast) = &attr.value_ast {
                            self.collect_from_expr(ast);
                        }
                    }
                    else if attr.name == "bind" && attr.argument.as_deref() == Some("class") {
                        if let Some(ast) = &attr.value_ast {
                            self.collect_from_expr(ast);
                        }
                    }
                }
                // 递归处理子节点
                for child in &el.children {
                    self.collect_from_template_node(child, hoisted_nodes);
                }
            }
            TemplateNodeIR::If(if_node) => {
                for child in &if_node.consequent {
                    self.collect_from_template_node(child, hoisted_nodes);
                }
                for (_, nodes) in &if_node.else_ifs {
                    for node in nodes {
                        self.collect_from_template_node(node, hoisted_nodes);
                    }
                }
                if let Some(alt) = &if_node.alternate {
                    for node in alt {
                        self.collect_from_template_node(node, hoisted_nodes);
                    }
                }
            }
            TemplateNodeIR::For(for_node) => {
                for child in &for_node.body {
                    self.collect_from_template_node(child, hoisted_nodes);
                }
            }
            TemplateNodeIR::Hoisted(id) => {
                if let Some(hoisted_node) = hoisted_nodes.get(id) {
                    self.collect_from_template_node(hoisted_node, hoisted_nodes);
                }
            }
            _ => {}
        }
    }
}

impl crate::TransformPass for StyleAnalysisPass {
    fn name(&self) -> String {
        "StyleAnalysisPass".to_string()
    }

    fn description(&self) -> String {
        "Extracts and collects style definitions from script and template".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        if let Some(script) = &ir.script {
            for stmt in &script.body {
                self.collect_from_stmt(stmt);
            }
        }

        if let Some(template) = &ir.template {
            for node in &template.nodes {
                self.collect_from_template_node(node, &ir.hoisted_nodes);
            }
        }

        for style in &ir.styles {
            self.collected_styles.push(style.code.clone());
        }

        // 去重
        self.collected_styles.sort();
        self.collected_styles.dedup();

        println!("StyleAnalysisPass collected styles: {:?}", self.collected_styles);

        // 将收集到的样式列表存入 metadata
        ir.metadata.insert("collected_styles".to_string(), NargoValue::String(self.collected_styles.join(" ")));

        Ok(())
    }
}
