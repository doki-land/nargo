use crate::{ParseState, TemplateParser};
use nargo_ir::{TemplateNodeIR, ElementIR, AttributeIR, ExpressionIR, Comment, ForNodeIR, ForIteratorIR, IfNodeIR, Trivia};
use nargo_types::{Result, is_void_element, Span};
use oak_core::{SourceText, parser::ParseSession, Parser, tree::RedTree};
use oak_svelte::{SvelteParser, SvelteLanguage, SvelteElementType};

/// 基于 oak-svelte 的 Svelte 模板解析器
pub struct OakSvelteParser;

impl TemplateParser for OakSvelteParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        let source = SourceText::new(state.cursor.source.to_string());
        let language = SvelteLanguage::default();
        let parser = SvelteParser::new(&language);
        let mut session = ParseSession::default();
        let parse_output = parser.parse(&source, &[], &mut session);
        
        // 将 green 树转换为 red 树，获取包含位置信息的 AST
        let red_tree = RedTree::new(parse_output.result);
        
        // 转换 oak-svelte 的解析结果为项目的 IR 格式
        let nodes = self.convert_to_ir(&red_tree, &source);
        Ok(nodes)
    }
}

impl OakSvelteParser {
    /// 将 oak-svelte 的解析结果转换为项目的 IR 格式
    fn convert_to_ir(&self, tree: &RedTree<'_, SvelteLanguage>, source: &SourceText) -> Vec<TemplateNodeIR> {
        let mut nodes = Vec::new();
        self.convert_node(tree, source, &mut nodes);
        nodes
    }
    
    /// 转换单个节点
    fn convert_node(&self, tree: &RedTree<'_, SvelteLanguage>, source: &SourceText, result: &mut Vec<TemplateNodeIR>) {
        match tree {
            RedTree::Node(node) => {
                match node.element_type() {
                    SvelteElementType::Root => {
                        for child in node.children() {
                            self.convert_node(&child, source, result);
                        }
                    },
                    SvelteElementType::Element => {
                        let mut tag = String::new();
                        let mut attributes = Vec::new();
                        let mut children = Vec::new();
                        let mut trivia = Trivia::default();
                        
                        for child in node.children() {
                            match &child {
                                RedTree::Node(child_node) => {
                                    match child_node.element_type() {
                                        SvelteElementType::Tag => {
                                            // 解析标签名
                                            tag = self.get_node_text(child_node, source);
                                        },
                                        SvelteElementType::Attribute | SvelteElementType::Directive => {
                                            let attribute = self.convert_attribute(&child, source);
                                            attributes.push(attribute);
                                        },
                                        SvelteElementType::Element => {
                                            let mut child_nodes = Vec::new();
                                            self.convert_node(&child, source, &mut child_nodes);
                                            children.extend(child_nodes);
                                        },
                                        _ => {}
                                    }
                                },
                                RedTree::Leaf(leaf) => {
                                    // 处理文本节点和其他叶子节点
                                    let text = leaf.text(source).to_string();
                                    if !text.trim().is_empty() {
                                        let span = Span {
                                            start: leaf.span().start as u32,
                                            end: leaf.span().end as u32,
                                        };
                                        let text_node = TemplateNodeIR::Text(
                                            text,
                                            span,
                                            Trivia::default(),
                                        );
                                        result.push(text_node);
                                    }
                                }
                            }
                        }
                        
                        let span = Span {
                            start: node.span().start as u32,
                            end: node.span().end as u32,
                        };
                        let element = TemplateNodeIR::Element(ElementIR {
                            tag,
                            attributes,
                            children,
                            is_static: false,
                            span,
                            trivia,
                        });
                        result.push(element);
                    },
                    _ => {}
                }
            },
            RedTree::Leaf(_) => {
                // 叶子节点已经在父节点中处理
            }
        }
    }
    
    /// 转换属性
    fn convert_attribute(&self, tree: &RedTree<'_, SvelteLanguage>, source: &SourceText) -> AttributeIR {
        let mut name = String::new();
        let mut value = None;
        let mut is_directive = false;
        let mut is_dynamic = false;
        let mut trivia = Trivia::default();
        
        if let RedTree::Node(node) = tree {
            for child in node.children() {
                if let RedTree::Node(child_node) = &child {
                    match child_node.element_type() {
                        SvelteElementType::AttributeName => {
                            name = self.get_node_text(child_node, source);
                        },
                        SvelteElementType::AttributeValue => {
                            let text = self.get_node_text(child_node, source);
                            value = Some(text);
                        },
                        SvelteElementType::Directive => {
                            is_directive = true;
                        },
                        _ => {}
                    }
                }
            }
        }
        
        let span = if let RedTree::Node(node) = tree {
            Span {
                start: node.span().start as u32,
                end: node.span().end as u32,
            }
        } else {
            Span { start: 0, end: 0 }
        };
        
        AttributeIR {
            name,
            value,
            value_ast: None,
            argument: None,
            modifiers: Vec::new(),
            is_directive,
            is_dynamic,
            span,
            trivia,
        }
    }
    
    /// 获取节点的文本内容
    fn get_node_text(&self, node: &oak_core::tree::RedNode<'_, SvelteLanguage>, source: &SourceText) -> String {
        node.text(source).to_string()
    }
}