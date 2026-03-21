#![warn(missing_docs)]

//! DejaVu 模板前端模块

use crate::frontend::Frontend;
use crate::ir::{BinaryOp, ForPattern, IRBuilder, TemplateIR, UnaryOp};
use crate::error::{TemplateError, TemplateResult};
use oak_dejavu::ast::{
    DejavuRoot, ElseBranchNode, ExpressionNode, ItemNode, LiteralExpressionNode, PatternNode,
    StatementNode,
};
use oak_dejavu::lexer::token_type::DejavuTokenType;

/// DejaVu 模板前端
///
/// 将 DejaVu 模板源码解析为 AST，然后遍历 AST 生成扁平 IR 指令序列。
pub struct DejaVuFrontend {}

impl DejaVuFrontend {
    /// 创建新的 DejaVu 前端实例
    pub fn new() -> Self {
        Self {}
    }

    /// 将 DejaVu 模板源码解析为 AST
    fn parse(&self, source: &str) -> TemplateResult<DejavuRoot> {
        oak_dejavu::parse(source).map_err(TemplateError::Syntax)
    }

    /// 编译 ItemNode 列表为 IR 指令
    fn compile_items(builder: &mut IRBuilder, items: &[ItemNode]) -> TemplateResult<()> {
        for item in items {
            Self::compile_item(builder, item)?;
        }
        Ok(())
    }

    /// 编译单个 ItemNode 为 IR 指令
    fn compile_item(builder: &mut IRBuilder, item: &ItemNode) -> TemplateResult<()> {
        match item {
            ItemNode::TemplateText(text) => {
                builder.output_text(text.content.clone());
            }
            ItemNode::TemplateInterpolation(interp) => {
                Self::compile_expr(builder, &interp.expr)?;
                builder.output();
            }
            ItemNode::IfControl(if_ctrl) => {
                Self::compile_if(builder, &if_ctrl.condition, &if_ctrl.then_body, &if_ctrl.else_branch)?;
            }
            ItemNode::ForControl(for_ctrl) => {
                Self::compile_for(
                    builder,
                    &for_ctrl.pattern,
                    &for_ctrl.iterable,
                    &for_ctrl.body,
                    for_ctrl.else_body.as_deref(),
                )?;
            }
            ItemNode::WhileControl(while_ctrl) => {
                Self::compile_while(builder, &while_ctrl.condition, &while_ctrl.body)?;
            }
            ItemNode::LoopControl(loop_ctrl) => {
                Self::compile_for(
                    builder,
                    &loop_ctrl.pattern,
                    &loop_ctrl.iterable,
                    &loop_ctrl.body,
                    None,
                )?;
            }
            ItemNode::RawBlock(raw) => {
                builder.output_text(raw.content.clone());
            }
            ItemNode::IncludeDirective(include) => {
                Self::compile_expr(builder, &include.path)?;
                builder.include(true);
            }
            ItemNode::Block(block) => {
                builder.scope_begin();
                Self::compile_items(builder, &block.items)?;
                builder.scope_end();
            }
            ItemNode::Statement(stmt) => {
                Self::compile_statement(builder, stmt)?;
            }
            ItemNode::TemplateControl(ctrl) => {
                builder.scope_begin();
                Self::compile_items(builder, &ctrl.items)?;
                builder.scope_end();
            }
            ItemNode::Namespace(_)
            | ItemNode::Class(_)
            | ItemNode::Flags(_)
            | ItemNode::Enum(_)
            | ItemNode::Trait(_)
            | ItemNode::Widget(_)
            | ItemNode::Using(_)
            | ItemNode::Micro(_)
            | ItemNode::TypeFunction(_)
            | ItemNode::Variant(_) => {}
        }
        Ok(())
    }

    /// 编译 if 控制结构为 IR 指令
    fn compile_if(
        builder: &mut IRBuilder,
        condition: &ExpressionNode,
        then_body: &[ItemNode],
        else_branch: &Option<ElseBranchNode>,
    ) -> TemplateResult<()> {
        Self::compile_expr(builder, condition)?;
        let jump_if_false = builder.emit_jump_if_false();
        Self::compile_items(builder, then_body)?;

        if let Some(else_br) = else_branch {
            let jump_end = builder.emit_jump();
            builder.patch_jump(jump_if_false);
            Self::compile_else_branch(builder, else_br)?;
            builder.patch_jump(jump_end);
        } else {
            builder.patch_jump(jump_if_false);
        }
        Ok(())
    }

    /// 编译 else 分支为 IR 指令
    fn compile_else_branch(builder: &mut IRBuilder, else_branch: &ElseBranchNode) -> TemplateResult<()> {
        match else_branch {
            ElseBranchNode::Elif {
                condition,
                body,
                else_branch,
            } => {
                Self::compile_expr(builder, condition)?;
                let jump_if_false = builder.emit_jump_if_false();
                Self::compile_items(builder, body)?;

                if let Some(inner_else) = else_branch {
                    let jump_end = builder.emit_jump();
                    builder.patch_jump(jump_if_false);
                    Self::compile_else_branch(builder, inner_else)?;
                    builder.patch_jump(jump_end);
                } else {
                    builder.patch_jump(jump_if_false);
                }
            }
            ElseBranchNode::Else { body } => {
                Self::compile_items(builder, body)?;
            }
        }
        Ok(())
    }

    /// 编译 for 循环为 IR 指令
    fn compile_for(
        builder: &mut IRBuilder,
        pattern: &PatternNode,
        iterable: &ExpressionNode,
        body: &[ItemNode],
        else_body: Option<&[ItemNode]>,
    ) -> TemplateResult<()> {
        Self::compile_expr(builder, iterable)?;
        let ir_pattern = Self::compile_pattern(pattern);
        let for_init_ip = builder.emit_for_init(ir_pattern);
        let loop_start = builder.current_ip();
        Self::compile_items(builder, body)?;
        builder.emit_for_next(loop_start);

        if let Some(else_items) = else_body {
            let jump_over_else = builder.emit_jump();
            builder.patch_jump(for_init_ip);
            Self::compile_items(builder, else_items)?;
            builder.patch_jump(jump_over_else);
        } else {
            builder.patch_jump(for_init_ip);
        }
        Ok(())
    }

    /// 编译 while 循环为 IR 指令
    fn compile_while(
        builder: &mut IRBuilder,
        condition: &ExpressionNode,
        body: &[ItemNode],
    ) -> TemplateResult<()> {
        let loop_start = builder.current_ip();
        Self::compile_expr(builder, condition)?;
        let jump_if_false = builder.emit_jump_if_false();
        Self::compile_items(builder, body)?;
        let jump_back = builder.emit_jump();
        builder.patch_jump_to(jump_back, loop_start);
        builder.patch_jump(jump_if_false);
        Ok(())
    }

    /// 编译语句节点为 IR 指令
    fn compile_statement(builder: &mut IRBuilder, stmt: &StatementNode) -> TemplateResult<()> {
        match stmt {
            StatementNode::Let(let_stmt) => {
                let name = Self::extract_pattern_name(&let_stmt.pattern);
                Self::compile_expr(builder, &let_stmt.expr)?;
                builder.store_var(name);
            }
            StatementNode::Expr(expr_stmt) => {
                if !expr_stmt.semi {
                    Self::compile_expr(builder, &expr_stmt.expr)?;
                    builder.output();
                }
            }
        }
        Ok(())
    }

    /// 编译表达式节点为 IR 指令
    fn compile_expr(builder: &mut IRBuilder, expr: &ExpressionNode) -> TemplateResult<()> {
        match expr {
            ExpressionNode::Ident(ident) => {
                builder.load_var(ident.name.clone());
            }
            ExpressionNode::Path(path) => {
                let mut parts = path.parts.iter();
                if let Some(first) = parts.next() {
                    builder.load_var(first.name.clone());
                    for part in parts {
                        builder.field_access(part.name.clone());
                    }
                } else {
                    builder.push_null();
                }
            }
            ExpressionNode::Literal(lit) => {
                Self::compile_literal(builder, lit);
            }
            ExpressionNode::Bool(b) => {
                builder.push_bool(b.value);
            }
            ExpressionNode::Paren(paren) => {
                Self::compile_expr(builder, &paren.expr)?;
            }
            ExpressionNode::Unary(unary) => {
                let op = Self::map_unary_op(&unary.op);
                match op {
                    Some(ir_op) => {
                        Self::compile_expr(builder, &unary.expr)?;
                        builder.unary_op(ir_op);
                    }
                    None => {
                        builder.push_null();
                    }
                }
            }
            ExpressionNode::Binary(binary) => {
                let op = Self::map_binary_op(&binary.op);
                match op {
                    Some(ir_op) => {
                        Self::compile_expr(builder, &binary.left)?;
                        Self::compile_expr(builder, &binary.right)?;
                        builder.binary_op(ir_op);
                    }
                    None => {
                        builder.push_null();
                    }
                }
            }
            ExpressionNode::Call(call) => {
                let callee_name = Self::extract_callee_name(&call.callee);
                for arg in &call.args {
                    Self::compile_expr(builder, arg)?;
                }
                builder.call_filter(callee_name, call.args.len() as u8);
            }
            ExpressionNode::Field(field) => {
                Self::compile_expr(builder, &field.receiver)?;
                builder.field_access(field.field.name.clone());
            }
            ExpressionNode::Index(index) => {
                Self::compile_expr(builder, &index.receiver)?;
                Self::compile_expr(builder, &index.index)?;
                builder.index_access();
            }
            ExpressionNode::Filter(filter) => {
                Self::compile_expr(builder, &filter.expr)?;
                for arg in &filter.args {
                    Self::compile_expr(builder, arg)?;
                }
                builder.call_filter(filter.name.name.clone(), filter.args.len() as u8);
            }
            ExpressionNode::If(if_expr) => {
                Self::compile_expr(builder, &if_expr.condition)?;
                let jump_if_false = builder.emit_jump_if_false();
                if let Some(then_stmt) = if_expr.then_branch.statements.first() {
                    if let StatementNode::Expr(expr_stmt) = then_stmt {
                        Self::compile_expr(builder, &expr_stmt.expr)?;
                    } else {
                        builder.push_null();
                    }
                } else {
                    builder.push_null();
                }
                let jump_end = builder.emit_jump();
                builder.patch_jump(jump_if_false);
                if let Some(else_block) = &if_expr.else_branch {
                    if let Some(else_stmt) = else_block.statements.first() {
                        if let StatementNode::Expr(expr_stmt) = else_stmt {
                            Self::compile_expr(builder, &expr_stmt.expr)?;
                        } else {
                            builder.push_null();
                        }
                    } else {
                        builder.push_null();
                    }
                } else {
                    builder.push_null();
                }
                builder.patch_jump(jump_end);
            }
            ExpressionNode::Match(_)
            | ExpressionNode::Lambda(_)
            | ExpressionNode::Object(_)
            | ExpressionNode::Block(_)
            | ExpressionNode::Loop(_)
            | ExpressionNode::Return(_)
            | ExpressionNode::Break(_)
            | ExpressionNode::Continue(_)
            | ExpressionNode::Yield(_)
            | ExpressionNode::Raise(_)
            | ExpressionNode::Catch(_)
            | ExpressionNode::Resume(_)
            | ExpressionNode::Translate(_) => {
                builder.push_null();
            }
        }
        Ok(())
    }

    /// 编译字面量表达式为 IR 指令
    fn compile_literal(builder: &mut IRBuilder, lit: &LiteralExpressionNode) {
        if let Ok(n) = lit.value.parse::<f64>() {
            builder.push_number(n);
        } else {
            builder.push_string(lit.value.clone());
        }
    }

    /// 将 DejavuTokenType 二元运算符映射为 IR BinaryOp
    fn map_binary_op(op: &DejavuTokenType) -> Option<BinaryOp> {
        match op {
            DejavuTokenType::Plus => Some(BinaryOp::Add),
            DejavuTokenType::Minus => Some(BinaryOp::Sub),
            DejavuTokenType::Star => Some(BinaryOp::Mul),
            DejavuTokenType::Slash => Some(BinaryOp::Div),
            DejavuTokenType::Percent => Some(BinaryOp::Mod),
            DejavuTokenType::EqEq => Some(BinaryOp::Eq),
            DejavuTokenType::NotEq => Some(BinaryOp::Ne),
            DejavuTokenType::LessThan => Some(BinaryOp::Lt),
            DejavuTokenType::LessEq => Some(BinaryOp::Le),
            DejavuTokenType::GreaterThan => Some(BinaryOp::Gt),
            DejavuTokenType::GreaterEq => Some(BinaryOp::Ge),
            DejavuTokenType::AndAnd => Some(BinaryOp::And),
            DejavuTokenType::OrOr => Some(BinaryOp::Or),
            _ => None,
        }
    }

    /// 将 DejavuTokenType 一元运算符映射为 IR UnaryOp
    fn map_unary_op(op: &DejavuTokenType) -> Option<UnaryOp> {
        match op {
            DejavuTokenType::Bang => Some(UnaryOp::Not),
            DejavuTokenType::Minus => Some(UnaryOp::Neg),
            _ => None,
        }
    }

    /// 从调用表达式中提取被调用者名称
    fn extract_callee_name(expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Ident(ident) => ident.name.clone(),
            ExpressionNode::Path(path) => path
                .parts
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join("::"),
            _ => String::new(),
        }
    }

    /// 将 PatternNode 编译为 ForPattern
    fn compile_pattern(pattern: &PatternNode) -> ForPattern {
        match pattern {
            PatternNode::Variable(var) => ForPattern::Identifier(var.name.name.clone()),
            PatternNode::Wildcard(_) => ForPattern::Identifier("_".to_string()),
            PatternNode::Tuple(tuple) => {
                let names: Vec<String> = tuple.items.iter().map(Self::extract_pattern_name).collect();
                ForPattern::Tuple(names)
            }
            PatternNode::Array(array_pat) => {
                let names: Vec<String> = array_pat.items.iter().map(Self::extract_pattern_name).collect();
                ForPattern::Tuple(names)
            }
            PatternNode::Object(obj_pat) => {
                let names: Vec<String> = obj_pat.props.iter().map(|(key, _)| key.name.clone()).collect();
                ForPattern::Tuple(names)
            }
            _ => ForPattern::Identifier("_".to_string()),
        }
    }

    /// 从模式节点中提取变量名
    fn extract_pattern_name(pattern: &PatternNode) -> String {
        match pattern {
            PatternNode::Variable(var) => var.name.name.clone(),
            PatternNode::Wildcard(_) => "_".to_string(),
            _ => "_".to_string(),
        }
    }
}

impl Default for DejaVuFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for DejaVuFrontend {
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR> {
        let ast = self.parse(source)?;
        let mut builder = IRBuilder::new();
        Self::compile_items(&mut builder, &ast.items)?;
        Ok(builder.build())
    }

    fn name(&self) -> &str {
        "dejavu"
    }

    fn default_extension(&self) -> &str {
        "djv"
    }
}
