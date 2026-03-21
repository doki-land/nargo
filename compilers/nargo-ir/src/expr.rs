#![warn(missing_docs)]

//! 表达式模块
//!
//! 提供 JavaScript 表达式的 IR 表示和相关功能。

use nargo_types::{NargoValue, Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{IRError, MAX_ARRAY_LENGTH, MAX_OBJECT_SIZE, MAX_RECURSION_DEPTH, MAX_STRING_LENGTH, Trivia};

/// TSE 属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TseAttribute {
    /// 属性名
    pub name: String,
    /// 属性值
    pub value: Option<JsExpr>,
    /// 是否为指令
    pub is_directive: bool,
    /// 位置信息
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl TseAttribute {
    /// 验证 TSE 属性的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if self.name.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("TSE attribute name length exceeded".to_string()).into());
        }
        if let Some(value) = &self.value {
            value.validate(depth + 1)?;
        }
        Ok(())
    }
}

/// JavaScript 表达式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsExpr {
    /// 标识符
    Identifier(String, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 字面量
    Literal(NargoValue, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 一元表达式
    Unary {
        /// 操作符
        op: String,
        /// 参数
        argument: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 二元表达式
    Binary {
        /// 左操作数
        left: Box<JsExpr>,
        /// 操作符
        op: String,
        /// 右操作数
        right: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 函数调用
    Call {
        /// 调用目标
        callee: Box<JsExpr>,
        /// 参数列表
        args: Vec<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 成员访问
    Member {
        /// 对象
        object: Box<JsExpr>,
        /// 属性
        property: Box<JsExpr>,
        /// 是否为计算属性
        computed: bool,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 可选链成员访问
    OptionalMember {
        /// 对象
        object: Box<JsExpr>,
        /// 属性
        property: Box<JsExpr>,
        /// 是否为计算属性
        computed: bool,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 可选链函数调用
    OptionalCall {
        /// 调用目标
        callee: Box<JsExpr>,
        /// 参数列表
        args: Vec<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 空值合并表达式
    NullishCoalescing {
        /// 左操作数
        left: Box<JsExpr>,
        /// 右操作数
        right: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 逻辑赋值表达式
    LogicalAssignment {
        /// 操作符
        op: String,
        /// 左操作数
        left: Box<JsExpr>,
        /// 右操作数
        right: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 数组字面量
    Array(Vec<JsExpr>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 对象字面量
    Object(HashMap<String, JsExpr>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 箭头函数
    ArrowFunction {
        /// 参数列表
        params: Vec<String>,
        /// 函数体
        body: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// TSE 元素
    TseElement {
        /// 标签名
        tag: String,
        /// 属性列表
        attributes: Vec<TseAttribute>,
        /// 子元素
        children: Vec<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 条件表达式
    Conditional {
        /// 条件
        test: Box<JsExpr>,
        /// 真分支
        consequent: Box<JsExpr>,
        /// 假分支
        alternate: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 模板字面量
    TemplateLiteral {
        /// 模板字符串片段
        quasis: Vec<String>,
        /// 表达式
        expressions: Vec<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 展开表达式
    Spread(Box<JsExpr>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 类型测试表达式
    TypeOf(Box<JsExpr>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 实例测试表达式
    InstanceOf {
        /// 左操作数
        left: Box<JsExpr>,
        /// 右操作数
        right: Box<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 其他表达式
    Other(String, #[serde(default)] Span, #[serde(default)] Trivia),
}

impl JsExpr {
    /// 获取表达式的位置信息
    pub fn span(&self) -> Span {
        match self {
            JsExpr::Identifier(_, span, _) => *span,
            JsExpr::Literal(_, span, _) => *span,
            JsExpr::Unary { span, .. } => *span,
            JsExpr::Binary { span, .. } => *span,
            JsExpr::Call { span, .. } => *span,
            JsExpr::Member { span, .. } => *span,
            JsExpr::OptionalMember { span, .. } => *span,
            JsExpr::OptionalCall { span, .. } => *span,
            JsExpr::NullishCoalescing { span, .. } => *span,
            JsExpr::LogicalAssignment { span, .. } => *span,
            JsExpr::Array(_, span, _) => *span,
            JsExpr::Object(_, span, _) => *span,
            JsExpr::ArrowFunction { span, .. } => *span,
            JsExpr::TseElement { span, .. } => *span,
            JsExpr::Conditional { span, .. } => *span,
            JsExpr::TemplateLiteral { span, .. } => *span,
            JsExpr::Spread(_, span, _) => *span,
            JsExpr::TypeOf(_, span, _) => *span,
            JsExpr::InstanceOf { span, .. } => *span,
            JsExpr::Other(_, span, _) => *span,
        }
    }

    /// 获取表达式的 trivia 信息
    pub fn trivia(&self) -> &Trivia {
        match self {
            JsExpr::Identifier(_, _, t) => t,
            JsExpr::Literal(_, _, t) => t,
            JsExpr::Unary { trivia, .. } => trivia,
            JsExpr::Binary { trivia, .. } => trivia,
            JsExpr::Call { trivia, .. } => trivia,
            JsExpr::Member { trivia, .. } => trivia,
            JsExpr::OptionalMember { trivia, .. } => trivia,
            JsExpr::OptionalCall { trivia, .. } => trivia,
            JsExpr::NullishCoalescing { trivia, .. } => trivia,
            JsExpr::LogicalAssignment { trivia, .. } => trivia,
            JsExpr::Array(_, _, t) => t,
            JsExpr::Object(_, _, t) => t,
            JsExpr::ArrowFunction { trivia, .. } => trivia,
            JsExpr::TseElement { trivia, .. } => trivia,
            JsExpr::Conditional { trivia, .. } => trivia,
            JsExpr::TemplateLiteral { trivia, .. } => trivia,
            JsExpr::Spread(_, _, t) => t,
            JsExpr::TypeOf(_, _, t) => t,
            JsExpr::InstanceOf { trivia, .. } => trivia,
            JsExpr::Other(_, _, t) => t,
        }
    }

    /// 验证表达式的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Expression recursion depth exceeded".to_string()).into());
        }

        match self {
            JsExpr::Identifier(id, _, _) => {
                if id.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Identifier length exceeded".to_string()).into());
                }
                Ok(())
            }
            JsExpr::Literal(value, _, _) => value.validate(depth + 1),
            JsExpr::Unary { argument, .. } => argument.validate(depth + 1),
            JsExpr::Binary { left, right, .. } => {
                left.validate(depth + 1)?;
                right.validate(depth + 1)
            }
            JsExpr::Call { callee, args, .. } => {
                callee.validate(depth + 1)?;
                if args.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Call arguments length exceeded".to_string()).into());
                }
                for arg in args {
                    arg.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::Member { object, property, .. } => {
                object.validate(depth + 1)?;
                property.validate(depth + 1)
            }
            JsExpr::OptionalMember { object, property, .. } => {
                object.validate(depth + 1)?;
                property.validate(depth + 1)
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                callee.validate(depth + 1)?;
                if args.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Optional call arguments length exceeded".to_string()).into());
                }
                for arg in args {
                    arg.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                left.validate(depth + 1)?;
                right.validate(depth + 1)
            }
            JsExpr::LogicalAssignment { op, left, right, .. } => {
                if op.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Logical assignment operator length exceeded".to_string()).into());
                }
                left.validate(depth + 1)?;
                right.validate(depth + 1)
            }
            JsExpr::Array(items, _, _) => {
                if items.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Array length exceeded".to_string()).into());
                }
                for item in items {
                    item.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::Object(props, _, _) => {
                if props.len() > MAX_OBJECT_SIZE {
                    return Err(IRError::SizeLimitExceeded("Object size exceeded".to_string()).into());
                }
                for (key, value) in props {
                    if key.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Object key length exceeded".to_string()).into());
                    }
                    value.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::ArrowFunction { params, body, .. } => {
                if params.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Arrow function parameters length exceeded".to_string()).into());
                }
                for param in params {
                    if param.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Parameter name length exceeded".to_string()).into());
                    }
                }
                body.validate(depth + 1)
            }
            JsExpr::TseElement { tag, attributes, children, .. } => {
                if tag.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("TSE element tag length exceeded".to_string()).into());
                }
                if attributes.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("TSE element attributes length exceeded".to_string()).into());
                }
                for attr in attributes {
                    attr.validate(depth + 1)?;
                }
                if children.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("TSE element children length exceeded".to_string()).into());
                }
                for child in children {
                    child.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                test.validate(depth + 1)?;
                consequent.validate(depth + 1)?;
                alternate.validate(depth + 1)
            }
            JsExpr::TemplateLiteral { quasis, expressions, .. } => {
                if quasis.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Template literal quasis length exceeded".to_string()).into());
                }
                for quasi in quasis {
                    if quasi.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Template literal quasi length exceeded".to_string()).into());
                    }
                }
                if expressions.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Template literal expressions length exceeded".to_string()).into());
                }
                for expr in expressions {
                    expr.validate(depth + 1)?;
                }
                Ok(())
            }
            JsExpr::Spread(expr, _, _) => expr.validate(depth + 1),
            JsExpr::TypeOf(expr, _, _) => expr.validate(depth + 1),
            JsExpr::InstanceOf { left, right, .. } => {
                left.validate(depth + 1)?;
                right.validate(depth + 1)
            }
            JsExpr::Other(code, _, _) => {
                if code.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Other expression code length exceeded".to_string()).into());
                }
                Ok(())
            }
        }
    }

    /// 优化表达式
    pub fn optimize(&mut self) {
        crate::optimizer::ExprOptimizer::optimize(self);
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self) -> bool {
        match self {
            JsExpr::Literal(_, _, _) => true,
            JsExpr::Unary { argument, .. } => argument.is_constant(),
            JsExpr::Binary { left, right, .. } => left.is_constant() && right.is_constant(),
            _ => false,
        }
    }
}

/// 表达式访问者 trait
pub trait JsExprVisitor<R> {
    /// 访问标识符表达式
    fn visit_identifier(&mut self, id: &String, span: &Span, trivia: &Trivia) -> R;
    /// 访问字面量表达式
    fn visit_literal(&mut self, value: &NargoValue, span: &Span, trivia: &Trivia) -> R;
    /// 访问一元表达式
    fn visit_unary(&mut self, op: &String, argument: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问二元表达式
    fn visit_binary(&mut self, left: &JsExpr, op: &String, right: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问函数调用表达式
    fn visit_call(&mut self, callee: &JsExpr, args: &Vec<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问成员访问表达式
    fn visit_member(&mut self, object: &JsExpr, property: &JsExpr, computed: bool, span: &Span, trivia: &Trivia) -> R;
    /// 访问可选链成员访问表达式
    fn visit_optional_member(&mut self, object: &JsExpr, property: &JsExpr, computed: bool, span: &Span, trivia: &Trivia) -> R;
    /// 访问可选链函数调用表达式
    fn visit_optional_call(&mut self, callee: &JsExpr, args: &Vec<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问空值合并表达式
    fn visit_nullish_coalescing(&mut self, left: &JsExpr, right: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问逻辑赋值表达式
    fn visit_logical_assignment(&mut self, op: &String, left: &JsExpr, right: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问数组字面量表达式
    fn visit_array(&mut self, items: &Vec<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问对象字面量表达式
    fn visit_object(&mut self, props: &HashMap<String, JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问箭头函数表达式
    fn visit_arrow_function(&mut self, params: &Vec<String>, body: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问 TSE 元素表达式
    fn visit_tse_element(&mut self, tag: &String, attributes: &Vec<TseAttribute>, children: &Vec<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问条件表达式
    fn visit_conditional(&mut self, test: &JsExpr, consequent: &JsExpr, alternate: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问模板字面量表达式
    fn visit_template_literal(&mut self, quasis: &Vec<String>, expressions: &Vec<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问展开表达式
    fn visit_spread(&mut self, expr: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问类型测试表达式
    fn visit_type_of(&mut self, expr: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问实例测试表达式
    fn visit_instance_of(&mut self, left: &JsExpr, right: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问其他表达式
    fn visit_other(&mut self, code: &String, span: &Span, trivia: &Trivia) -> R;
}

impl JsExpr {
    /// 接受访问者
    pub fn accept<R>(&self, visitor: &mut dyn JsExprVisitor<R>) -> R {
        match self {
            JsExpr::Identifier(id, span, trivia) => visitor.visit_identifier(id, span, trivia),
            JsExpr::Literal(value, span, trivia) => visitor.visit_literal(value, span, trivia),
            JsExpr::Unary { op, argument, span, trivia } => visitor.visit_unary(op, argument, span, trivia),
            JsExpr::Binary { left, op, right, span, trivia } => visitor.visit_binary(left, op, right, span, trivia),
            JsExpr::Call { callee, args, span, trivia } => visitor.visit_call(callee, args, span, trivia),
            JsExpr::Member { object, property, computed, span, trivia } => visitor.visit_member(object, property, *computed, span, trivia),
            JsExpr::OptionalMember { object, property, computed, span, trivia } => visitor.visit_optional_member(object, property, *computed, span, trivia),
            JsExpr::OptionalCall { callee, args, span, trivia } => visitor.visit_optional_call(callee, args, span, trivia),
            JsExpr::NullishCoalescing { left, right, span, trivia } => visitor.visit_nullish_coalescing(left, right, span, trivia),
            JsExpr::LogicalAssignment { op, left, right, span, trivia } => visitor.visit_logical_assignment(op, left, right, span, trivia),
            JsExpr::Array(items, span, trivia) => visitor.visit_array(items, span, trivia),
            JsExpr::Object(props, span, trivia) => visitor.visit_object(props, span, trivia),
            JsExpr::ArrowFunction { params, body, span, trivia } => visitor.visit_arrow_function(params, body, span, trivia),
            JsExpr::TseElement { tag, attributes, children, span, trivia } => visitor.visit_tse_element(tag, attributes, children, span, trivia),
            JsExpr::Conditional { test, consequent, alternate, span, trivia } => visitor.visit_conditional(test, consequent, alternate, span, trivia),
            JsExpr::TemplateLiteral { quasis, expressions, span, trivia } => visitor.visit_template_literal(quasis, expressions, span, trivia),
            JsExpr::Spread(expr, span, trivia) => visitor.visit_spread(expr, span, trivia),
            JsExpr::TypeOf(expr, span, trivia) => visitor.visit_type_of(expr, span, trivia),
            JsExpr::InstanceOf { left, right, span, trivia } => visitor.visit_instance_of(left, right, span, trivia),
            JsExpr::Other(code, span, trivia) => visitor.visit_other(code, span, trivia),
        }
    }
}
