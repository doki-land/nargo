#![warn(missing_docs)]

//! 语句模块
//!
//! 提供 JavaScript 语句的 IR 表示和相关功能。

use nargo_types::{Result, Span};
use serde::{Deserialize, Serialize};

use crate::{
    expr::JsExpr,
    types::{IRError, MAX_ARRAY_LENGTH, MAX_RECURSION_DEPTH, MAX_STRING_LENGTH, Trivia},
};

/// JavaScript 语句
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsStmt {
    /// 表达式语句
    Expr(JsExpr, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 变量声明
    VariableDecl {
        /// 声明类型（var, let, const）
        kind: String,
        /// 变量名
        id: String,
        /// 初始值
        init: Option<JsExpr>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 导入语句
    Import {
        /// 导入源
        source: String,
        /// 导入说明符
        specifiers: Vec<String>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 导出语句
    Export {
        /// 导出声明
        declaration: Box<JsStmt>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 导出所有
    ExportAll {
        /// 导出源
        source: String,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 导出命名
    ExportNamed {
        /// 导出源
        source: Option<String>,
        /// 导出说明符
        specifiers: Vec<String>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 函数声明
    FunctionDecl {
        /// 函数名
        id: String,
        /// 参数列表
        params: Vec<String>,
        /// 函数体
        body: Vec<JsStmt>,
        /// 是否为异步函数
        #[serde(default)]
        is_async: bool,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// 返回语句
    Return(Option<JsExpr>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// if 语句
    If {
        /// 条件
        test: JsExpr,
        /// 真分支
        consequent: Box<JsStmt>,
        /// 假分支
        alternate: Option<Box<JsStmt>>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// while 语句
    While {
        /// 条件
        test: JsExpr,
        /// 循环体
        body: Box<JsStmt>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// for 语句
    For {
        /// 初始化
        init: Option<Box<JsStmt>>,
        /// 条件
        test: Option<JsExpr>,
        /// 更新
        update: Option<JsExpr>,
        /// 循环体
        body: Box<JsStmt>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// for-in 语句
    ForIn {
        /// 变量
        left: Box<JsStmt>,
        /// 表达式
        right: JsExpr,
        /// 循环体
        body: Box<JsStmt>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// for-of 语句
    ForOf {
        /// 变量
        left: Box<JsStmt>,
        /// 表达式
        right: JsExpr,
        /// 循环体
        body: Box<JsStmt>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// try/catch 语句
    Try {
        /// try 块
        block: Box<JsStmt>,
        /// catch 块
        handler: Option<(String, Box<JsStmt>)>,
        /// finally 块
        finalizer: Option<Box<JsStmt>>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// switch 语句
    Switch {
        /// 表达式
        discriminant: JsExpr,
        /// 分支
        cases: Vec<(Option<JsExpr>, Vec<JsStmt>)>,
        /// 位置信息
        #[serde(default)]
        span: Span,
        /// Trivia 信息
        #[serde(default)]
        trivia: Trivia,
    },
    /// throw 语句
    Throw(JsExpr, #[serde(default)] Span, #[serde(default)] Trivia),
    /// 块语句
    Block(Vec<JsStmt>, #[serde(default)] Span, #[serde(default)] Trivia),
    /// break 语句
    Break(#[serde(default)] Span, #[serde(default)] Trivia),
    /// continue 语句
    Continue(#[serde(default)] Span, #[serde(default)] Trivia),
    /// 其他语句
    Other(String, #[serde(default)] Span, #[serde(default)] Trivia),
}

impl JsStmt {
    /// 获取语句的位置信息
    pub fn span(&self) -> Span {
        match self {
            JsStmt::Expr(_, span, _) => *span,
            JsStmt::VariableDecl { span, .. } => *span,
            JsStmt::Import { span, .. } => *span,
            JsStmt::Export { span, .. } => *span,
            JsStmt::ExportAll { span, .. } => *span,
            JsStmt::ExportNamed { span, .. } => *span,
            JsStmt::FunctionDecl { span, .. } => *span,
            JsStmt::Return(_, span, _) => *span,
            JsStmt::If { span, .. } => *span,
            JsStmt::While { span, .. } => *span,
            JsStmt::For { span, .. } => *span,
            JsStmt::ForIn { span, .. } => *span,
            JsStmt::ForOf { span, .. } => *span,
            JsStmt::Try { span, .. } => *span,
            JsStmt::Switch { span, .. } => *span,
            JsStmt::Throw(_, span, _) => *span,
            JsStmt::Block(_, span, _) => *span,
            JsStmt::Break(span, _) => *span,
            JsStmt::Continue(span, _) => *span,
            JsStmt::Other(_, span, _) => *span,
        }
    }

    /// 获取语句的 trivia 信息
    pub fn trivia(&self) -> &Trivia {
        match self {
            JsStmt::Expr(_, _, t) => t,
            JsStmt::VariableDecl { trivia, .. } => trivia,
            JsStmt::Import { trivia, .. } => trivia,
            JsStmt::Export { trivia, .. } => trivia,
            JsStmt::ExportAll { trivia, .. } => trivia,
            JsStmt::ExportNamed { trivia, .. } => trivia,
            JsStmt::FunctionDecl { trivia, .. } => trivia,
            JsStmt::Return(_, _, t) => t,
            JsStmt::If { trivia, .. } => trivia,
            JsStmt::While { trivia, .. } => trivia,
            JsStmt::For { trivia, .. } => trivia,
            JsStmt::ForIn { trivia, .. } => trivia,
            JsStmt::ForOf { trivia, .. } => trivia,
            JsStmt::Try { trivia, .. } => trivia,
            JsStmt::Switch { trivia, .. } => trivia,
            JsStmt::Throw(_, _, t) => t,
            JsStmt::Block(_, _, t) => t,
            JsStmt::Break(_, t) => t,
            JsStmt::Continue(_, t) => t,
            JsStmt::Other(_, _, t) => t,
        }
    }

    /// 验证语句的有效性
    pub fn validate(&self, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(IRError::CircularReference("Statement recursion depth exceeded".to_string()).into());
        }

        match self {
            JsStmt::Expr(expr, _, _) => expr.validate(depth + 1),
            JsStmt::VariableDecl { id, init, .. } => {
                if id.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Variable name length exceeded".to_string()).into());
                }
                if let Some(init_expr) = init {
                    init_expr.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::Import { source, specifiers, .. } => {
                if source.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Import source length exceeded".to_string()).into());
                }
                if specifiers.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Import specifiers length exceeded".to_string()).into());
                }
                for specifier in specifiers {
                    if specifier.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Import specifier length exceeded".to_string()).into());
                    }
                }
                Ok(())
            }
            JsStmt::Export { declaration, .. } => declaration.validate(depth + 1),
            JsStmt::ExportAll { source, .. } => {
                if source.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Export all source length exceeded".to_string()).into());
                }
                Ok(())
            }
            JsStmt::ExportNamed { source, specifiers, .. } => {
                if let Some(source_str) = source {
                    if source_str.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Export named source length exceeded".to_string()).into());
                    }
                }
                if specifiers.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Export named specifiers length exceeded".to_string()).into());
                }
                for specifier in specifiers {
                    if specifier.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Export specifier length exceeded".to_string()).into());
                    }
                }
                Ok(())
            }
            JsStmt::FunctionDecl { id, params, body, .. } => {
                if id.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Function name length exceeded".to_string()).into());
                }
                if params.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Function parameters length exceeded".to_string()).into());
                }
                for param in params {
                    if param.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Parameter name length exceeded".to_string()).into());
                    }
                }
                if body.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Function body length exceeded".to_string()).into());
                }
                for stmt in body {
                    stmt.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::Return(expr, _, _) => {
                if let Some(expr) = expr {
                    expr.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                test.validate(depth + 1)?;
                consequent.validate(depth + 1)?;
                if let Some(alt) = alternate {
                    alt.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::While { test, body, .. } => {
                test.validate(depth + 1)?;
                body.validate(depth + 1)
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    init_stmt.validate(depth + 1)?;
                }
                if let Some(test_expr) = test {
                    test_expr.validate(depth + 1)?;
                }
                if let Some(update_expr) = update {
                    update_expr.validate(depth + 1)?;
                }
                body.validate(depth + 1)
            }
            JsStmt::ForIn { left, right, body, .. } => {
                left.validate(depth + 1)?;
                right.validate(depth + 1)?;
                body.validate(depth + 1)
            }
            JsStmt::ForOf { left, right, body, .. } => {
                left.validate(depth + 1)?;
                right.validate(depth + 1)?;
                body.validate(depth + 1)
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                block.validate(depth + 1)?;
                if let Some((catch_id, catch_body)) = handler {
                    if catch_id.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Catch parameter name length exceeded".to_string()).into());
                    }
                    catch_body.validate(depth + 1)?;
                }
                if let Some(finally_body) = finalizer {
                    finally_body.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::Switch { discriminant, cases, .. } => {
                discriminant.validate(depth + 1)?;
                if cases.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Switch cases length exceeded".to_string()).into());
                }
                for (test, stmts) in cases {
                    if let Some(test_expr) = test {
                        test_expr.validate(depth + 1)?;
                    }
                    if stmts.len() > MAX_ARRAY_LENGTH {
                        return Err(IRError::SizeLimitExceeded("Switch case statements length exceeded".to_string()).into());
                    }
                    for stmt in stmts {
                        stmt.validate(depth + 1)?;
                    }
                }
                Ok(())
            }
            JsStmt::Throw(expr, _, _) => expr.validate(depth + 1),
            JsStmt::Block(stmts, _, _) => {
                if stmts.len() > MAX_ARRAY_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Block statements length exceeded".to_string()).into());
                }
                for stmt in stmts {
                    stmt.validate(depth + 1)?;
                }
                Ok(())
            }
            JsStmt::Break(_, _) => Ok(()),
            JsStmt::Continue(_, _) => Ok(()),
            JsStmt::Other(code, _, _) => {
                if code.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("Other statement code length exceeded".to_string()).into());
                }
                Ok(())
            }
        }
    }

    /// 优化语句
    pub fn optimize(&mut self) {
        crate::optimizer::StmtOptimizer::optimize(self);
    }

    /// 检查语句是否为空
    pub fn is_empty(&self) -> bool {
        match self {
            JsStmt::Block(stmts, _, _) => stmts.is_empty(),
            _ => false,
        }
    }
}

/// 语句访问者 trait
pub trait JsStmtVisitor<R> {
    /// 访问表达式语句
    fn visit_expr(&mut self, expr: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问变量声明语句
    fn visit_variable_decl(&mut self, kind: &String, id: &String, init: &Option<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问导入语句
    fn visit_import(&mut self, source: &String, specifiers: &Vec<String>, span: &Span, trivia: &Trivia) -> R;
    /// 访问导出语句
    fn visit_export(&mut self, declaration: &JsStmt, span: &Span, trivia: &Trivia) -> R;
    /// 访问导出所有语句
    fn visit_export_all(&mut self, source: &String, span: &Span, trivia: &Trivia) -> R;
    /// 访问导出命名语句
    fn visit_export_named(&mut self, source: &Option<String>, specifiers: &Vec<String>, span: &Span, trivia: &Trivia) -> R;
    /// 访问函数声明语句
    fn visit_function_decl(&mut self, id: &String, params: &Vec<String>, body: &Vec<JsStmt>, is_async: bool, span: &Span, trivia: &Trivia) -> R;
    /// 访问返回语句
    fn visit_return(&mut self, expr: &Option<JsExpr>, span: &Span, trivia: &Trivia) -> R;
    /// 访问 if 语句
    fn visit_if(&mut self, test: &JsExpr, consequent: &JsStmt, alternate: &Option<Box<JsStmt>>, span: &Span, trivia: &Trivia) -> R;
    /// 访问 while 语句
    fn visit_while(&mut self, test: &JsExpr, body: &JsStmt, span: &Span, trivia: &Trivia) -> R;
    /// 访问 for 语句
    fn visit_for(&mut self, init: &Option<Box<JsStmt>>, test: &Option<JsExpr>, update: &Option<JsExpr>, body: &JsStmt, span: &Span, trivia: &Trivia) -> R;
    /// 访问 for-in 语句
    fn visit_for_in(&mut self, left: &JsStmt, right: &JsExpr, body: &JsStmt, span: &Span, trivia: &Trivia) -> R;
    /// 访问 for-of 语句
    fn visit_for_of(&mut self, left: &JsStmt, right: &JsExpr, body: &JsStmt, span: &Span, trivia: &Trivia) -> R;
    /// 访问 try/catch 语句
    fn visit_try(&mut self, block: &JsStmt, handler: &Option<(String, Box<JsStmt>)>, finalizer: &Option<Box<JsStmt>>, span: &Span, trivia: &Trivia) -> R;
    /// 访问 switch 语句
    fn visit_switch(&mut self, discriminant: &JsExpr, cases: &Vec<(Option<JsExpr>, Vec<JsStmt>)>, span: &Span, trivia: &Trivia) -> R;
    /// 访问 throw 语句
    fn visit_throw(&mut self, expr: &JsExpr, span: &Span, trivia: &Trivia) -> R;
    /// 访问块语句
    fn visit_block(&mut self, stmts: &Vec<JsStmt>, span: &Span, trivia: &Trivia) -> R;
    /// 访问 break 语句
    fn visit_break(&mut self, span: &Span, trivia: &Trivia) -> R;
    /// 访问 continue 语句
    fn visit_continue(&mut self, span: &Span, trivia: &Trivia) -> R;
    /// 访问其他语句
    fn visit_other(&mut self, code: &String, span: &Span, trivia: &Trivia) -> R;
}

impl JsStmt {
    /// 接受访问者
    pub fn accept<R>(&self, visitor: &mut dyn JsStmtVisitor<R>) -> R {
        match self {
            JsStmt::Expr(expr, span, trivia) => visitor.visit_expr(expr, span, trivia),
            JsStmt::VariableDecl { kind, id, init, span, trivia } => visitor.visit_variable_decl(kind, id, init, span, trivia),
            JsStmt::Import { source, specifiers, span, trivia } => visitor.visit_import(source, specifiers, span, trivia),
            JsStmt::Export { declaration, span, trivia } => visitor.visit_export(declaration, span, trivia),
            JsStmt::ExportAll { source, span, trivia } => visitor.visit_export_all(source, span, trivia),
            JsStmt::ExportNamed { source, specifiers, span, trivia } => visitor.visit_export_named(source, specifiers, span, trivia),
            JsStmt::FunctionDecl { id, params, body, is_async, span, trivia } => visitor.visit_function_decl(id, params, body, *is_async, span, trivia),
            JsStmt::Return(expr, span, trivia) => visitor.visit_return(expr, span, trivia),
            JsStmt::If { test, consequent, alternate, span, trivia } => visitor.visit_if(test, consequent, alternate, span, trivia),
            JsStmt::While { test, body, span, trivia } => visitor.visit_while(test, body, span, trivia),
            JsStmt::For { init, test, update, body, span, trivia } => visitor.visit_for(init, test, update, body, span, trivia),
            JsStmt::ForIn { left, right, body, span, trivia } => visitor.visit_for_in(left, right, body, span, trivia),
            JsStmt::ForOf { left, right, body, span, trivia } => visitor.visit_for_of(left, right, body, span, trivia),
            JsStmt::Try { block, handler, finalizer, span, trivia } => visitor.visit_try(block, handler, finalizer, span, trivia),
            JsStmt::Switch { discriminant, cases, span, trivia } => visitor.visit_switch(discriminant, cases, span, trivia),
            JsStmt::Throw(expr, span, trivia) => visitor.visit_throw(expr, span, trivia),
            JsStmt::Block(stmts, span, trivia) => visitor.visit_block(stmts, span, trivia),
            JsStmt::Break(span, trivia) => visitor.visit_break(span, trivia),
            JsStmt::Continue(span, trivia) => visitor.visit_continue(span, trivia),
            JsStmt::Other(code, span, trivia) => visitor.visit_other(code, span, trivia),
        }
    }
}
