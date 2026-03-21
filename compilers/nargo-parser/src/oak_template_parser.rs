use crate::{ParseState, TemplateParser};
use nargo_ir::{AttributeIR, ElementIR, TemplateNodeIR, Trivia};
use nargo_types::{Position, Result, Span};
use oak_core::{Parser, SourceText, parser::ParseSession, tree::RedTree};
use oak_vue::{VueElementType, VueLanguage, VueParser};

/// 基于 oak-vue 的 Vue 模板解析器
pub struct OakVueTemplateParser;

impl TemplateParser for OakVueTemplateParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        let source = SourceText::new(state.cursor.source.to_string());
        let language = VueLanguage::default();
        let parser = VueParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);

        // 将 green 树转换为 red 树，获取包含位置信息的 AST
        let green_node = parse_output.result.map_err(|e| nargo_types::Error::parse_error(format!("Vue parsing error: {:?}", e), Span::default()))?;
        let red_tree = RedTree::new(green_node);

        // 转换 oak-vue 的解析结果为项目的 IR 格式
        let nodes = self.convert_to_ir(&red_tree, &source);
        Ok(nodes)
    }
}

impl OakVueTemplateParser {
    /// 将 oak-vue 的解析结果转换为项目的 IR 格式
    fn convert_to_ir(&self, tree: &RedTree<'_, VueLanguage>, source: &SourceText) -> Vec<TemplateNodeIR> {
        let mut nodes = Vec::new();
        self.convert_node(tree, source, &mut nodes);
        nodes
    }

    /// 转换单个节点
    fn convert_node(&self, tree: &RedTree<'_, VueLanguage>, source: &SourceText, result: &mut Vec<TemplateNodeIR>) {
        match tree {
            RedTree::Node(node) => {
                match node.element_type() {
                    VueElementType::Root => {
                        for child in node.children() {
                            self.convert_node(&child, source, result);
                        }
                    }
                    VueElementType::Element => {
                        let mut tag = String::new();
                        let mut attributes = Vec::new();
                        let mut children = Vec::new();
                        let mut is_static = true;
                        let mut trivia = Trivia::default();

                        for child in node.children() {
                            match &child {
                                RedTree::Node(child_node) => {
                                    match child_node.element_type() {
                                        VueElementType::Tag => {
                                            // 解析标签名
                                            tag = self.get_node_text(child_node, source);
                                        }
                                        VueElementType::Attribute | VueElementType::Directive => {
                                            let attribute = self.convert_attribute(&child, source);
                                            // 检查是否为动态属性或指令
                                            if attribute.is_directive || attribute.is_dynamic {
                                                is_static = false;
                                            }
                                            attributes.push(attribute);
                                        }
                                        VueElementType::Element => {
                                            let mut child_nodes = Vec::new();
                                            self.convert_node(&child, source, &mut child_nodes);
                                            // 检查子元素是否为静态
                                            if !child_nodes.iter().all(|n| match n {
                                                TemplateNodeIR::Text(_, _, _) => true,
                                                TemplateNodeIR::Element(el) => el.is_static,
                                                _ => false,
                                            }) {
                                                is_static = false;
                                            }
                                            children.extend(child_nodes);
                                        }
                                        _ => {
                                            // 处理其他节点类型
                                        }
                                    }
                                }
                                RedTree::Leaf(leaf) => {
                                    // 处理文本节点和其他叶子节点
                                    let text = leaf.text(source).to_string();
                                    if !text.trim().is_empty() {
                                        let span = Span { start: Position::new(1, 1, leaf.span().start as u32), end: Position::new(1, 1, leaf.span().end as u32) };
                                        let text_node = TemplateNodeIR::Text(text, span, Trivia::default());
                                        result.push(text_node);
                                    }
                                }
                            }
                        }

                        let span = Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) };
                        let element = TemplateNodeIR::Element(ElementIR { tag, attributes, children, is_static, span, trivia });
                        result.push(element);
                    }
                    _ => {}
                }
            }
            RedTree::Leaf(leaf) => {
                // 处理独立的叶子节点
                let text = leaf.text(source).to_string();
                if !text.trim().is_empty() {
                    let span = Span { start: Position::new(1, 1, leaf.span().start as u32), end: Position::new(1, 1, leaf.span().end as u32) };
                    let text_node = TemplateNodeIR::Text(text, span, Trivia::default());
                    result.push(text_node);
                }
            }
        }
    }

    /// 转换属性
    fn convert_attribute(&self, tree: &RedTree<'_, VueLanguage>, source: &SourceText) -> AttributeIR {
        let mut name = String::new();
        let mut value = None;
        let mut value_ast = None;
        let mut argument = None;
        let mut modifiers = Vec::new();
        let mut is_directive = false;
        let mut is_dynamic = false;
        let mut trivia = Trivia::default();

        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    match child_node.element_type() {
                        VueElementType::AttributeName => {
                            name = self.get_node_text(child_node, source);
                            // 检查是否为动态属性
                            if name.starts_with("v-bind:") || name == "v-bind" || name.starts_with("@") || name == "v-on" {
                                is_dynamic = true;
                            }
                        }
                        VueElementType::AttributeValue => {
                            let text = self.get_node_text(child_node, source);
                            value = Some(text);
                        }
                        VueElementType::Directive => {
                            is_directive = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        let span = if let RedTree::Node(node) = tree { Span { start: Position::new(1, 1, node.span().start as u32), end: Position::new(1, 1, node.span().end as u32) } } else { Span { start: Position::new(1, 1, 0), end: Position::new(1, 1, 0) } };

        AttributeIR { name, value, value_ast, argument, modifiers, is_directive, is_dynamic, span, trivia }
    }

    /// 获取节点的文本内容
    fn get_node_text(&self, node: &oak_core::tree::RedNode<'_, VueLanguage>, source: &SourceText) -> String {
        node.text(source).to_string()
    }
}
