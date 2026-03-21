#![warn(missing_docs)]

use crate::frontend::Frontend;
use crate::ir::{BinaryOp, ForPattern, IRBuilder, TemplateIR, UnaryOp};
use crate::error::{TemplateError, TemplateResult};
use oak_core::{
    SourceText,
    parser::{ParseSession, Parser},
    tree::RedTree,
};
use oak_jinja::{JinjaElementType, JinjaLanguage, JinjaParser};

/// Jinja2 模板语言前端
///
/// 使用 `oak_jinja` 解析器将 Jinja2 模板源码解析为 RedTree，
/// 然后遍历 RedTree 编译为扁平 IR 指令序列，供 VM 执行。
pub struct Jinja2Frontend {
    /// Jinja2 语言定义
    language: JinjaLanguage,
}

impl Jinja2Frontend {
    /// 创建新的 Jinja2 前端实例
    pub fn new() -> Self {
        Self {
            language: JinjaLanguage::default(),
        }
    }
}

impl Default for Jinja2Frontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for Jinja2Frontend {
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR> {
        let source_text = SourceText::new(source);
        let parser = JinjaParser::new(&self.language);
        let mut session = ParseSession::<JinjaLanguage>::default();
        let parse_output = parser.parse(&source_text, &[], &mut session);
        let green_node = parse_output
            .result
            .map_err(|e| TemplateError::Syntax(format!("Jinja2 parse error: {}", e)))?;
        let red_tree = RedTree::new(green_node);
        let mut builder = IRBuilder::new();
        compile_children(&mut builder, &red_tree, &source_text)?;
        Ok(builder.build())
    }

    fn name(&self) -> &str {
        "jinja2"
    }

    fn default_extension(&self) -> &str {
        "j2"
    }
}

/// 遍历 RedTree 子节点列表，逐个编译为 IR 指令
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: RedTree 节点引用
/// - `source`: 模板源文本
fn compile_children(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    for child in tree.children() {
        compile_node(builder, &child, source)?;
    }
    Ok(())
}

/// 根据节点类型分派编译逻辑
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: RedTree 节点引用
/// - `source`: 模板源文本
fn compile_node(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    let kind: JinjaElementType = tree.kind();
    match kind {
        JinjaElementType::Text => {
            let text = tree.text(source);
            if !text.is_empty() {
                builder.output_text(text.into_owned());
            }
            Ok(())
        }
        JinjaElementType::Variable => {
            compile_expr(builder, tree, source);
            builder.output();
            Ok(())
        }
        JinjaElementType::IfStatement => compile_if(builder, tree, source),
        JinjaElementType::ForStatement => compile_for(builder, tree, source),
        JinjaElementType::Block => compile_block(builder, tree, source),
        JinjaElementType::Comment => Ok(()),
        JinjaElementType::MacroDefinition => Ok(()),
        JinjaElementType::Tag => Ok(()),
        JinjaElementType::Error => {
            let text = tree.text(source);
            Err(TemplateError::Syntax(format!(
                "Jinja2 parse error at: {}",
                text
            )))
        }
        _ => Ok(()),
    }
}

/// 编译 Variable 节点的表达式，将结果压入操作数栈
///
/// Variable 节点可能包含子节点（结构化表达式），
/// 也可能没有子节点（需要从文本回退解析）。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: Variable 类型的 RedTree 节点引用
/// - `source`: 模板源文本
fn compile_expr(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) {
    let children: Vec<_> = tree.children().collect();
    if children.is_empty() {
        let text = tree.text(source);
        compile_expr_text(builder, text.trim());
        return;
    }
    compile_expr_children(builder, &children, source);
}

/// 从子节点列表编译表达式，将结果压入操作数栈
///
/// 遍历子节点，识别过滤器、二元运算符、一元运算符等结构，
/// 按优先级递归编译并发射对应的栈操作指令。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `children`: 子节点列表
/// - `source`: 模板源文本
fn compile_expr_children(
    builder: &mut IRBuilder,
    children: &[RedTree<'_, JinjaLanguage>],
    source: &SourceText,
) {
    if children.is_empty() {
        builder.push_null();
        return;
    }
    if children.len() == 1 {
        compile_single_expr_child(builder, &children[0], source);
        return;
    }

    let mut pipe_positions: Vec<usize> = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let kind: JinjaElementType = child.kind();
        if kind == JinjaElementType::Filter {
            pipe_positions.push(i);
        }
    }

    if !pipe_positions.is_empty() {
        let first_pipe = pipe_positions[0];
        let left_children = &children[..first_pipe];
        let right_children = &children[first_pipe + 1..];

        if left_children.len() == 1 {
            compile_single_expr_child(builder, &left_children[0], source);
        } else {
            compile_expr_children(builder, left_children, source);
        }

        let filter_name = right_children
            .first()
            .map(|c| c.text(source).trim().to_string())
            .unwrap_or_default();

        let arg_count = if right_children.len() > 1 {
            for arg in &right_children[1..] {
                compile_single_expr_child(builder, arg, source);
            }
            (right_children.len() - 1) as u8
        } else {
            0
        };

        builder.call_filter(filter_name, arg_count);
        return;
    }

    let mut binary_ops: Vec<(usize, BinaryOp)> = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let text = child.text(source);
        let op = match text.trim() {
            "+" => Some(BinaryOp::Add),
            "-" => Some(BinaryOp::Sub),
            "*" => Some(BinaryOp::Mul),
            "/" => Some(BinaryOp::Div),
            "%" => Some(BinaryOp::Mod),
            "==" => Some(BinaryOp::Eq),
            "!=" => Some(BinaryOp::Ne),
            "<" => Some(BinaryOp::Lt),
            "<=" => Some(BinaryOp::Le),
            ">" => Some(BinaryOp::Gt),
            ">=" => Some(BinaryOp::Ge),
            "and" => Some(BinaryOp::And),
            "or" => Some(BinaryOp::Or),
            _ => None,
        };
        if let Some(op) = op {
            binary_ops.push((i, op));
        }
    }

    if let Some(&(idx, op)) = binary_ops.last() {
        let left_children = &children[..idx];
        let right_children = &children[idx + 1..];
        if !left_children.is_empty() && !right_children.is_empty() {
            if left_children.len() == 1 {
                compile_single_expr_child(builder, &left_children[0], source);
            } else {
                compile_expr_children(builder, left_children, source);
            }
            if right_children.len() == 1 {
                compile_single_expr_child(builder, &right_children[0], source);
            } else {
                compile_expr_children(builder, right_children, source);
            }
            builder.binary_op(op);
            return;
        }
    }

    for (i, child) in children.iter().enumerate() {
        let text = child.text(source);
        if text.trim() == "not" {
            let remaining = &children[i + 1..];
            if !remaining.is_empty() {
                if remaining.len() == 1 {
                    compile_single_expr_child(builder, &remaining[0], source);
                } else {
                    compile_expr_children(builder, remaining, source);
                }
                builder.unary_op(UnaryOp::Not);
                return;
            }
        }
    }

    compile_single_expr_child(builder, &children[0], source);
}

/// 编译单个表达式子节点，将结果压入操作数栈
///
/// 根据子节点的 JinjaElementType 分派到对应的栈操作编译逻辑：
/// - `Identifier` → `load_var` + 可能的 `field_access`
/// - `Literal` → `push_bool` / `push_number` / `push_string` / `push_null`
/// - `Filter` → 递归编译过滤器
/// - `Expression` → 递归编译子节点
/// - `Function` → `push_null` + 编译参数 + `call_filter`
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `child`: 单个子节点引用
/// - `source`: 模板源文本
fn compile_single_expr_child(
    builder: &mut IRBuilder,
    child: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) {
    let kind: JinjaElementType = child.kind();
    match kind {
        JinjaElementType::Identifier => {
            let text = child.text(source);
            let text = text.trim();
            if text == "not" {
                let sub_children: Vec<_> = child.children().collect();
                if !sub_children.is_empty() {
                    compile_expr_children(builder, &sub_children, source);
                    builder.unary_op(UnaryOp::Not);
                    return;
                }
            }
            compile_identifier_path(builder, text);
        }
        JinjaElementType::Literal => {
            let text = child.text(source);
            compile_literal(builder, text.trim());
        }
        JinjaElementType::Filter => {
            let sub_children: Vec<_> = child.children().collect();
            if !sub_children.is_empty() {
                compile_expr_children(builder, &sub_children, source);
            } else {
                builder.push_null();
            }
        }
        JinjaElementType::Expression => {
            let sub_children: Vec<_> = child.children().collect();
            if sub_children.len() == 1 {
                compile_single_expr_child(builder, &sub_children[0], source);
            } else if !sub_children.is_empty() {
                compile_expr_children(builder, &sub_children, source);
            } else {
                let text = child.text(source);
                compile_expr_text(builder, text.trim());
            }
        }
        JinjaElementType::Function => {
            let sub_children: Vec<_> = child.children().collect();
            let mut callee = String::new();
            let mut args_children: Vec<RedTree<'_, JinjaLanguage>> = Vec::new();
            let mut past_first = false;
            for sub in &sub_children {
                let sub_kind: JinjaElementType = sub.kind();
                if sub_kind == JinjaElementType::Identifier && callee.is_empty() && !past_first {
                    callee = sub.text(source).trim().to_string();
                    past_first = true;
                } else {
                    args_children.push(sub.clone());
                }
            }
            if callee.is_empty() {
                let text = child.text(source);
                callee = text.trim().to_string();
            }
            builder.push_null();
            for arg in &args_children {
                compile_single_expr_child(builder, arg, source);
            }
            builder.call_filter(callee, args_children.len() as u8);
        }
        _ => {
            let sub_children: Vec<_> = child.children().collect();
            if !sub_children.is_empty() {
                compile_expr_children(builder, &sub_children, source);
            } else {
                let text = child.text(source);
                compile_expr_text(builder, text.trim());
            }
        }
    }
}

/// 编译 IfStatement 节点为跳转指令序列
///
/// 生成条件跳转指令实现 if/elif/else 分支：
/// - 编译条件表达式，发射 `JumpIfFalse` 跳到下一分支
/// - 编译 then 体
/// - 发射 `Jump` 跳到所有分支结束点
/// - 递归处理 elif 链和 else 体
/// - 最后修补所有跳转目标
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: IfStatement 类型的 RedTree 节点引用
/// - `source`: 模板源文本
fn compile_if(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    let mut end_jumps: Vec<usize> = Vec::new();
    compile_if_chain(builder, tree, source, &mut end_jumps)?;
    for jump_ip in end_jumps {
        builder.patch_jump(jump_ip);
    }
    Ok(())
}

/// 递归编译 if/elif 链
///
/// 处理单个 if 或 elif 节点的条件判断和体编译，
/// 遇到嵌套的 elif 或 else 时递归处理。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: IfStatement 类型的 RedTree 节点引用
/// - `source`: 模板源文本
/// - `end_jumps`: 收集所有分支结束跳转的指令位置
fn compile_if_chain(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
    end_jumps: &mut Vec<usize>,
) -> TemplateResult<()> {
    let children: Vec<_> = tree.children().collect();
    let mut state = IfParseState::Condition;
    let mut jump_if_false_ip: Option<usize> = None;

    for child in &children {
        let kind: JinjaElementType = child.kind();
        let text = child.text(source);
        let trimmed = text.trim();

        match state {
            IfParseState::Condition => {
                match kind {
                    JinjaElementType::Expression
                    | JinjaElementType::Identifier
                    | JinjaElementType::Literal
                    | JinjaElementType::Filter
                    | JinjaElementType::Function => {
                        compile_single_expr_child(builder, child, source);
                        jump_if_false_ip = Some(builder.emit_jump_if_false());
                        state = IfParseState::Body;
                    }
                    JinjaElementType::Text => {
                        if !trimmed.is_empty() {
                            compile_expr_text(builder, trimmed);
                            jump_if_false_ip = Some(builder.emit_jump_if_false());
                            state = IfParseState::Body;
                        }
                    }
                    _ => {
                        state = IfParseState::Body;
                        compile_node(builder, child, source)?;
                    }
                }
            }
            IfParseState::Body => {
                if kind == JinjaElementType::IfStatement {
                    if is_elif_node(child, source) {
                        let end_jump = builder.emit_jump();
                        end_jumps.push(end_jump);
                        if let Some(ip) = jump_if_false_ip.take() {
                            builder.patch_jump(ip);
                        }
                        compile_if_chain(builder, child, source, end_jumps)?;
                        return Ok(());
                    } else if is_else_node(child, source) {
                        let end_jump = builder.emit_jump();
                        end_jumps.push(end_jump);
                        if let Some(ip) = jump_if_false_ip.take() {
                            builder.patch_jump(ip);
                        }
                        compile_else_children(builder, child, source)?;
                        return Ok(());
                    } else {
                        compile_node(builder, child, source)?;
                    }
                } else {
                    compile_node(builder, child, source)?;
                }
            }
        }
    }

    if let Some(ip) = jump_if_false_ip {
        builder.patch_jump(ip);
    }

    Ok(())
}

/// 编译 else 分支的子节点
///
/// 跳过 else 关键字节点，编译剩余的体节点。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: else IfStatement 类型的 RedTree 节点引用
/// - `source`: 模板源文本
fn compile_else_children(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    let children: Vec<_> = tree.children().collect();
    let mut past_header = false;
    for child in &children {
        if !past_header {
            let kind: JinjaElementType = child.kind();
            let text = child.text(source);
            let trimmed = text.trim();
            match kind {
                JinjaElementType::Identifier => {
                    if trimmed == "else" {
                        past_header = true;
                    }
                }
                JinjaElementType::Text => {
                    if trimmed.starts_with("else") {
                        past_header = true;
                    } else if !trimmed.is_empty() {
                        past_header = true;
                        compile_node(builder, child, source)?;
                    }
                }
                _ => {
                    past_header = true;
                    compile_node(builder, child, source)?;
                }
            }
        } else {
            compile_node(builder, child, source)?;
        }
    }
    Ok(())
}

/// If 语句解析状态
enum IfParseState {
    /// 正在解析条件表达式
    Condition,
    /// 正在解析体内容
    Body,
}

/// 判断 IfStatement 子节点是否为 elif 分支
///
/// # 参数
/// - `tree`: RedTree 节点引用
/// - `source`: 模板源文本
fn is_elif_node(tree: &RedTree<'_, JinjaLanguage>, source: &SourceText) -> bool {
    for child in tree.children() {
        let kind: JinjaElementType = child.kind();
        if kind == JinjaElementType::Identifier {
            let text = child.text(source);
            if text.trim() == "elif" {
                return true;
            }
        }
        if kind == JinjaElementType::Text {
            let text = child.text(source);
            if text.trim().starts_with("elif") {
                return true;
            }
        }
        break;
    }
    false
}

/// 判断节点是否为 else 分支
///
/// # 参数
/// - `tree`: RedTree 节点引用
/// - `source`: 模板源文本
fn is_else_node(tree: &RedTree<'_, JinjaLanguage>, source: &SourceText) -> bool {
    let text = tree.text(source);
    text.trim().starts_with("else")
}

/// 编译 ForStatement 节点为循环指令序列
///
/// 生成循环指令实现 for 循环：
/// - 编译可迭代表达式，将结果压入操作数栈
/// - 发射 `ForInit` 指令（占位 loop_end）
/// - 编译循环体
/// - 发射 `ForNext` 指令（跳回循环起始）
/// - 修补 `ForInit` 的 loop_end 为循环结束位置
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: ForStatement 类型的 RedTree 节点引用
/// - `source`: 模板源文本
fn compile_for(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    let children: Vec<_> = tree.children().collect();
    let mut pattern = ForPattern::Identifier("_".to_string());
    let mut state = ForParseState::Pattern;
    let mut for_init_ip: Option<usize> = None;
    let mut loop_start: Option<usize> = None;

    for child in &children {
        let kind: JinjaElementType = child.kind();
        let text = child.text(source);
        let trimmed = text.trim();

        match state {
            ForParseState::Pattern => {
                match kind {
                    JinjaElementType::Identifier => {
                        if trimmed == "for" {
                            continue;
                        }
                        pattern = parse_pattern(trimmed);
                        state = ForParseState::In;
                    }
                    JinjaElementType::Expression => {
                        let sub_children: Vec<_> = child.children().collect();
                        if sub_children.len() == 1 {
                            let sub_kind: JinjaElementType = sub_children[0].kind();
                            if sub_kind == JinjaElementType::Identifier {
                                let sub_text = sub_children[0].text(source);
                                pattern = parse_pattern(sub_text.trim());
                            }
                        }
                        state = ForParseState::In;
                    }
                    _ => {
                        state = ForParseState::In;
                    }
                }
            }
            ForParseState::In => {
                if kind == JinjaElementType::Identifier && trimmed == "in" {
                    state = ForParseState::Iterable;
                    continue;
                }
                if trimmed == "in" {
                    state = ForParseState::Iterable;
                    continue;
                }
                state = ForParseState::Iterable;
            }
            ForParseState::Iterable => {
                match kind {
                    JinjaElementType::Expression
                    | JinjaElementType::Identifier
                    | JinjaElementType::Literal
                    | JinjaElementType::Filter
                    | JinjaElementType::Function => {
                        compile_single_expr_child(builder, child, source);
                        let ip = builder.emit_for_init(pattern.clone());
                        for_init_ip = Some(ip);
                        loop_start = Some(builder.current_ip());
                        state = ForParseState::Body;
                    }
                    JinjaElementType::Text => {
                        if !trimmed.is_empty() {
                            compile_expr_text(builder, trimmed);
                            let ip = builder.emit_for_init(pattern.clone());
                            for_init_ip = Some(ip);
                            loop_start = Some(builder.current_ip());
                            state = ForParseState::Body;
                        }
                    }
                    _ => {
                        compile_expr_text(builder, trimmed);
                        let ip = builder.emit_for_init(pattern.clone());
                        for_init_ip = Some(ip);
                        loop_start = Some(builder.current_ip());
                        state = ForParseState::Body;
                        compile_node(builder, child, source)?;
                    }
                }
            }
            ForParseState::Body => {
                compile_node(builder, child, source)?;
            }
        }
    }

    if let (Some(init_ip), Some(start)) = (for_init_ip, loop_start) {
        builder.emit_for_next(start);
        builder.patch_jump(init_ip);
    }

    Ok(())
}

/// For 语句解析状态
enum ForParseState {
    /// 正在解析循环变量模式
    Pattern,
    /// 正在跳过 in 关键字
    In,
    /// 正在解析可迭代表达式
    Iterable,
    /// 正在解析循环体
    Body,
}

/// 解析循环变量模式文本
///
/// 支持单个标识符（如 `item`）和元组解构（如 `key, value`）。
///
/// # 参数
/// - `text`: 循环变量模式文本
fn parse_pattern(text: &str) -> ForPattern {
    let names: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
    if names.len() == 1 {
        ForPattern::Identifier(names[0].to_string())
    } else {
        ForPattern::Tuple(names.iter().map(|s| s.to_string()).collect())
    }
}

/// 编译 Block 节点为作用域指令序列
///
/// 发射 `ScopeBegin`，跳过块名称节点编译体内容，最后发射 `ScopeEnd`。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `tree`: Block 类型的 RedTree 节点引用
/// - `source`: 模板源文本
fn compile_block(
    builder: &mut IRBuilder,
    tree: &RedTree<'_, JinjaLanguage>,
    source: &SourceText,
) -> TemplateResult<()> {
    builder.scope_begin();
    let children: Vec<_> = tree.children().collect();
    let mut found_name = false;
    for child in &children {
        if !found_name {
            let kind: JinjaElementType = child.kind();
            match kind {
                JinjaElementType::Identifier => {
                    let text = child.text(source);
                    let trimmed = text.trim();
                    if trimmed != "block" && trimmed != "endblock" {
                        found_name = true;
                    }
                }
                _ => {
                    found_name = true;
                    compile_node(builder, child, source)?;
                }
            }
        } else {
            compile_node(builder, child, source)?;
        }
    }
    builder.scope_end();
    Ok(())
}

/// 将标识符路径文本编译为栈操作指令
///
/// 支持点号路径（如 `user.name`）编译为 `load_var` + `field_access` 指令序列。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 标识符路径文本
fn compile_identifier_path(builder: &mut IRBuilder, text: &str) {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() {
        builder.load_var(text.to_string());
        return;
    }
    builder.load_var(parts[0].trim().to_string());
    for part in &parts[1..] {
        let field = part.trim().to_string();
        if !field.is_empty() {
            builder.field_access(field);
        }
    }
}

/// 将字面量文本编译为栈操作指令
///
/// 支持布尔值、空值、数字和字符串字面量。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 字面量文本
fn compile_literal(builder: &mut IRBuilder, text: &str) {
    if !compile_literal_text(builder, text) {
        builder.push_string(text.to_string());
    }
}

/// 将表达式文本编译为栈操作指令（回退解析）
///
/// 当 RedTree 子节点结构不足以提供完整信息时，
/// 回退到基于文本的递归下降解析，按优先级依次尝试：
/// 字面量 → not → and/or → 比较 → 管道过滤器 → 加减 → 乘除 → 函数调用 → 标识符路径
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
fn compile_expr_text(builder: &mut IRBuilder, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        builder.push_null();
        return;
    }

    if compile_literal_text(builder, text) {
        return;
    }
    if compile_not_text(builder, text) {
        return;
    }
    if compile_and_or_text(builder, text) {
        return;
    }
    if compile_comparison_text(builder, text) {
        return;
    }
    if compile_pipe_filter_text(builder, text) {
        return;
    }
    if compile_add_sub_text(builder, text) {
        return;
    }
    if compile_mul_div_text(builder, text) {
        return;
    }
    if compile_call_text(builder, text) {
        return;
    }

    compile_identifier_path(builder, text);
}

/// 尝试将表达式文本编译为字面量栈操作
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若匹配字面量则返回 true（已发射指令），否则返回 false
fn compile_literal_text(builder: &mut IRBuilder, text: &str) -> bool {
    if text == "true" || text == "True" {
        builder.push_bool(true);
        return true;
    }
    if text == "false" || text == "False" {
        builder.push_bool(false);
        return true;
    }
    if text == "none" || text == "None" || text == "null" {
        builder.push_null();
        return true;
    }
    if let Ok(n) = text.parse::<f64>() {
        builder.push_number(n);
        return true;
    }
    if (text.starts_with('"') && text.ends_with('"') && text.len() >= 2)
        || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
    {
        builder.push_string(text[1..text.len() - 1].to_string());
        return true;
    }
    false
}

/// 尝试将 not 一元表达式文本编译为栈操作
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含 not 前缀则返回 true（已发射指令），否则返回 false
fn compile_not_text(builder: &mut IRBuilder, text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.starts_with("not ") {
        let inner = text[4..].trim();
        compile_expr_text(builder, inner);
        builder.unary_op(UnaryOp::Not);
        true
    } else {
        false
    }
}

/// 尝试将 and/or 逻辑表达式文本编译为栈操作
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含逻辑运算符则返回 true（已发射指令），否则返回 false
fn compile_and_or_text(builder: &mut IRBuilder, text: &str) -> bool {
    let lower = text.to_lowercase();
    if let Some(pos) = find_keyword_outside_strings(&lower, " and ") {
        let left = text[..pos].trim();
        let right = text[pos + 5..].trim();
        compile_expr_text(builder, left);
        compile_expr_text(builder, right);
        builder.binary_op(BinaryOp::And);
        return true;
    }
    if let Some(pos) = find_keyword_outside_strings(&lower, " or ") {
        let left = text[..pos].trim();
        let right = text[pos + 4..].trim();
        compile_expr_text(builder, left);
        compile_expr_text(builder, right);
        builder.binary_op(BinaryOp::Or);
        return true;
    }
    false
}

/// 尝试将比较表达式文本编译为栈操作
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含比较运算符则返回 true（已发射指令），否则返回 false
fn compile_comparison_text(builder: &mut IRBuilder, text: &str) -> bool {
    let operators = [
        (">=", BinaryOp::Ge),
        ("<=", BinaryOp::Le),
        ("!=", BinaryOp::Ne),
        ("==", BinaryOp::Eq),
        (">", BinaryOp::Gt),
        ("<", BinaryOp::Lt),
    ];
    for (op_str, op) in &operators {
        if let Some(pos) = find_operator_outside_strings(text, op_str) {
            let left = text[..pos].trim();
            let right = text[pos + op_str.len()..].trim();
            compile_expr_text(builder, left);
            compile_expr_text(builder, right);
            builder.binary_op(*op);
            return true;
        }
    }
    false
}

/// 尝试将管道过滤器表达式文本编译为栈操作
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含管道符则返回 true（已发射指令），否则返回 false
fn compile_pipe_filter_text(builder: &mut IRBuilder, text: &str) -> bool {
    let pos = match find_operator_outside_strings(text, "|") {
        Some(p) => p,
        None => return false,
    };
    let left = text[..pos].trim();
    let right = text[pos + 1..].trim();
    compile_expr_text(builder, left);
    let (filter_name, args_text) = parse_filter_call_text(right);
    let arg_count = if args_text.is_empty() {
        0u8
    } else {
        let args: Vec<&str> = args_text
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for arg in &args {
            compile_expr_text(builder, arg);
        }
        args.len() as u8
    };
    builder.call_filter(filter_name, arg_count);
    true
}

/// 尝试将加减法表达式文本编译为栈操作
///
/// 从右向左扫描以保持左结合性，跳过字符串和括号内的运算符。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含加减运算符则返回 true（已发射指令），否则返回 false
fn compile_add_sub_text(builder: &mut IRBuilder, text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut paren_depth = 0usize;

    for i in (1..chars.len()).rev() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == ')' {
            paren_depth += 1;
            continue;
        }
        if ch == '(' {
            paren_depth = paren_depth.saturating_sub(1);
            continue;
        }
        if paren_depth > 0 {
            continue;
        }
        if (ch == '+' || ch == '-') && i > 0 {
            let prev = chars[i - 1];
            if prev.is_whitespace()
                || prev.is_alphanumeric()
                || prev == ')'
                || prev == '\''
                || prev == '"'
            {
                let byte_pos: usize = chars[..i].iter().collect::<String>().len();
                let left = text[..byte_pos].trim();
                let right = text[byte_pos + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    compile_expr_text(builder, left);
                    compile_expr_text(builder, right);
                    let op = if ch == '+' {
                        BinaryOp::Add
                    } else {
                        BinaryOp::Sub
                    };
                    builder.binary_op(op);
                    return true;
                }
            }
        }
    }
    false
}

/// 尝试将乘除法表达式文本编译为栈操作
///
/// 从右向左扫描以保持左结合性，跳过字符串和括号内的运算符。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若包含乘除运算符则返回 true（已发射指令），否则返回 false
fn compile_mul_div_text(builder: &mut IRBuilder, text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut paren_depth = 0usize;

    for i in (1..chars.len()).rev() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == ')' {
            paren_depth += 1;
            continue;
        }
        if ch == '(' {
            paren_depth = paren_depth.saturating_sub(1);
            continue;
        }
        if paren_depth > 0 {
            continue;
        }
        if (ch == '*' || ch == '/' || ch == '%') && i > 0 {
            let prev = chars[i - 1];
            if prev.is_whitespace()
                || prev.is_alphanumeric()
                || prev == ')'
                || prev == '\''
                || prev == '"'
            {
                let byte_pos: usize = chars[..i].iter().collect::<String>().len();
                let left = text[..byte_pos].trim();
                let right = text[byte_pos + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    compile_expr_text(builder, left);
                    compile_expr_text(builder, right);
                    let op = match ch {
                        '*' => BinaryOp::Mul,
                        '/' => BinaryOp::Div,
                        '%' => BinaryOp::Mod,
                        _ => unreachable!(),
                    };
                    builder.binary_op(op);
                    return true;
                }
            }
        }
    }
    false
}

/// 尝试将函数调用表达式文本编译为栈操作
///
/// 函数调用编译为 `push_null` + 编译参数 + `call_filter`，
/// 将函数名作为过滤器名称处理。
///
/// # 参数
/// - `builder`: IR 指令构建器
/// - `text`: 表达式文本
///
/// # 返回值
/// 若是函数调用格式则返回 true（已发射指令），否则返回 false
fn compile_call_text(builder: &mut IRBuilder, text: &str) -> bool {
    let paren_pos = match text.find('(') {
        Some(p) => p,
        None => return false,
    };
    if !text.ends_with(')') {
        return false;
    }
    let callee = text[..paren_pos].trim().to_string();
    if callee.is_empty() {
        return false;
    }
    let args_text = &text[paren_pos + 1..text.len() - 1];
    builder.push_null();
    let arg_count = if args_text.trim().is_empty() {
        0u8
    } else {
        let args: Vec<&str> = args_text
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for arg in &args {
            compile_expr_text(builder, arg);
        }
        args.len() as u8
    };
    builder.call_filter(callee, arg_count);
    true
}

/// 解析过滤器调用文本，提取过滤器名称和参数文本
///
/// # 参数
/// - `text`: 过滤器调用文本（如 `upper` 或 `default("N/A")`）
///
/// # 返回值
/// 元组：(过滤器名称, 参数文本)
fn parse_filter_call_text(text: &str) -> (String, String) {
    let text = text.trim();
    if let Some(paren_pos) = text.find('(') {
        if text.ends_with(')') {
            let name = text[..paren_pos].trim().to_string();
            let args_text = text[paren_pos + 1..text.len() - 1].to_string();
            return (name, args_text);
        }
    }
    (text.to_string(), String::new())
}

/// 在表达式中查找运算符位置（跳过字符串字面量和括号）
///
/// # 参数
/// - `expr`: 表达式文本
/// - `op`: 要查找的运算符
///
/// # 返回值
/// 运算符的起始位置，若未找到则返回 None
fn find_operator_outside_strings(expr: &str, op: &str) -> Option<usize> {
    let chars: Vec<char> = expr.chars().collect();
    let op_chars: Vec<char> = op.chars().collect();
    let op_len = op_chars.len();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut paren_depth = 0usize;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            i += 1;
            continue;
        }

        if ch == '(' {
            paren_depth += 1;
            i += 1;
            continue;
        }

        if ch == ')' {
            paren_depth = paren_depth.saturating_sub(1);
            i += 1;
            continue;
        }

        if paren_depth > 0 {
            i += 1;
            continue;
        }

        if i + op_len <= chars.len() {
            let slice: String = chars[i..i + op_len].iter().collect();
            if slice == op {
                let byte_pos = chars[..i].iter().collect::<String>().len();
                return Some(byte_pos);
            }
        }

        i += 1;
    }
    None
}

/// 在表达式中查找关键字位置（跳过字符串字面量和括号）
///
/// # 参数
/// - `expr`: 表达式文本（已转小写）
/// - `keyword`: 要查找的关键字（含前后空格）
///
/// # 返回值
/// 关键字的起始位置，若未找到则返回 None
fn find_keyword_outside_strings(expr: &str, keyword: &str) -> Option<usize> {
    let chars: Vec<char> = expr.chars().collect();
    let kw_chars: Vec<char> = keyword.chars().collect();
    let kw_len = kw_chars.len();
    let mut in_string = false;
    let mut string_char = ' ';
    let mut paren_depth = 0usize;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            i += 1;
            continue;
        }

        if ch == '(' {
            paren_depth += 1;
            i += 1;
            continue;
        }

        if ch == ')' {
            paren_depth = paren_depth.saturating_sub(1);
            i += 1;
            continue;
        }

        if paren_depth > 0 {
            i += 1;
            continue;
        }

        if i + kw_len <= chars.len() {
            let slice: String = chars[i..i + kw_len].iter().collect();
            if slice == keyword {
                let byte_pos = chars[..i].iter().collect::<String>().len();
                return Some(byte_pos);
            }
        }

        i += 1;
    }
    None
}
