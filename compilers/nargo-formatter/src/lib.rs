#![warn(missing_docs)]

//! Nargo 代码格式化工具，用于统一 Nargo 项目的代码风格。
//!
//! 该模块提供了一个 `NargoFormatter` 结构体，用于格式化 Nargo、TS、TSX、JS、JSX 文件。
//! 它能够处理 Nargo 单文件组件（SFC）的不同区块，包括 `<template>`、`<script>`、`<style>` 和自定义块。

use nargo_ir::{JsExpr, JsStmt, TemplateNodeIR, Trivia};
use nargo_parser::Parser;
use nargo_types::{Error, NargoValue, Result};
use oak_core::source::{SourceBuffer, ToSource};
use oak_json::parse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 格式化配置选项
///
/// 定义了代码格式化的各种配置选项，包括缩进、引号、分号等。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatterConfig {
    /// 缩进大小
    #[serde(alias = "indentSize", default = "default_indent_size")]
    pub indent_size: usize,
    /// 缩进类型（spaces 或 tabs）
    #[serde(alias = "indentType", default = "default_indent_type")]
    pub indent_type: IndentType,
    /// 行宽
    #[serde(alias = "lineWidth", default = "default_line_width")]
    pub line_width: usize,
    /// 是否使用单引号
    #[serde(alias = "singleQuote", default = "default_single_quote")]
    pub single_quote: bool,
    /// 是否使用分号
    #[serde(default = "default_semi")]
    pub semi: bool,
    /// 尾随逗号选项
    #[serde(alias = "trailingComma", default = "default_trailing_comma")]
    pub trailing_comma: TrailingComma,
    /// 是否在属性后添加空格
    #[serde(alias = "attrSpacing", default = "default_attr_spacing")]
    pub attr_spacing: bool,
    /// 是否在自闭合标签前添加空格
    #[serde(alias = "selfClosingSpace", default = "default_self_closing_space")]
    pub self_closing_space: bool,
    /// 是否在标签之间添加空行
    #[serde(alias = "tagSpacing", default = "default_tag_spacing")]
    pub tag_spacing: bool,
    /// 是否在箭头函数参数周围添加括号
    #[serde(alias = "arrowParens", default = "default_arrow_parens")]
    pub arrow_parens: bool,
    /// 是否在对象字面量大括号周围添加空格
    #[serde(alias = "objectSpacing", default = "default_object_spacing")]
    pub object_spacing: bool,
    /// 是否在数组字面量方括号周围添加空格
    #[serde(alias = "arraySpacing", default = "default_array_spacing")]
    pub array_spacing: bool,
    /// 是否在函数参数周围添加空格
    #[serde(alias = "functionSpacing", default = "default_function_spacing")]
    pub function_spacing: bool,
    /// 是否在区块之间添加空行
    #[serde(alias = "blockSpacing", default = "default_block_spacing")]
    pub block_spacing: bool,
}

/// 默认缩进大小
fn default_indent_size() -> usize {
    2
}

/// 默认缩进类型
fn default_indent_type() -> IndentType {
    IndentType::Spaces
}

/// 默认行宽
fn default_line_width() -> usize {
    80
}

/// 默认是否使用单引号
fn default_single_quote() -> bool {
    true
}

/// 默认是否使用分号
fn default_semi() -> bool {
    false
}

/// 默认尾随逗号选项
fn default_trailing_comma() -> TrailingComma {
    TrailingComma::Es5
}

/// 默认是否在属性后添加空格
fn default_attr_spacing() -> bool {
    true
}

/// 默认是否在自闭合标签前添加空格
fn default_self_closing_space() -> bool {
    true
}

/// 默认是否在标签之间添加空行
fn default_tag_spacing() -> bool {
    true
}

/// 默认是否在箭头函数参数周围添加括号
fn default_arrow_parens() -> bool {
    true
}

/// 默认是否在对象字面量大括号周围添加空格
fn default_object_spacing() -> bool {
    true
}

/// 默认是否在数组字面量方括号周围添加空格
fn default_array_spacing() -> bool {
    true
}

/// 默认是否在函数参数周围添加空格
fn default_function_spacing() -> bool {
    true
}

/// 默认是否在区块之间添加空行
fn default_block_spacing() -> bool {
    true
}

/// 缩进类型
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum IndentType {
    /// 使用空格
    #[serde(alias = "spaces")]
    #[default]
    Spaces,
    /// 使用制表符
    #[serde(alias = "tabs")]
    Tabs,
}

/// 尾随逗号选项
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum TrailingComma {
    /// 无尾随逗号
    #[serde(alias = "none")]
    None,
    /// ES5 风格（对象和数组）
    #[serde(alias = "es5")]
    #[default]
    Es5,
    /// 所有可能的地方
    #[serde(alias = "all")]
    All,
}

/// 格式化 Nargo 相关文件的工具
///
/// 支持格式化 Nargo、TS、TSX、JS、JSX 文件，能够处理 Nargo 单文件组件的不同区块。
pub struct NargoFormatter {
    /// 解析器实例
    parser: Parser<'static>,
    /// 解析器注册表
    registry: Arc<nargo_parser::ParserRegistry>,
    /// 格式化配置
    config: FormatterConfig,
    /// 缩进字符串缓存
    indent_cache: std::collections::HashMap<usize, String>,
}

impl NargoFormatter {
    /// 创建新的格式化器实例，使用默认配置
    ///
    /// # 返回值
    /// 格式化器实例
    pub fn new() -> Self {
        Self::with_config(FormatterConfig::default())
    }

    /// 创建新的格式化器实例，使用自定义配置
    ///
    /// # 参数
    /// - `config`: 格式化配置
    ///
    /// # 返回值
    /// 格式化器实例
    pub fn with_config(config: FormatterConfig) -> Self {
        let mut registry = nargo_parser::ParserRegistry::new();

        // 注册模板解析器
        registry.register_template_parser("vue", std::sync::Arc::new(nargo_parser::OakVueTemplateParser));

        // 注册脚本解析器
        registry.register_script_parser("ts", std::sync::Arc::new(nargo_parser::OakTypeScriptParser));
        registry.register_script_parser("js", std::sync::Arc::new(nargo_parser::OakTypeScriptParser));

        // 注册样式解析器
        registry.register_style_parser("css", std::sync::Arc::new(nargo_parser::OakCssParser));

        // 注册元数据解析器
        // 暂时不注册 json 解析器，因为 nargo_parser 没有导出 JsonOaksParser

        let registry = std::sync::Arc::new(registry);
        let parser = Parser::new("format".to_string(), "", registry.clone());
        Self { parser, registry, config, indent_cache: std::collections::HashMap::new() }
    }

    /// 从文件加载配置
    ///
    /// # 参数
    /// - `file_path`: 配置文件路径
    ///
    /// # 返回值
    /// 配置实例
    pub fn load_config_from_file(file_path: &str) -> Result<FormatterConfig> {
        let content = std::fs::read_to_string(file_path)?;
        let config: FormatterConfig = serde_json::from_str(&content).map_err(|e| nargo_types::Error::external_error("serde_json".to_string(), e.to_string(), nargo_types::Span::default()))?;
        Ok(config)
    }

    /// 保存配置到文件
    ///
    /// # 参数
    /// - `config`: 配置实例
    /// - `file_path`: 配置文件路径
    ///
    /// # 返回值
    /// 操作结果
    pub fn save_config_to_file(config: &FormatterConfig, file_path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(config).map_err(|e| nargo_types::Error::external_error("serde_json".to_string(), e.to_string(), nargo_types::Span::default()))?;
        std::fs::write(file_path, content)?;
        Ok(())
    }

    /// 格式化源代码
    ///
    /// # 参数
    /// - `source`: 源代码字符串
    ///
    /// # 返回值
    /// 格式化后的代码
    pub async fn format(&mut self, source: &str) -> Result<String> {
        // 暂时禁用 JSON 格式化，因为 oak-json 的 formatter 不可用
        // if let Ok(json_value) = parse(source) {
        //     let doc = json_value.to_doc();
        //     let mut buffer = SourceBuffer::new();
        //     doc.to_source(&mut buffer);
        //     return Ok(buffer.to_string());
        // }

        // 预分配输出字符串容量，减少内存分配
        let mut output = String::with_capacity(source.len() * 2);

        // 创建新的解析器实例，使用当前源代码
        let mut parser = Parser::new("format".to_string(), source, self.registry.clone());
        let ir = parser.parse_all()?;

        if let Some(template) = &ir.template {
            output.push_str("<template>\n");
            for node in &template.nodes {
                let formatted = self.format_template_node(node, 1);
                if !formatted.is_empty() {
                    output.push_str(&formatted);
                }
            }
            output.push_str("</template>\n\n");
        }

        if let Some(script) = &ir.script {
            output.push_str("<script>\n");
            for stmt in &script.body {
                let formatted = self.format_js_stmt(stmt, 1);
                let trimmed = formatted.trim();
                if !trimmed.is_empty() && trimmed != ";" {
                    output.push_str(&formatted);
                    output.push('\n');
                }
            }
            output.push_str("</script>\n\n");
        }

        for style in &ir.styles {
            output.push_str("<style");
            if style.lang != "css" {
                output.push_str(&format!(" lang=\"{}\"", style.lang));
            }
            if style.scoped {
                output.push_str(" scoped");
            }
            output.push_str(">");
            output.push_str(&style.code);
            if !style.code.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("</style>\n\n");
        }

        for block in &ir.custom_blocks {
            output.push_str(&format!("<{}", block.name));
            for (k, v) in &block.attributes {
                output.push_str(&format!(" {}=\"{}\"", k, v));
            }
            output.push_str(">");
            output.push_str(&block.content);
            if !block.content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&format!("</{}>\n\n", block.name));
        }

        Ok(output.trim().to_string() + "\n")
    }

    /// 获取缩进字符串
    ///
    /// # 参数
    /// - `indent`: 缩进级别
    ///
    /// # 返回值
    /// 缩进字符串
    fn get_indent(&mut self, indent: usize) -> String {
        // 从缓存中获取缩进字符串，如果不存在则生成并缓存
        self.indent_cache
            .entry(indent)
            .or_insert_with(|| match self.config.indent_type {
                IndentType::Spaces => " ".repeat(self.config.indent_size * indent),
                IndentType::Tabs => "\t".repeat(indent),
            })
            .clone()
    }

    /// 格式化模板节点
    ///
    /// # 参数
    /// - `node`: 模板节点
    /// - `indent`: 缩进级别
    ///
    /// # 返回值
    /// 格式化后的字符串
    fn format_template_node(&mut self, node: &TemplateNodeIR, indent: usize) -> String {
        let indent_str = self.get_indent(indent);
        match node {
            TemplateNodeIR::Text(text, _, _) => {
                let trimmed = text.trim();
                if trimmed.is_empty() { String::new() } else { format!("{}{}\n", indent_str, trimmed) }
            }
            TemplateNodeIR::Element(el) => {
                let trivia_str = self.format_trivia(&el.trivia, indent);
                // 从标签中提取标签名，移除 < 和 > 字符
                let tag_name = el.tag.trim_start_matches('<').trim_end_matches('>');

                // 预分配字符串容量，减少内存分配
                let mut s = String::with_capacity(1024);
                s.push_str(&trivia_str);
                s.push_str(&indent_str);
                s.push('<');
                s.push_str(tag_name);

                for attr in &el.attributes {
                    let name = if attr.is_directive {
                        if attr.name.starts_with("v-") { attr.name.clone() } else { format!("v-{}", attr.name) }
                    }
                    else if attr.is_dynamic {
                        if attr.name.starts_with(':') { attr.name.clone() } else { format!(":{}", attr.name) }
                    }
                    else {
                        attr.name.clone()
                    };

                    s.push(' ');
                    s.push_str(&name);

                    if let Some(val) = &attr.value {
                        if self.config.attr_spacing {
                            s.push(' ');
                        }
                        s.push('=');
                        s.push('"');
                        s.push_str(val);
                        s.push('"');
                    }
                    else if let Some(val_ast) = &attr.value_ast {
                        if self.config.attr_spacing {
                            s.push(' ');
                        }
                        s.push('=');
                        s.push('"');
                        s.push_str(&self.format_js_expr(val_ast));
                        s.push('"');
                    }
                }

                if el.children.is_empty() {
                    if self.config.self_closing_space {
                        s.push(' ');
                    }
                    s.push_str("/>");
                    s.push('\n');
                }
                else {
                    s.push_str(">\n");
                    for child in &el.children {
                        s.push_str(&self.format_template_node(child, indent + 1));
                    }
                    s.push_str(&indent_str);
                    s.push_str("</");
                    s.push_str(tag_name);
                    s.push('>');
                    s.push('\n');
                }
                s
            }
            TemplateNodeIR::Interpolation(expr) => {
                format!("{}{{{{{}}}}}\n", indent_str, self.format_js_expr(&expr.ast.as_ref().unwrap()))
            }
            TemplateNodeIR::Comment(comment, _, _) => {
                format!("{}<!--{}-->\n", indent_str, comment)
            }
            _ => String::new(),
        }
    }

    /// 格式化 JS 语句
    ///
    /// # 参数
    /// - `stmt`: JS 语句
    /// - `indent`: 缩进级别
    ///
    /// # 返回值
    /// 格式化后的字符串
    fn format_js_stmt(&mut self, stmt: &JsStmt, indent: usize) -> String {
        let indent_str = self.get_indent(indent);
        let trivia = stmt.trivia();
        let trivia_str = self.format_trivia(trivia, indent);

        // 预分配字符串容量，减少内存分配
        let mut result = String::with_capacity(512);
        result.push_str(&trivia_str);
        result.push_str(&indent_str);

        match stmt {
            JsStmt::Expr(expr, _, _) => {
                let expr_str = self.format_js_expr(expr);
                result.push_str(&expr_str);
                if self.config.semi {
                    result.push(';');
                }
                result.push('\n');
            }
            JsStmt::Import { source, specifiers, .. } => {
                let quote = if self.config.single_quote { "'" } else { "\"" };
                result.push_str("import {");
                result.push_str(&specifiers.join(", "));
                result.push_str("} from ");
                result.push_str(quote);
                result.push_str(source);
                result.push_str(quote);
                result.push_str(";\n");
            }
            JsStmt::Export { declaration, .. } => {
                let decl_str = self.format_js_stmt(declaration, 0);
                if decl_str.starts_with("export") {
                    result.push_str(&decl_str);
                }
                else {
                    result.push_str("export ");
                    result.push_str(&decl_str);
                }
            }
            JsStmt::ExportAll { source, .. } => {
                let quote = if self.config.single_quote { "'" } else { "\"" };
                result.push_str("export * from ");
                result.push_str(quote);
                result.push_str(source);
                result.push_str(quote);
                result.push_str(";\n");
            }
            JsStmt::ExportNamed { source, specifiers, .. } => {
                result.push_str("export {");
                result.push_str(&specifiers.join(", "));
                result.push_str("}");
                if let Some(src) = source {
                    let quote = if self.config.single_quote { "'" } else { "\"" };
                    result.push_str(" from ");
                    result.push_str(quote);
                    result.push_str(src);
                    result.push_str(quote);
                }
                if self.config.semi {
                    result.push(';');
                }
                result.push('\n');
            }
            JsStmt::VariableDecl { kind, id, init, .. } => {
                result.push_str(kind);
                result.push(' ');
                result.push_str(id);
                if let Some(init_expr) = init {
                    result.push_str(" = ");
                    result.push_str(&self.format_js_expr(init_expr));
                }
                if self.config.semi {
                    result.push(';');
                }
                result.push('\n');
            }
            JsStmt::FunctionDecl { id, params, body, .. } => {
                result.push_str("function ");
                result.push_str(id);
                result.push('(');
                result.push_str(&params.join(", "));
                result.push_str(") {\n");
                for sub_stmt in body {
                    result.push_str(&self.format_js_stmt(sub_stmt, indent + 1));
                }
                result.push_str(&indent_str);
                result.push_str("}\n");
            }
            _ => {}
        }

        result
    }

    /// 格式化 JS 表达式
    ///
    /// # 参数
    /// - `expr`: JS 表达式
    ///
    /// # 返回值
    /// 格式化后的字符串
    fn format_js_expr(&self, expr: &JsExpr) -> String {
        let trivia = expr.trivia();
        // 预分配字符串容量，减少内存分配
        let mut result = String::with_capacity(256);
        for comment in &trivia.leading_comments {
            result.push_str("//");
            result.push_str(&comment.content);
            result.push('\n');
        }

        match expr {
            JsExpr::Literal(val, _, _) => {
                result.push_str(&self.format_nargo_value(val));
            }
            JsExpr::Identifier(id, _, _) => {
                result.push_str(id);
            }
            JsExpr::Unary { op, argument, .. } => {
                result.push_str(op);
                result.push_str(&self.format_js_expr(argument));
            }
            JsExpr::Binary { left, op, right, .. } => {
                result.push_str(&self.format_js_expr(left));
                result.push(' ');
                result.push_str(op);
                result.push(' ');
                result.push_str(&self.format_js_expr(right));
            }
            JsExpr::Call { callee, args, .. } => {
                result.push_str(&self.format_js_expr(callee));
                if self.config.function_spacing {
                    result.push(' ');
                }
                result.push('(');
                if self.config.function_spacing && !args.is_empty() {
                    result.push(' ');
                }
                let mut first = true;
                for arg in args {
                    if !first {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_js_expr(arg));
                    first = false;
                }
                if self.config.function_spacing && !args.is_empty() {
                    result.push(' ');
                }
                result.push(')');
            }
            JsExpr::Member { object, property, computed, .. } => {
                result.push_str(&self.format_js_expr(object));
                if *computed {
                    result.push('[');
                    result.push_str(&self.format_js_expr(property));
                    result.push(']');
                }
                else {
                    if let JsExpr::Identifier(ref prop, _, _) = **property {
                        result.push('.');
                        result.push_str(prop);
                    }
                    else {
                        result.push('[');
                        result.push_str(&self.format_js_expr(property));
                        result.push(']');
                    }
                }
            }
            JsExpr::Object(props, _, _) => {
                let mut props_vec: Vec<_> = props.iter().collect();
                props_vec.sort_by(|a, b| a.0.cmp(b.0));
                result.push('{');
                if self.config.object_spacing && !props_vec.is_empty() {
                    result.push(' ');
                }
                let mut first = true;
                for (k, v) in &props_vec {
                    if !first {
                        result.push_str(", ");
                    }
                    result.push_str(k);
                    result.push_str(": ");
                    result.push_str(&self.format_js_expr(v));
                    first = false;
                }
                if self.config.object_spacing && !props_vec.is_empty() {
                    result.push(' ');
                }
                result.push('}');
            }
            JsExpr::Array(elements, _, _) => {
                result.push('[');
                let mut first = true;
                for elem in elements {
                    if !first {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_js_expr(elem));
                    first = false;
                }
                result.push(']');
            }
            JsExpr::ArrowFunction { params, body, .. } => {
                if self.config.arrow_parens || params.len() != 1 {
                    result.push('(');
                    result.push_str(&params.join(", "));
                    result.push_str(") => ");
                }
                else {
                    result.push_str(&params[0]);
                    result.push_str(" => ");
                }
                result.push_str(&self.format_js_expr(body));
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                result.push_str(&self.format_js_expr(test));
                result.push_str(" ? ");
                result.push_str(&self.format_js_expr(consequent));
                result.push_str(" : ");
                result.push_str(&self.format_js_expr(alternate));
            }
            JsExpr::TemplateLiteral { quasis, expressions, .. } => {
                result.push('`');
                for i in 0..quasis.len() {
                    result.push_str(&quasis[i]);
                    if i < expressions.len() {
                        result.push_str("${{");
                        result.push_str(&self.format_js_expr(&expressions[i]));
                        result.push_str("}}");
                    }
                }
                result.push('`');
            }
            _ => {}
        }

        result
    }

    /// 格式化 trivia（注释等）
    ///
    /// # 参数
    /// - `trivia`: trivia 信息
    /// - `indent`: 缩进级别
    ///
    /// # 返回值
    /// 格式化后的字符串
    fn format_trivia(&mut self, trivia: &Trivia, indent: usize) -> String {
        let indent_str = self.get_indent(indent);
        let mut s = String::new();

        for comment in &trivia.leading_comments {
            s.push_str(&format!("{}//{}\n", indent_str, comment.content));
        }

        s
    }

    /// 格式化 Nargo 值
    ///
    /// # 参数
    /// - `val`: Nargo 值
    ///
    /// # 返回值
    /// 格式化后的字符串
    fn format_nargo_value(&self, val: &NargoValue) -> String {
        match val {
            NargoValue::Null => "null".to_string(),
            NargoValue::Bool(b) => b.to_string(),
            NargoValue::Number(n) => n.to_string(),
            NargoValue::String(s) => {
                if self.config.single_quote {
                    format!("'{}'", s)
                }
                else {
                    format!("\"{}\"", s)
                }
            }
            NargoValue::Array(arr) => {
                let items: Vec<_> = arr.iter().map(|v| self.format_nargo_value(v)).collect();
                format!("[{}]", items.join(", "))
            }
            NargoValue::Object(obj) => {
                let items: Vec<_> = obj.iter().map(|(k, v)| format!("{}: {}", k, self.format_nargo_value(v))).collect();
                format!("{{ {} }}", items.join(", "))
            }
            NargoValue::Signal(s) => format!("${}", s),
            NargoValue::Raw(r) => r.clone(),
            _ => format!("{:?}", val),
        }
    }
}
