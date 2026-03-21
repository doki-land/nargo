//! Intermediate Representation (IR) types for Nargo.
//!
//! This module provides the IR types used for parsing and compilation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source span for tracking locations in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Span {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start: Position {
                offset: start,
                line: 1,
                column: 0,
            },
            end: Position {
                offset: end,
                line: 1,
                column: 0,
            },
        }
    }

    /// Creates an unknown span.
    pub fn unknown() -> Self {
        Self::default()
    }

    /// Checks if this span is unknown.
    pub fn is_unknown(&self) -> bool {
        self.start.offset == 0 && self.end.offset == 0
    }
}

/// Position in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Position {
    /// Byte offset.
    pub offset: u32,
    /// Line number (1-based).
    pub line: u32,
    /// Column number (0-based).
    pub column: u32,
}

/// Trivia (comments and whitespace).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Trivia {
    /// Leading comments.
    pub leading: Vec<String>,
    /// Trailing comments.
    pub trailing: Vec<String>,
}

/// Comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment content.
    pub content: String,
    /// Whether this is a block comment.
    pub is_block: bool,
}

/// IR Module representing a parsed single-file component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IRModule {
    /// URI of the source file.
    pub uri: String,
    /// Name of the module.
    pub name: String,
    /// Template section.
    pub template: Option<TemplateIR>,
    /// Script section.
    pub script: Option<JsProgram>,
    /// Style sections.
    pub styles: Vec<StyleIR>,
    /// Custom blocks.
    pub custom_blocks: Vec<CustomBlock>,
}

/// Template IR for the template section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateIR {
    /// Template nodes.
    pub nodes: Vec<TemplateNodeIR>,
    /// Span of the template.
    pub span: Span,
}

/// Template node IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateNodeIR {
    /// Element node.
    Element(ElementIR),
    /// Text node.
    Text(String, Span),
    /// Interpolation node.
    Interpolation(ExpressionIR),
    /// Comment node.
    Comment(String, Span),
}

/// Element IR.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementIR {
    /// Tag name.
    pub tag: String,
    /// Attributes.
    pub attributes: Vec<AttributeIR>,
    /// Children.
    pub children: Vec<TemplateNodeIR>,
    /// Span.
    pub span: Span,
    /// Trivia.
    pub trivia: Trivia,
}

/// Attribute IR.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributeIR {
    /// Attribute name.
    pub name: String,
    /// Attribute value.
    pub value: Option<String>,
    /// Parsed value AST.
    pub value_ast: Option<JsExpr>,
    /// Whether this is a directive.
    pub is_directive: bool,
    /// Whether this is dynamic.
    pub is_dynamic: bool,
    /// Span.
    pub span: Span,
}

/// Expression IR for interpolations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpressionIR {
    /// Raw code.
    pub code: String,
    /// Span.
    pub span: Span,
    /// Parsed AST.
    pub ast: Option<JsExpr>,
}

/// Style IR.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleIR {
    /// Style content.
    pub content: String,
    /// Language (css, scss, etc.).
    pub lang: String,
    /// Span.
    pub span: Span,
    /// Whether scoped.
    pub scoped: bool,
    /// Whether module.
    pub module: bool,
}

/// Custom block IR.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomBlock {
    /// Block name.
    pub name: String,
    /// Block content.
    pub content: String,
    /// Block attributes.
    pub attributes: HashMap<String, String>,
}

/// JavaScript program IR.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsProgram {
    /// Statement body.
    pub body: Vec<JsStmt>,
    /// Span.
    pub span: Span,
}

/// JavaScript statement IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JsStmt {
    /// Expression statement.
    Expr(JsExpr, Span, Trivia),
    /// Variable declaration.
    VariableDecl {
        /// Declaration kind (let, const, var).
        kind: String,
        /// Variable name.
        id: String,
        /// Initializer.
        init: Option<JsExpr>,
        /// Span.
        span: Span,
        /// Type annotation.
        type_ann: Option<String>,
    },
    /// Function declaration.
    FunctionDecl {
        /// Function name.
        id: String,
        /// Parameters.
        params: Vec<String>,
        /// Function body.
        body: Vec<JsStmt>,
        /// Span.
        span: Span,
        /// Return type.
        return_type: Option<String>,
        /// Whether async.
        is_async: bool,
        /// Whether generator.
        is_generator: bool,
    },
    /// Import statement.
    Import {
        /// Import specifiers.
        specifiers: Vec<ImportSpecifier>,
        /// Source module.
        source: String,
        /// Span.
        span: Span,
    },
    /// Export statement.
    Export {
        /// Exported declaration.
        decl: Box<JsStmt>,
        /// Span.
        span: Span,
        /// Whether type export.
        is_type: bool,
    },
    /// Export default statement.
    ExportDefault {
        /// Exported declaration.
        decl: Box<JsStmt>,
        /// Span.
        span: Span,
    },
    /// Export all statement.
    ExportAll {
        /// Source module.
        source: String,
        /// Span.
        span: Span,
    },
    /// Export named statement.
    ExportNamed {
        /// Exported names.
        specifiers: Vec<String>,
        /// Source module.
        source: Option<String>,
        /// Span.
        span: Span,
    },
    /// Return statement.
    Return(JsExpr, Span, Trivia),
    /// If statement.
    If {
        /// Condition.
        test: JsExpr,
        /// Consequent branch.
        consequent: Vec<JsStmt>,
        /// Alternate branch.
        alternate: Option<Vec<JsStmt>>,
        /// Span.
        span: Span,
    },
    /// While statement.
    While {
        /// Condition.
        test: JsExpr,
        /// Body.
        body: Vec<JsStmt>,
        /// Span.
        span: Span,
    },
    /// For statement.
    For {
        /// Initializer.
        init: Box<JsStmt>,
        /// Condition.
        test: Option<JsExpr>,
        /// Update.
        update: Option<JsExpr>,
        /// Body.
        body: Vec<JsStmt>,
        /// Span.
        span: Span,
    },
    /// Block statement.
    Block(Vec<JsStmt>, Span, Trivia),
    /// Break statement.
    Break(Span, Trivia),
    /// Continue statement.
    Continue(Span, Trivia),
    /// Other statement.
    Other(String, Span, Trivia),
}

impl JsStmt {
    /// Returns the span of the statement.
    pub fn span(&self) -> Span {
        match self {
            JsStmt::Expr(_, span, _) => *span,
            JsStmt::VariableDecl { span, .. } => *span,
            JsStmt::FunctionDecl { span, .. } => *span,
            JsStmt::Import { span, .. } => *span,
            JsStmt::Export { span, .. } => *span,
            JsStmt::ExportDefault { span, .. } => *span,
            JsStmt::ExportAll { span, .. } => *span,
            JsStmt::ExportNamed { span, .. } => *span,
            JsStmt::Return(_, span, _) => *span,
            JsStmt::If { span, .. } => *span,
            JsStmt::While { span, .. } => *span,
            JsStmt::For { span, .. } => *span,
            JsStmt::Block(_, span, _) => *span,
            JsStmt::Break(span, _) => *span,
            JsStmt::Continue(span, _) => *span,
            JsStmt::Other(_, span, _) => *span,
        }
    }

    /// Returns a clone of the trivia of the statement.
    pub fn trivia(&self) -> Trivia {
        match self {
            JsStmt::Expr(_, _, trivia) => trivia.clone(),
            JsStmt::Return(_, _, trivia) => trivia.clone(),
            JsStmt::Block(_, _, trivia) => trivia.clone(),
            JsStmt::Break(_, trivia) => trivia.clone(),
            JsStmt::Continue(_, trivia) => trivia.clone(),
            JsStmt::Other(_, _, trivia) => trivia.clone(),
            _ => Trivia::default(),
        }
    }
}

/// Import specifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportSpecifier {
    /// Default import.
    Default {
        /// Name.
        name: String,
    },
    /// Named import.
    Named {
        /// Import name.
        name: String,
        /// Local alias.
        alias: Option<String>,
    },
    /// Namespace import.
    Namespace {
        /// Local alias.
        alias: Option<String>,
    },
}

/// JavaScript expression IR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JsExpr {
    /// Literal value.
    Literal(NargoValue, Span, Trivia),
    /// Identifier.
    Identifier(String, Span, Trivia),
    /// Unary expression.
    Unary {
        /// Operator.
        op: String,
        /// Argument.
        argument: Box<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Binary expression.
    Binary {
        /// Left operand.
        left: Box<JsExpr>,
        /// Operator.
        op: String,
        /// Right operand.
        right: Box<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Call expression.
    Call {
        /// Callee.
        callee: Box<JsExpr>,
        /// Arguments.
        args: Vec<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Member expression.
    Member {
        /// Object.
        object: Box<JsExpr>,
        /// Property name.
        property: String,
        /// Whether computed.
        computed: bool,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Object expression.
    Object(HashMap<String, JsExpr>, Span, Trivia),
    /// Array expression.
    Array(Vec<JsExpr>, Span, Trivia),
    /// Arrow function.
    ArrowFunction {
        /// Parameters.
        params: Vec<String>,
        /// Body.
        body: Box<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Conditional expression.
    Conditional {
        /// Condition.
        test: Box<JsExpr>,
        /// Consequent.
        consequent: Box<JsExpr>,
        /// Alternate.
        alternate: Box<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Template literal.
    TemplateLiteral {
        /// Quasi strings.
        quasis: Vec<String>,
        /// Expressions.
        expressions: Vec<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// TSE element.
    TseElement {
        /// Tag name.
        tag: String,
        /// Attributes.
        attributes: Vec<AttributeIR>,
        /// Children.
        children: Vec<JsExpr>,
        /// Span.
        span: Span,
        /// Trivia.
        trivia: Trivia,
    },
    /// Other expression.
    Other(String, Span, Trivia),
}

impl JsExpr {
    /// Returns the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            JsExpr::Literal(_, span, _) => *span,
            JsExpr::Identifier(_, span, _) => *span,
            JsExpr::Unary { span, .. } => *span,
            JsExpr::Binary { span, .. } => *span,
            JsExpr::Call { span, .. } => *span,
            JsExpr::Member { span, .. } => *span,
            JsExpr::Object(_, span, _) => *span,
            JsExpr::Array(_, span, _) => *span,
            JsExpr::ArrowFunction { span, .. } => *span,
            JsExpr::Conditional { span, .. } => *span,
            JsExpr::TemplateLiteral { span, .. } => *span,
            JsExpr::TseElement { span, .. } => *span,
            JsExpr::Other(_, span, _) => *span,
        }
    }

    /// Returns a clone of the trivia of the expression.
    pub fn trivia(&self) -> Trivia {
        match self {
            JsExpr::Literal(_, _, trivia) => trivia.clone(),
            JsExpr::Identifier(_, _, trivia) => trivia.clone(),
            JsExpr::Unary { trivia, .. } => trivia.clone(),
            JsExpr::Binary { trivia, .. } => trivia.clone(),
            JsExpr::Call { trivia, .. } => trivia.clone(),
            JsExpr::Member { trivia, .. } => trivia.clone(),
            JsExpr::Object(_, _, trivia) => trivia.clone(),
            JsExpr::Array(_, _, trivia) => trivia.clone(),
            JsExpr::ArrowFunction { trivia, .. } => trivia.clone(),
            JsExpr::Conditional { trivia, .. } => trivia.clone(),
            JsExpr::TemplateLiteral { trivia, .. } => trivia.clone(),
            JsExpr::TseElement { trivia, .. } => trivia.clone(),
            JsExpr::Other(_, _, trivia) => trivia.clone(),
        }
    }
}

/// Nargo value for literals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NargoValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Number value.
    Number(f64),
    /// String value.
    String(String),
    /// Array value.
    Array(Vec<NargoValue>),
    /// Object value.
    Object(HashMap<String, NargoValue>),
    /// Signal reference.
    Signal(String),
    /// Raw code.
    Raw(String),
    /// Other value.
    Other(String),
}

impl Default for NargoValue {
    fn default() -> Self {
        NargoValue::Null
    }
}
