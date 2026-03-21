#![warn(missing_docs)]

//! Handlebars 前端模块

use crate::frontend::Frontend;
use crate::ir::*;
use crate::error::TemplateResult;
use oak_core::{
    SourceText,
    parser::{ParseSession, Parser},
    tree::RedTree,
};
use oak_handlebars::{HandlebarsElementType, HandlebarsLanguage, HandlebarsParser};

/// Handlebars 模板前端
///
/// 使用 `oak_handlebars::HandlebarsParser` 将模板源码解析为 RedTree，
/// 遍历 RedTree 编译为扁平 IR 指令序列，由 VM 执行渲染。
pub struct HandlebarsFrontend {
    /// Handlebars 语言配置
    language: HandlebarsLanguage,
}

impl HandlebarsFrontend {
    /// 创建新的 Handlebars 前端实例
    pub fn new() -> Self {
        Self {
            language: HandlebarsLanguage::default(),
        }
    }
}

impl Default for HandlebarsFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for HandlebarsFrontend {
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR> {
        let source_text = SourceText::new(source);
        let parser = HandlebarsParser::new(&self.language);
        let mut cache = ParseSession::<HandlebarsLanguage>::default();
        let parse_output = parser.parse(&source_text, &[], &mut cache);
        let green_node = parse_output
            .result
            .map_err(|e| crate::error::TemplateError::Syntax(format!("Handlebars parse error: {}", e)))?;
        let red_tree = RedTree::<HandlebarsLanguage>::new(green_node);
        let mut builder = IRBuilder::new();
        compile_children(&red_tree, &source_text, &mut builder)?;
        Ok(builder.build())
    }

    fn name(&self) -> &str {
        "handlebars"
    }

    fn default_extension(&self) -> &str {
        "hbs"
    }
}

fn compile_children<L: oak_core::Language>(
    tree: &RedTree<'_, L>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    for child in tree.children() {
        let kind: HandlebarsElementType = child.kind();
        compile_element(child, kind, source, builder)?;
    }
    Ok(())
}

fn compile_element<L: oak_core::Language>(
    tree: RedTree<'_, L>,
    kind: HandlebarsElementType,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    match kind {
        HandlebarsElementType::ContentNode | HandlebarsElementType::Content => {
            let text = tree.text(source);
            if !text.is_empty() {
                builder.output_text(text.into_owned());
            }
            Ok(())
        }
        HandlebarsElementType::Mustache => compile_mustache(tree, source, builder),
        HandlebarsElementType::Block => compile_block(tree, source, builder),
        HandlebarsElementType::InverseBlock => compile_inverse_block(tree, source, builder),
        HandlebarsElementType::Partial => compile_partial(tree, source, builder),
        HandlebarsElementType::CommentNode | HandlebarsElementType::Comment => Ok(()),
        _ => Ok(()),
    }
}

fn compile_mustache<L: oak_core::Language>(
    tree: RedTree<'_, L>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    let mut is_unescaped = false;
    let mut expr_text = String::new();

    for child in tree.children() {
        let child_kind: HandlebarsElementType = child.kind();
        match child_kind {
            HandlebarsElementType::OpenUnescaped | HandlebarsElementType::CloseUnescaped => {
                is_unescaped = true;
            }
            HandlebarsElementType::Expression => {
                expr_text = extract_expression_text(&child, source);
            }
            HandlebarsElementType::Path => {
                if expr_text.is_empty() {
                    expr_text = child.text(source).trim().to_string();
                }
            }
            _ => {}
        }
    }

    if expr_text.is_empty() {
        return Ok(());
    }

    compile_expression(&expr_text, builder);

    if is_unescaped {
        builder.output();
    } else {
        builder.call_filter("escape".to_string(), 0);
        builder.output();
    }

    Ok(())
}

fn compile_block<L: oak_core::Language>(
    tree: RedTree<'_, L>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    let mut block_name = String::new();
    let mut params = Vec::new();
    let mut main_children: Vec<RedTree<'_, L>> = Vec::new();
    let mut else_children: Vec<RedTree<'_, L>> = Vec::new();
    let mut in_else = false;

    for child in tree.children() {
        let child_kind: HandlebarsElementType = child.kind();
        match child_kind {
            HandlebarsElementType::OpenBlock
            | HandlebarsElementType::CloseBlock
            | HandlebarsElementType::Close
            | HandlebarsElementType::Whitespace
            | HandlebarsElementType::Newline => {}
            HandlebarsElementType::Expression => {
                let expr_children: Vec<_> = child.children().collect();
                if expr_children.is_empty() {
                    continue;
                }
                let first = expr_children[0];
                let first_kind: HandlebarsElementType = first.kind();
                if first_kind == HandlebarsElementType::Path {
                    let path_text = first.text(source).trim().to_string();
                    if block_name.is_empty() {
                        block_name = path_text;
                    } else {
                        params.push(path_text);
                    }
                }
                for &ec in &expr_children[1..] {
                    let ec_kind: HandlebarsElementType = ec.kind();
                    if ec_kind == HandlebarsElementType::Parameter {
                        params.push(ec.text(source).trim().to_string());
                    }
                }
            }
            HandlebarsElementType::Path => {
                let path_text = child.text(source).trim().to_string();
                if block_name.is_empty() {
                    block_name = path_text;
                }
            }
            HandlebarsElementType::Parameter => {
                params.push(child.text(source).trim().to_string());
            }
            HandlebarsElementType::ElseBlock => {
                in_else = true;
                for else_child in child.children() {
                    else_children.push(else_child);
                }
            }
            _ => {
                if !in_else {
                    main_children.push(child);
                }
            }
        }
    }

    match block_name.as_str() {
        "if" => compile_if_block(params, main_children, else_children, source, builder),
        "unless" => compile_unless_block(params, main_children, else_children, source, builder),
        "each" => compile_each_block(params, main_children, else_children, source, builder),
        _ => {
            for child in main_children {
                let kind: HandlebarsElementType = child.kind();
                compile_element(child, kind, source, builder)?;
            }
            Ok(())
        }
    }
}

fn compile_if_block<L: oak_core::Language>(
    params: Vec<String>,
    main_children: Vec<RedTree<'_, L>>,
    else_children: Vec<RedTree<'_, L>>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    if params.is_empty() {
        builder.push_bool(true);
    } else {
        compile_expression(&params[0], builder);
    }

    let jump_to_else_ip = builder.emit_jump_if_false();

    for child in main_children {
        let kind: HandlebarsElementType = child.kind();
        compile_element(child, kind, source, builder)?;
    }

    if !else_children.is_empty() {
        let jump_to_end_ip = builder.emit_jump();
        builder.patch_jump(jump_to_else_ip);
        for child in else_children {
            let kind: HandlebarsElementType = child.kind();
            compile_element(child, kind, source, builder)?;
        }
        builder.patch_jump(jump_to_end_ip);
    } else {
        builder.patch_jump(jump_to_else_ip);
    }

    Ok(())
}

fn compile_unless_block<L: oak_core::Language>(
    params: Vec<String>,
    main_children: Vec<RedTree<'_, L>>,
    else_children: Vec<RedTree<'_, L>>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    if params.is_empty() {
        builder.push_bool(false);
    } else {
        compile_expression(&params[0], builder);
    }

    builder.unary_op(UnaryOp::Not);

    let jump_to_end_ip = builder.emit_jump_if_false();

    for child in main_children {
        let kind: HandlebarsElementType = child.kind();
        compile_element(child, kind, source, builder)?;
    }

    if !else_children.is_empty() {
        let jump_over_else_ip = builder.emit_jump();
        builder.patch_jump(jump_to_end_ip);
        for child in else_children {
            let kind: HandlebarsElementType = child.kind();
            compile_element(child, kind, source, builder)?;
        }
        builder.patch_jump(jump_over_else_ip);
    } else {
        builder.patch_jump(jump_to_end_ip);
    }

    Ok(())
}

fn compile_each_block<L: oak_core::Language>(
    params: Vec<String>,
    main_children: Vec<RedTree<'_, L>>,
    else_children: Vec<RedTree<'_, L>>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    if params.is_empty() {
        builder.push_null();
    } else {
        compile_expression(&params[0], builder);
    }

    let pattern = if params.len() > 1 {
        ForPattern::Tuple(vec!["this".to_string(), params[1].clone()])
    } else {
        ForPattern::Identifier("this".to_string())
    };

    let for_init_ip = builder.emit_for_init(pattern);
    let loop_start = builder.current_ip();

    for child in main_children {
        let kind: HandlebarsElementType = child.kind();
        compile_element(child, kind, source, builder)?;
    }

    builder.emit_for_next(loop_start);

    if !else_children.is_empty() {
        let jump_over_else_ip = builder.emit_jump();
        builder.patch_jump(for_init_ip);
        for child in else_children {
            let kind: HandlebarsElementType = child.kind();
            compile_element(child, kind, source, builder)?;
        }
        builder.patch_jump(jump_over_else_ip);
    } else {
        builder.patch_jump(for_init_ip);
    }

    Ok(())
}

fn compile_inverse_block<L: oak_core::Language>(
    tree: RedTree<'_, L>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    let mut condition_text = None;
    let mut main_children: Vec<RedTree<'_, L>> = Vec::new();
    let mut else_children: Vec<RedTree<'_, L>> = Vec::new();
    let mut in_else = false;

    for child in tree.children() {
        let child_kind: HandlebarsElementType = child.kind();
        match child_kind {
            HandlebarsElementType::OpenInverseBlock
            | HandlebarsElementType::CloseBlock
            | HandlebarsElementType::Close
            | HandlebarsElementType::Whitespace
            | HandlebarsElementType::Newline => {}
            HandlebarsElementType::Expression => {
                let expr_text = extract_expression_text(&child, source);
                if !expr_text.is_empty() && condition_text.is_none() {
                    condition_text = Some(expr_text);
                }
            }
            HandlebarsElementType::Path => {
                if condition_text.is_none() {
                    let path_text = child.text(source).trim().to_string();
                    if !path_text.is_empty() {
                        condition_text = Some(path_text);
                    }
                }
            }
            HandlebarsElementType::ElseBlock => {
                in_else = true;
                for else_child in child.children() {
                    else_children.push(else_child);
                }
            }
            _ => {
                if !in_else {
                    main_children.push(child);
                }
            }
        }
    }

    let text = condition_text.unwrap_or_else(|| "true".to_string());
    compile_expression(&text, builder);
    builder.unary_op(UnaryOp::Not);

    let jump_to_end_ip = builder.emit_jump_if_false();

    for child in main_children {
        let kind: HandlebarsElementType = child.kind();
        compile_element(child, kind, source, builder)?;
    }

    if !else_children.is_empty() {
        let jump_over_else_ip = builder.emit_jump();
        builder.patch_jump(jump_to_end_ip);
        for child in else_children {
            let kind: HandlebarsElementType = child.kind();
            compile_element(child, kind, source, builder)?;
        }
        builder.patch_jump(jump_over_else_ip);
    } else {
        builder.patch_jump(jump_to_end_ip);
    }

    Ok(())
}

fn compile_partial<L: oak_core::Language>(
    tree: RedTree<'_, L>,
    source: &SourceText,
    builder: &mut IRBuilder,
) -> TemplateResult<()>
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    let mut partial_name = String::new();

    for child in tree.children() {
        let child_kind: HandlebarsElementType = child.kind();
        match child_kind {
            HandlebarsElementType::OpenPartial
            | HandlebarsElementType::Close
            | HandlebarsElementType::Whitespace
            | HandlebarsElementType::Newline => {}
            HandlebarsElementType::Path => {
                if partial_name.is_empty() {
                    partial_name = child.text(source).trim().to_string();
                }
            }
            HandlebarsElementType::Identifier => {
                if partial_name.is_empty() {
                    partial_name = child.text(source).trim().to_string();
                }
            }
            _ => {}
        }
    }

    if partial_name.is_empty() {
        return Ok(());
    }

    builder.push_string(partial_name);
    builder.include(true);

    Ok(())
}

fn extract_expression_text<L: oak_core::Language>(
    tree: &RedTree<'_, L>,
    source: &SourceText,
) -> String
where
    HandlebarsElementType: From<L::ElementType> + From<L::TokenType>,
{
    let mut parts = Vec::new();
    for child in tree.children() {
        let child_kind: HandlebarsElementType = child.kind();
        match child_kind {
            HandlebarsElementType::Path => {
                parts.push(child.text(source).trim().to_string());
            }
            HandlebarsElementType::Parameter => {
                parts.push(child.text(source).trim().to_string());
            }
            HandlebarsElementType::SubExpression => {
                parts.push(child.text(source).trim().to_string());
            }
            _ => {}
        }
    }
    parts.join(" ")
}

fn compile_expression(text: &str, builder: &mut IRBuilder) {
    let text = text.trim();

    if text.is_empty() {
        builder.push_null();
        return;
    }

    if text == "this" {
        builder.load_var("this".to_string());
        return;
    }

    if text == "true" {
        builder.push_bool(true);
        return;
    }

    if text == "false" {
        builder.push_bool(false);
        return;
    }

    if text == "null" || text == "undefined" {
        builder.push_null();
        return;
    }

    if let Ok(n) = text.parse::<f64>() {
        builder.push_number(n);
        return;
    }

    if (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
    {
        if text.len() >= 2 {
            builder.push_string(text[1..text.len() - 1].to_string());
            return;
        }
    }

    if text.starts_with('(') && text.ends_with(')') {
        let inner = text[1..text.len() - 1].trim();
        let parts: Vec<&str> = inner.splitn(2, char::is_whitespace).collect();
        if !parts.is_empty() {
            let callee = parts[0].trim();
            builder.push_null();
            let arg_count = if parts.len() > 1 {
                let args: Vec<&str> = parts[1].split_whitespace().collect();
                for arg in &args {
                    compile_expression(arg, builder);
                }
                args.len() as u8
            } else {
                0
            };
            builder.call_filter(callee.to_string(), arg_count);
            return;
        }
    }

    if text.contains('.') {
        compile_path_expression(text, builder);
        return;
    }

    builder.load_var(text.to_string());
}

fn compile_path_expression(text: &str, builder: &mut IRBuilder) {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() {
        builder.push_null();
        return;
    }

    let first = parts[0].trim();
    if first == "this" {
        builder.load_var("this".to_string());
    } else {
        builder.load_var(first.to_string());
    }

    for part in &parts[1..] {
        let field = part.trim().to_string();
        if !field.is_empty() {
            builder.field_access(field);
        }
    }
}
