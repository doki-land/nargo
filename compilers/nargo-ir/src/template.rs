#![warn(missing_docs)]

//! 模板模块
//!
//! 提供 HXO 模板的 IR 表示和相关功能。

use nargo_types::{Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    expr::JsExpr,
    types::{IRError, MAX_ARRAY_LENGTH, MAX_RECURSION_DEPTH, MAX_STRING_LENGTH, Trivia},
};

/// 测试 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestIR {
    /// 测试名称
    pub name: String,
    /// 测试体
    pub body: crate::program::JsProgram,
    /// 位置信息
    pub span: Span,
}

impl TestIR {
    /// 验证测试的有效性
    pub fn validate(&self) -> Result<()> {
        if self.name.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Test name length exceeded".to_string()).into());
        }
        self.body.validate()
    }
}

/// 模板 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TemplateIR {
    /// 模板节点
    pub nodes: Vec<TemplateNodeIR>,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
}

impl TemplateIR {
    /// 验证模板的有效性
    pub fn validate(&self) -> Result<()> {
        if self.nodes.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Template nodes length exceeded".to_string()).into());
        }
        for node in &self.nodes {
            node.validate(0)?;
        }
        Ok(())
    }
}

/// 模板节点 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateNodeIR {
    /// 元素
    Element(ElementIR),
    /// if 节点
    If(IfNodeIR),
    /// for 节点
    For(ForNodeIR),
    /// 文本
    Text(String, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 插值
    Interpolation(ExpressionIR),
    /// 注释
    Comment(String, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 提升节点
    Hoisted(String),
}

impl TemplateNodeIR {
    /// 验证模板节点的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Template node recursion depth exceeded".to_string()).into());
        }

        match self {
            TemplateNodeIR::Element(element) => element.validate(depth + 1),
            TemplateNodeIR::If(if_node) => if_node.validate(depth + 1),
            TemplateNodeIR::For(for_node) => for_node.validate(depth + 1),
            TemplateNodeIR::Text(text, _, _) => {
                if text.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Text length exceeded".to_string()).into());
                }
                Ok(())
            }
            TemplateNodeIR::Interpolation(expr) => expr.validate(depth + 1),
            TemplateNodeIR::Comment(comment, _, _) => {
                if comment.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Comment length exceeded".to_string()).into());
                }
                Ok(())
            }
            TemplateNodeIR::Hoisted(key) => {
                if key.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Hoisted key length exceeded".to_string()).into());
                }
                Ok(())
            }
        }
    }
}

/// if 节点 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IfNodeIR {
    /// 条件
    pub condition: ExpressionIR,
    /// 真分支
    pub consequent: Vec<TemplateNodeIR>,
    /// 假分支（else 或 else if）
    pub alternate: Option<Vec<TemplateNodeIR>>,
    /// else if 分支
    pub else_ifs: Vec<(ExpressionIR, Vec<TemplateNodeIR>)>,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
}

impl IfNodeIR {
    /// 验证 if 节点的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("If node recursion depth exceeded".to_string()).into());
        }

        self.condition.validate(depth + 1)?;

        if self.consequent.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("If node consequent length exceeded".to_string()).into());
        }
        for node in &self.consequent {
            node.validate(depth + 1)?;
        }

        if let Some(alternate) = &self.alternate {
            if alternate.len() > MAX_ARRAY_LENGTH {
                return Err(IRError::SizeLimitExceeded("If node alternate length exceeded".to_string()).into());
            }
            for node in alternate {
                node.validate(depth + 1)?;
            }
        }

        if self.else_ifs.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("If node else ifs length exceeded".to_string()).into());
        }
        for (condition, body) in &self.else_ifs {
            condition.validate(depth + 1)?;
            if body.len() > MAX_ARRAY_LENGTH {
                return Err(IRError::SizeLimitExceeded("If node else if body length exceeded".to_string()).into());
            }
            for node in body {
                node.validate(depth + 1)?;
            }
        }

        Ok(())
    }
}

/// for 节点 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ForNodeIR {
    /// 迭代器
    pub iterator: ForIteratorIR,
    /// 循环体
    pub body: Vec<TemplateNodeIR>,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
}

impl ForNodeIR {
    /// 验证 for 节点的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("For node recursion depth exceeded".to_string()).into());
        }

        self.iterator.validate(depth + 1)?;

        if self.body.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("For node body length exceeded".to_string()).into());
        }
        for node in &self.body {
            node.validate(depth + 1)?;
        }

        Ok(())
    }
}

/// for 迭代器 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ForIteratorIR {
    /// 项目
    pub item: String,
    /// 索引
    pub index: Option<String>,
    /// 集合
    pub collection: ExpressionIR,
}

impl ForIteratorIR {
    /// 验证 for 迭代器的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("For iterator recursion depth exceeded".to_string()).into());
        }

        if self.item.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("For iterator item length exceeded".to_string()).into());
        }

        if let Some(index) = &self.index {
            if index.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("For iterator index length exceeded".to_string()).into());
            }
        }

        self.collection.validate(depth + 1)
    }
}

/// 元素 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ElementIR {
    /// 标签名
    pub tag: String,
    /// 属性列表
    pub attributes: Vec<AttributeIR>,
    /// 子节点
    pub children: Vec<TemplateNodeIR>,
    /// 是否为静态元素
    pub is_static: bool,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl ElementIR {
    /// 验证元素的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Element recursion depth exceeded".to_string()).into());
        }

        if self.tag.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Element tag length exceeded".to_string()).into());
        }

        if self.attributes.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Element attributes length exceeded".to_string()).into());
        }
        for attr in &self.attributes {
            attr.validate(depth + 1)?;
        }

        if self.children.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Element children length exceeded".to_string()).into());
        }
        for child in &self.children {
            child.validate(depth + 1)?;
        }

        Ok(())
    }
}

/// 属性 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeIR {
    /// 属性名
    pub name: String,
    /// 属性值
    pub value: Option<String>,
    /// 属性值的 AST
    pub value_ast: Option<JsExpr>,
    /// 参数
    pub argument: Option<String>,
    /// 修饰符
    pub modifiers: Vec<String>,
    /// 是否为指令
    pub is_directive: bool,
    /// 是否为动态属性
    pub is_dynamic: bool,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl AttributeIR {
    /// 验证属性的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Attribute recursion depth exceeded".to_string()).into());
        }

        if self.name.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Attribute name length exceeded".to_string()).into());
        }

        if let Some(value) = &self.value {
            if value.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Attribute value length exceeded".to_string()).into());
            }
        }

        if let Some(value_ast) = &self.value_ast {
            value_ast.validate(depth + 1)?;
        }

        if let Some(argument) = &self.argument {
            if argument.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Attribute argument length exceeded".to_string()).into());
            }
        }

        if self.modifiers.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Attribute modifiers length exceeded".to_string()).into());
        }
        for modifier in &self.modifiers {
            if modifier.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Attribute modifier length exceeded".to_string()).into());
            }
        }

        Ok(())
    }
}

/// 表达式 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExpressionIR {
    /// 表达式代码
    pub code: String,
    /// 表达式 AST
    pub ast: Option<JsExpr>,
    /// 是否为静态表达式
    pub is_static: bool,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl ExpressionIR {
    /// 验证表达式的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Expression IR recursion depth exceeded".to_string()).into());
        }

        if self.code.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Expression code length exceeded".to_string()).into());
        }

        if let Some(ast) = &self.ast {
            ast.validate(depth + 1)?;
        }

        Ok(())
    }

    /// 检查表达式是否为空
    pub fn is_empty(&self) -> bool {
        self.code.is_empty() && self.ast.is_none() && self.trivia.is_empty()
    }
}

/// 自定义块 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomBlockIR {
    /// 块名称
    pub name: String,
    /// 块内容
    pub content: String,
    /// 块属性
    pub attributes: HashMap<String, String>,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl CustomBlockIR {
    /// 验证自定义块的有效性
    pub fn validate(&self) -> Result<()> {
        if self.name.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Custom block name length exceeded".to_string()).into());
        }

        if self.content.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Custom block content length exceeded".to_string()).into());
        }

        if self.attributes.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Custom block attributes size exceeded".to_string()).into());
        }
        for (key, value) in &self.attributes {
            if key.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Custom block attribute key length exceeded".to_string()).into());
            }
            if value.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Custom block attribute value length exceeded".to_string()).into());
            }
        }

        Ok(())
    }

    /// 检查自定义块是否为空
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.content.is_empty() && self.attributes.is_empty() && self.trivia.is_empty()
    }
}

/// 样式 IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StyleIR {
    /// 样式代码
    pub code: String,
    /// 样式语言
    pub lang: String,
    /// 是否为作用域样式
    pub scoped: bool,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl StyleIR {
    /// 验证样式的有效性
    pub fn validate(&self) -> Result<()> {
        if self.code.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Style code length exceeded".to_string()).into());
        }

        if self.lang.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Style lang length exceeded".to_string()).into());
        }

        Ok(())
    }

    /// 检查样式是否为空
    pub fn is_empty(&self) -> bool {
        self.code.is_empty() && self.lang.is_empty() && self.trivia.is_empty()
    }
}

/// 模板节点访问者 trait
pub trait TemplateNodeVisitor<R> {
    /// 访问元素节点
    fn visit_element(&mut self, element: &ElementIR, depth: usize) -> R;
    /// 访问 if 节点
    fn visit_if(&mut self, if_node: &IfNodeIR, depth: usize) -> R;
    /// 访问 for 节点
    fn visit_for(&mut self, for_node: &ForNodeIR, depth: usize) -> R;
    /// 访问文本节点
    fn visit_text(&mut self, text: &String, span: &Span, trivia: &Trivia, depth: usize) -> R;
    /// 访问插值节点
    fn visit_interpolation(&mut self, expr: &ExpressionIR, depth: usize) -> R;
    /// 访问注释节点
    fn visit_comment(&mut self, comment: &String, span: &Span, trivia: &Trivia, depth: usize) -> R;
    /// 访问提升节点
    fn visit_hoisted(&mut self, key: &String, depth: usize) -> R;
}

impl TemplateNodeIR {
    /// 接受访问者
    pub fn accept<R>(&self, visitor: &mut dyn TemplateNodeVisitor<R>, depth: usize) -> R {
        match self {
            TemplateNodeIR::Element(element) => visitor.visit_element(element, depth),
            TemplateNodeIR::If(if_node) => visitor.visit_if(if_node, depth),
            TemplateNodeIR::For(for_node) => visitor.visit_for(for_node, depth),
            TemplateNodeIR::Text(text, span, trivia) => visitor.visit_text(text, span, trivia, depth),
            TemplateNodeIR::Interpolation(expr) => visitor.visit_interpolation(expr, depth),
            TemplateNodeIR::Comment(comment, span, trivia) => visitor.visit_comment(comment, span, trivia, depth),
            TemplateNodeIR::Hoisted(key) => visitor.visit_hoisted(key, depth),
        }
    }
}
