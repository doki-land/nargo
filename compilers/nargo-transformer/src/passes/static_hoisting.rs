use nargo_ir::*;
use nargo_types::Result;
use std::collections::HashMap;

/// 静态提升优化变换插件
pub struct StaticHoistingPass {
    hoisted_count: usize,
    /// 记录脚本中的常量绑定 (const x = 123)
    const_bindings: HashMap<String, JsExpr>,
}

impl StaticHoistingPass {
    /// 创建新的静态提升优化变换插件
    pub fn new() -> Self {
        Self { hoisted_count: 0, const_bindings: HashMap::new() }
    }

    /// 分析脚本中的常量
    fn collect_const_bindings(&mut self, script: &JsProgram) {
        // 预分配容量，减少内存分配
        self.const_bindings.reserve(script.body.len());

        for stmt in &script.body {
            if let JsStmt::VariableDecl { kind, id, init: Some(init), .. } = stmt {
                if kind == "const" {
                    // 收集字面量、数组字面量、对象字面量（递归检查是否为纯静态）
                    if self.is_expr_static(init) {
                        self.const_bindings.insert(id.clone(), init.clone());
                    }
                }
            }
        }
    }

    /// 递归检查表达式是否为静态（由常量组成）
    fn is_expr_static(&self, expr: &JsExpr) -> bool {
        match expr {
            JsExpr::Literal(_, _, _) => true,
            JsExpr::Array(elements, _, _) => elements.iter().all(|e| self.is_expr_static(e)),
            JsExpr::Object(props, _, _) => props.values().all(|v| self.is_expr_static(v)),
            JsExpr::Identifier(name, _, _) => self.const_bindings.contains_key(name),
            JsExpr::Member { object, property, .. } => self.is_expr_static(object) && self.is_expr_static(property),
            JsExpr::Binary { left, right, .. } => self.is_expr_static(left) && self.is_expr_static(right),
            JsExpr::Unary { argument, .. } => self.is_expr_static(argument),
            JsExpr::Conditional { test, consequent, alternate, .. } => self.is_expr_static(test) && self.is_expr_static(consequent) && self.is_expr_static(alternate),
            JsExpr::TemplateLiteral { expressions, .. } => expressions.iter().all(|e| self.is_expr_static(e)),
            JsExpr::TseElement { attributes, children, .. } => attributes.iter().all(|a| a.value.as_ref().map(|v| self.is_expr_static(v)).unwrap_or(true)) && children.iter().all(|c| self.is_expr_static(c)),
            _ => false,
        }
    }

    fn mark_static(&self, node: &mut TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                for child in &mut el.children {
                    self.mark_static(child);
                }

                // 检查属性是否都是静态的
                let has_dynamic_attr = el.attributes.iter().any(|a| {
                    if !a.is_dynamic {
                        return false;
                    }
                    // 如果是动态属性，检查它引用的表达式是否为静态
                    if let Some(ast) = &a.value_ast {
                        if self.is_expr_static(ast) {
                            return false; // 引用的是静态表达式
                        }
                    }
                    true
                });

                let all_children_static = el.children.iter().all(|c| match c {
                    TemplateNodeIR::Text(_, _, _) => true,
                    TemplateNodeIR::Element(child_el) => child_el.is_static,
                    TemplateNodeIR::Comment(_, _, _) => true,
                    TemplateNodeIR::Interpolation(interp) => {
                        // 检查插值表达式是否为静态
                        if let Some(ast) = &interp.ast { self.is_expr_static(ast) } else { false }
                    }
                    TemplateNodeIR::Hoisted(_) => true,
                    TemplateNodeIR::If(_) | TemplateNodeIR::For(_) => false,
                });

                el.is_static = !has_dynamic_attr && all_children_static;
            }
            TemplateNodeIR::If(if_node) => {
                for child in &mut if_node.consequent {
                    self.mark_static(child);
                }
                if let Some(alt) = &mut if_node.alternate {
                    for child in alt {
                        self.mark_static(child);
                    }
                }
                for (_, nodes) in &mut if_node.else_ifs {
                    for node in nodes {
                        self.mark_static(node);
                    }
                }
            }
            TemplateNodeIR::For(for_node) => {
                for child in &mut for_node.body {
                    self.mark_static(child);
                }
            }
            TemplateNodeIR::Text(_, _, _) | TemplateNodeIR::Comment(_, _, _) | TemplateNodeIR::Hoisted(_) => {
                // 这些节点本身就是静态的
            }
            TemplateNodeIR::Interpolation(interp) => {
                // 如果插值引用的是常量，可以标记为静态以便后续优化
                if let Some(ast) = &interp.ast {
                    interp.is_static = self.is_expr_static(ast);
                }
            }
        }
    }

    fn perform_hoisting(&mut self, node: &mut TemplateNodeIR, hoisted_nodes: &mut HashMap<String, TemplateNodeIR>) {
        let should_hoist = match node {
            TemplateNodeIR::Element(el) => el.is_static,
            TemplateNodeIR::Text(_, _, _) => true, // 纯文本也可以提升，但通常开销较小
            TemplateNodeIR::Interpolation(interp) => interp.is_static,
            _ => false,
        };

        if should_hoist {
            let id = format!("_hoisted_{}", self.hoisted_count);
            self.hoisted_count += 1;

            let original_node = std::mem::replace(node, TemplateNodeIR::Hoisted(id.clone()));
            hoisted_nodes.insert(id, original_node);
        }
        else {
            match node {
                TemplateNodeIR::Element(el) => {
                    for child in &mut el.children {
                        self.perform_hoisting(child, hoisted_nodes);
                    }
                }
                TemplateNodeIR::If(if_node) => {
                    for child in &mut if_node.consequent {
                        self.perform_hoisting(child, hoisted_nodes);
                    }
                    if let Some(alt) = &mut if_node.alternate {
                        for child in alt {
                            self.perform_hoisting(child, hoisted_nodes);
                        }
                    }
                    for (_, nodes) in &mut if_node.else_ifs {
                        for node in nodes {
                            self.perform_hoisting(node, hoisted_nodes);
                        }
                    }
                }
                TemplateNodeIR::For(for_node) => {
                    for child in &mut for_node.body {
                        self.perform_hoisting(child, hoisted_nodes);
                    }
                }
                _ => {}
            }
        }
    }
}

impl crate::TransformPass for StaticHoistingPass {
    fn name(&self) -> String {
        "StaticHoistingPass".to_string()
    }

    fn description(&self) -> String {
        "Marks and extracts static template nodes for global hoisting".to_string()
    }

    fn transform(&mut self, ir: &mut IRModule) -> Result<()> {
        // 0. 收集常量信息
        if let Some(script) = &ir.script {
            self.collect_const_bindings(script);
        }

        if let Some(template) = &mut ir.template {
            // 1. 标记静态节点
            for node in &mut template.nodes {
                self.mark_static(node);
            }

            // 2. 执行提升
            let mut hoisted = HashMap::new();
            for node in &mut template.nodes {
                self.perform_hoisting(node, &mut hoisted);
            }
            ir.hoisted_nodes.extend(hoisted);
        }
        Ok(())
    }
}
