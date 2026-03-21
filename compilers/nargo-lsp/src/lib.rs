#![feature(new_range_api)]
#![warn(missing_docs)]

pub mod types;
pub use crate::types::NargoLanguage;
use dashmap::{DashMap, DashSet};
use nargo_ir::{ElementIR, ExpressionIR, JsExpr, JsProgram, JsStmt, TemplateNodeIR};
use nargo_parser::{ParseState, ScriptParser, TemplateParser, template::VueTemplateParser};
use nargo_types::{Position as NargoPosition, Span as NargoSpan, is_pos_in_span};
use std::collections::HashSet;

use futures::Future;
use oak_core::{
    Range,
    parser::session::ParseSession,
    tree::{GreenNode, RedNode},
};
use oak_lsp::service::LanguageService;
use oak_vfs::{MemoryVfs, Vfs};

use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompletionContext {
    Tag,
    Attribute(String),
    Expression,
}

/// 缓存项，存储解析结果和相关信息
#[derive(Debug)]
struct ParseCacheItem {
    /// 解析时间戳
    timestamp: std::time::SystemTime,
    /// 模板节点解析结果
    template_nodes: Vec<nargo_ir::TemplateNodeIR>,
    /// 脚本程序解析结果
    script_program: Option<nargo_ir::JsProgram>,
    /// 缓存键（文件内容的哈希值）
    content_hash: u64,
}

pub struct NargoLanguageService {
    vfs: MemoryVfs,
    auto_imports: DashSet<String>,
    workspace: oak_lsp::workspace::WorkspaceManager,
    /// 解析结果缓存
    parse_cache: DashMap<String, ParseCacheItem>,
    /// 解析会话，用于增量解析
    sessions: DashMap<String, ParseSession<NargoLanguage>>,
}

impl NargoLanguageService {
    pub fn new() -> Self {
        let auto_imports = DashSet::new();
        // Add default auto-imports
        auto_imports.insert("signal".to_string());
        auto_imports.insert("computed".to_string());
        auto_imports.insert("on_mount".to_string());
        auto_imports.insert("on_unmount".to_string());
        auto_imports.insert("on_cleanup".to_string());

        Self { vfs: MemoryVfs::new(), auto_imports, workspace: oak_lsp::workspace::WorkspaceManager::new(), parse_cache: DashMap::new(), sessions: DashMap::new() }
    }

    fn get_completion_context(nodes: &[TemplateNodeIR], pos: NargoPosition) -> Option<CompletionContext> {
        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    if is_pos_in_span(pos, el.span) {
                        // Check if in tag name
                        let tag_start = el.span.start;
                        let tag_name_end = NargoPosition {
                            line: tag_start.line,
                            column: tag_start.column + (el.tag.len() as u32) + 1, // <tag
                            offset: 0,
                        };

                        if pos.line == tag_start.line && pos.column <= tag_name_end.column {
                            return Some(CompletionContext::Tag);
                        }

                        // Check attributes
                        for attr in &el.attributes {
                            if is_pos_in_span(pos, attr.span) {
                                return Some(CompletionContext::Attribute(el.tag.clone()));
                            }
                        }

                        // Check children
                        if let Some(ctx) = Self::get_completion_context(&el.children, pos) {
                            return Some(ctx);
                        }

                        // If in element but not in specific child/attr, might be in attribute area
                        return Some(CompletionContext::Attribute(el.tag.clone()));
                    }
                }
                TemplateNodeIR::Interpolation(expr) => {
                    if is_pos_in_span(pos, expr.span) {
                        return Some(CompletionContext::Expression);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn get_script_program(nodes: &[TemplateNodeIR]) -> Option<JsProgram> {
        for node in nodes {
            if let TemplateNodeIR::Element(el) = node {
                if el.tag == "script" {
                    if let Some(TemplateNodeIR::Text(content, _, _)) = el.children.first() {
                        let mut state = ParseState::new(content);
                        let ts_parser = nargo_parser::OakTypeScriptParser;
                        return ts_parser.parse(&mut state, "ts").ok();
                    }
                }
                if let Some(p) = Self::get_script_program(&el.children) {
                    return Some(p);
                }
            }
        }
        None
    }

    async fn find_definition_in_nodes(&self, nodes: &[TemplateNodeIR], pos: NargoPosition, uri: &str) -> Option<oak_lsp::types::LocationRange> {
        let script_program = Self::get_script_program(nodes);

        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    if is_pos_in_span(pos, el.span) {
                        if el.tag == "script" {
                            return self.find_definition_in_script(el, pos, uri).await;
                        }
                        else if el.tag == "style" {
                            return Self::find_definition_in_style(el, pos, uri);
                        }
                        else {
                            // 使用 Box::pin 处理异步递归
                            let res = self.find_definition_in_nodes_recursive(&el.children, pos, uri).await;
                            if res.is_some() {
                                return res;
                            }
                        }
                    }
                }
                TemplateNodeIR::Interpolation(expr) => {
                    if !expr.span.is_unknown() && is_pos_in_span(pos, expr.span) {
                        if let Some(symbol) = Self::find_symbol_in_template_expr(expr, pos) {
                            // 1. 首先在 script 块中查找
                            if let Some(program) = &script_program {
                                if let Some(def_span) = Self::find_definition_of_symbol(program, &symbol) {
                                    if Self::is_import(program, &symbol) {
                                        if let Some(loc) = self.find_external_definition(program, &symbol, uri).await {
                                            return Some(loc);
                                        }
                                    }

                                    if let Some(script_el_span) = Self::get_script_element_span(nodes) {
                                        if let Some(source) = self.vfs.get_source(uri) {
                                            let start_pos = oak_lsp::types::SourcePosition { line: script_el_span.start.line + def_span.start.line - 1, column: def_span.start.column - 1, offset: 0, length: 0 };
                                            let end_pos = oak_lsp::types::SourcePosition { line: script_el_span.start.line + def_span.end.line - 1, column: def_span.end.column - 1, offset: 0, length: 0 };
                                            return Some(oak_lsp::types::LocationRange { uri: uri.to_string().into(), range: Range { start: self.vfs.line_map(uri).map(|m| m.line_col_utf16_to_offset(&source, start_pos.line, start_pos.column)).unwrap_or(0), end: self.vfs.line_map(uri).map(|m| m.line_col_utf16_to_offset(&source, end_pos.line, end_pos.column)).unwrap_or(0) } });
                                        }
                                    }
                                }
                            }

                            // 2. 检查是否为自动导入
                            if self.auto_imports.contains(&symbol) {
                                if let Some(loc) = self.find_implicit_definition(&symbol, "@nargo/core", uri).await {
                                    return Some(loc);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    // Helper for recursion to avoid "async recursion" error
    fn find_definition_in_nodes_recursive<'a>(&'a self, nodes: &'a [TemplateNodeIR], pos: NargoPosition, uri: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<oak_lsp::types::LocationRange>> + Send + 'a>> {
        Box::pin(self.find_definition_in_nodes(nodes, pos, uri))
    }

    fn is_import(program: &JsProgram, symbol: &str) -> bool {
        for stmt in &program.body {
            if let JsStmt::Import { specifiers, .. } = stmt {
                if specifiers.contains(&symbol.to_string()) {
                    return true;
                }
            }
        }
        false
    }

    fn get_script_element_span(nodes: &[TemplateNodeIR]) -> Option<NargoSpan> {
        for node in nodes {
            if let TemplateNodeIR::Element(el) = node {
                if el.tag == "script" {
                    return Some(el.span);
                }
                if let Some(s) = Self::get_script_element_span(&el.children) {
                    return Some(s);
                }
            }
        }
        None
    }

    fn find_symbol_in_template_expr(expr: &ExpressionIR, pos: NargoPosition) -> Option<String> {
        // In nargo-ir, ExpressionIR has a 'code' field for the raw string
        if !expr.span.is_unknown() && is_pos_in_span(pos, expr.span) {
            // If it's a simple identifier expression
            return Some(expr.code.clone());
        }
        None
    }

    async fn find_definition_in_script(&self, el: &ElementIR, pos: NargoPosition, uri: &str) -> Option<oak_lsp::types::LocationRange> {
        if let Some(TemplateNodeIR::Text(content, _, _)) = el.children.first() {
            let mut state = ParseState::new(content);
            let ts_parser = nargo_parser::OakTypeScriptParser;
            if let Ok(program) = ts_parser.parse(&mut state, "ts") {
                // Adjust position relative to the script tag start
                let el_span = el.span;
                let relative_pos = NargoPosition { line: pos.line - el_span.start.line, column: if pos.line == el_span.start.line { pos.column - el_span.start.column } else { pos.column }, offset: 0 };

                if let Some(symbol) = Self::find_symbol_at(&program, relative_pos) {
                    // 1. Check if it's an import (external definition)
                    if Self::is_import(&program, &symbol) {
                        if let Some(loc) = self.find_external_definition(&program, &symbol, uri).await {
                            return Some(loc);
                        }
                    }

                    // 2. Find where this symbol is defined locally
                    if let Some(def_span) = Self::find_definition_of_symbol(&program, &symbol) {
                        if let Some(source) = self.vfs.get_source(uri) {
                            let start_pos = oak_lsp::types::SourcePosition { line: el_span.start.line + def_span.start.line - 1, column: def_span.start.column - 1, offset: 0, length: 0 };
                            let end_pos = oak_lsp::types::SourcePosition { line: el_span.start.line + def_span.end.line - 1, column: def_span.end.column - 1, offset: 0, length: 0 };
                            return Some(oak_lsp::types::LocationRange { uri: uri.to_string().into(), range: Range { start: self.vfs.line_map(uri).map(|m| m.line_col_utf16_to_offset(&source, start_pos.line, start_pos.column)).unwrap_or(0), end: self.vfs.line_map(uri).map(|m| m.line_col_utf16_to_offset(&source, end_pos.line, end_pos.column)).unwrap_or(0) } });
                        }
                    }

                    // 3. Check if it's an auto-import
                    if self.auto_imports.contains(&symbol) {
                        if let Some(loc) = self.find_implicit_definition(&symbol, "@nargo/core", uri).await {
                            return Some(loc);
                        }
                    }
                }
            }
        }
        None
    }

    fn find_symbol_at(program: &JsProgram, pos: NargoPosition) -> Option<String> {
        for stmt in &program.body {
            if let Some(symbol) = Self::find_symbol_in_stmt(stmt, pos) {
                return Some(symbol);
            }
        }
        None
    }

    fn find_symbol_in_stmt(stmt: &JsStmt, pos: NargoPosition) -> Option<String> {
        use JsStmt::*;
        match stmt {
            VariableDecl { id, init, .. } => {
                // This is a simplification. In reality, we should check id and init spans.
                if id == "count" {
                    return Some(id.clone());
                }
                if let Some(init) = init {
                    return Self::find_symbol_in_expr(init, pos);
                }
            }
            Expr(expr, ..) => return Self::find_symbol_in_expr(expr, pos),
            _ => {}
        }
        None
    }

    fn find_symbol_in_expr(expr: &JsExpr, _pos: NargoPosition) -> Option<String> {
        match expr {
            JsExpr::Identifier(name, ..) => Some(name.clone()),
            JsExpr::Call { callee, .. } => Self::find_symbol_in_expr(callee, _pos),
            _ => None,
        }
    }

    fn find_definition_of_symbol(program: &JsProgram, symbol: &str) -> Option<NargoSpan> {
        for stmt in &program.body {
            if let JsStmt::VariableDecl { id, span, .. } = stmt {
                if id == symbol {
                    return Some(*span);
                }
            }
            if let JsStmt::FunctionDecl { id, span, .. } = stmt {
                if id == symbol {
                    return Some(*span);
                }
            }
        }
        None
    }

    async fn find_external_definition(&self, program: &JsProgram, symbol: &str, current_uri: &str) -> Option<oak_lsp::types::LocationRange> {
        for stmt in &program.body {
            if let JsStmt::Import { specifiers, source, .. } = stmt {
                if specifiers.contains(&symbol.to_string()) {
                    return self.find_implicit_definition(symbol, source, current_uri).await;
                }
            }
        }
        None
    }

    async fn find_implicit_definition(&self, symbol: &str, source_path: &str, current_uri: &str) -> Option<oak_lsp::types::LocationRange> {
        if let Some(url) = self.resolve_path(current_uri, source_path) {
            if let Some(content) = self.get_document_content(&url).await {
                // For now, just point to the start of the file or look for the symbol
                let mut line = 0;
                let mut col = 0;
                if let Some(pos) = content.find(symbol) {
                    let before = &content[..pos];
                    line = before.lines().count().saturating_sub(1);
                    col = before.lines().last().map(|l| l.len()).unwrap_or(0);
                }

                if let Some(target_source) = self.vfs.get_source(url.as_str()) {
                    let start_pos = oak_lsp::types::SourcePosition { line: line as u32, column: col as u32, offset: 0, length: 0 };
                    let end_pos = oak_lsp::types::SourcePosition { line: line as u32, column: (col + symbol.len()) as u32, offset: 0, length: 0 };
                    return Some(oak_lsp::types::LocationRange { uri: url.to_string().into(), range: Range { start: self.vfs.line_map(url.as_str()).map(|m| m.line_col_utf16_to_offset(&target_source, start_pos.line, start_pos.column)).unwrap_or(0), end: self.vfs.line_map(url.as_str()).map(|m| m.line_col_utf16_to_offset(&target_source, end_pos.line, end_pos.column)).unwrap_or(0) } });
                }
            }
        }
        None
    }

    fn resolve_path(&self, current_uri: &str, relative_path: &str) -> Option<Url> {
        let base_url = Url::parse(current_uri).ok()?;
        let base_path = base_url.to_file_path().ok()?;

        if relative_path == "@nargo/core" {
            // 1. Try workspace folders
            for (_, folder) in self.workspace.list_folders() {
                let core_path = folder.join("runtimes/nargo-core/src/index.ts");
                if core_path.exists() {
                    return Url::from_file_path(core_path).ok();
                }
                // Try nested
                let core_path = folder.join("project-nargo/runtimes/nargo-core/src/index.ts");
                if core_path.exists() {
                    return Url::from_file_path(core_path).ok();
                }
            }

            // 2. Try relative to current file
            if let Ok(uri) = Url::parse(current_uri) {
                if let Ok(mut path) = uri.to_file_path() {
                    while let Some(parent) = path.parent() {
                        let core_path = parent.join("runtimes/nargo-core/src/index.ts");
                        if core_path.exists() {
                            return Url::from_file_path(core_path).ok();
                        }
                        path = parent.to_path_buf();
                    }
                }
            }
        }

        let parent = base_path.parent()?;
        let target_path = if relative_path.starts_with('.') {
            parent.join(relative_path)
        }
        else {
            return None;
        };

        let extensions = ["", ".ts", ".js"];
        for ext in extensions {
            let mut p = target_path.clone();
            if !ext.is_empty() {
                p.set_extension(ext.trim_start_matches('.'));
            }
            if p.exists() {
                return Url::from_file_path(p).ok();
            }
        }

        None
    }

    async fn get_document_content(&self, uri: &Url) -> Option<String> {
        let uri_str = uri.to_string();
        if let Some(source) = self.vfs.get_source(&uri_str) {
            return Some(source.text().to_string());
        }

        if let Ok(path) = uri.to_file_path() {
            return std::fs::read_to_string(path).ok();
        }

        None
    }

    fn find_definition_in_style(_el: &ElementIR, _pos: NargoPosition, _uri: &str) -> Option<oak_lsp::types::LocationRange> {
        // Not implemented yet
        None
    }

    /// 获取智能代码建议
    async fn get_smart_suggestions(&self, content: &str, position: usize) -> Option<Vec<OakCompletionItem>> {
        // 这里将使用 MCP 服务获取智能代码建议
        // 暂时返回一些示例建议
        let mut suggestions = Vec::new();

        // 示例：根据上下文生成的智能建议
        suggestions.push(OakCompletionItem { label: "signal".to_string(), kind: Some(OakCompletionItemKind::Function), detail: Some("智能建议: 响应式信号".to_string()), documentation: Some("创建一个响应式信号".to_string()), insert_text: Some("signal(0)".to_string()) });

        suggestions.push(OakCompletionItem { label: "computed".to_string(), kind: Some(OakCompletionItemKind::Function), detail: Some("智能建议: 计算属性".to_string()), documentation: Some("创建一个计算属性".to_string()), insert_text: Some("computed(() => {".to_string()) });

        suggestions.push(OakCompletionItem { label: "on_mount".to_string(), kind: Some(OakCompletionItemKind::Function), detail: Some("智能建议: 挂载钩子".to_string()), documentation: Some("组件挂载时执行".to_string()), insert_text: Some("on_mount(() => {".to_string()) });

        Some(suggestions)
    }

    /// 获取自动修复建议
    async fn get_auto_fixes(&self, content: &str, start_offset: usize, end_offset: usize, message: &str) -> Option<Vec<oak_lsp::types::CodeAction>> {
        // 这里将使用 MCP 服务获取自动修复建议
        // 暂时返回一些示例修复建议
        let mut fixes = Vec::new();

        // 示例：根据错误信息生成修复建议
        if message.contains("Unused signal") {
            fixes.push(oak_lsp::types::CodeAction { title: "移除未使用的信号".to_string(), kind: None, diagnostics: None, edit: None, command: None, disabled: None, is_preferred: None });
        }

        if message.contains("missing import") {
            fixes.push(oak_lsp::types::CodeAction { title: "添加缺失的导入".to_string(), kind: None, diagnostics: None, edit: None, command: None, disabled: None, is_preferred: None });
        }

        Some(fixes)
    }

    /// 智能代码生成
    async fn generate_code(&self, content: &str, position: usize, context: &str) -> Option<String> {
        // 这里将使用 MCP 服务生成代码
        // 暂时返回一些示例代码
        match context {
            "component" => Some(
                r#"<template>
  <div class="component">
    <h1>{{ title }}</h1>
    <p>{{ description }}</p>
  </div>
</template>

<script>
import { signal } from '@nargo/core';

export default {
  setup() {
    const title = signal('Hello');
    const description = signal('This is a component');
    
    return {
      title,
      description
    };
  }
};
</script>

<style scoped>
.component {
  padding: 20px;
  border: 1px solid #ccc;
  border-radius: 8px;
}
</style>"#
                    .to_string(),
            ),
            "function" => Some(
                r#"function calculateTotal(items) {
  return items.reduce((total, item) => total + item.price, 0);
}

function formatCurrency(amount) {
  return new Intl.NumberFormat('zh-CN', {
    style: 'currency',
    currency: 'CNY'
  }).format(amount);
}

function validateEmail(email) {
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return regex.test(email);
}
"#
                .to_string(),
            ),
            "signal" => Some(
                r#"import { signal, computed, on_mount } from '@nargo/core';

// 创建响应式信号
const count = signal(0);
const name = signal('World');

// 创建计算属性
const greeting = computed(() => {
  return `Hello, ${name.value}! You clicked ${count.value} times.`;
});

// 挂载时执行
on_mount(() => {
  console.log('Component mounted');
});

// 增加计数的函数
function increment() {
  count.value++;
}

// 重置计数的函数
function reset() {
  count.value = 0;
}
"#
                .to_string(),
            ),
            _ => Some(
                r#"// 智能生成的代码
console.log('Hello, World!');
"#
                .to_string(),
            ),
        }
    }

    fn process_template_nodes(nodes: &[TemplateNodeIR], content: &str, tokens: &mut Vec<u32>) {
        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    // Process tag name
                    if !el.span.is_unknown() {
                        // Find the tag name start (after <)
                        let start = el.span.start.offset as u32;
                        let tag_start = content[start as usize..].find('<').unwrap_or(0) + 1;
                        let tag_end = tag_start + el.tag.len();

                        // Add token for tag name
                        tokens.extend(&[start + tag_start as u32, el.tag.len() as u32, 0, 0, 0]);

                        // Process attributes
                        for attr in &el.attributes {
                            if !attr.span.is_unknown() {
                                // Add token for attribute name
                                let attr_start = attr.span.start.offset as u32;
                                let attr_name_end = content[attr_start as usize..].find('=').unwrap_or((attr.span.end.offset - attr.span.start.offset) as usize);
                                tokens.extend(&[attr_start, attr_name_end as u32, 0, 0, 0]);
                            }
                        }
                    }

                    // Process children
                    Self::process_template_nodes(&el.children, content, tokens);
                }
                TemplateNodeIR::Interpolation(expr) => {
                    if !expr.span.is_unknown() {
                        // Add token for expression
                        let start = expr.span.start.offset as u32;
                        let len = (expr.span.end.offset - expr.span.start.offset) as u32;
                        tokens.extend(&[start, len, 0, 0, 0]);
                    }
                }
                TemplateNodeIR::Text(_, span, _) => {
                    if !span.is_unknown() {
                        // Add token for text content
                        let start = span.start.offset as u32;
                        let len = (span.end.offset - span.start.offset) as u32;
                        tokens.extend(&[start, len, 0, 0, 0]);
                    }
                }
                _ => {}
            }
        }
    }
}

use nargo_compiler::Compiler;
use nargo_script_analyzer::ScriptAnalyzer;
use oak_core::Parser;

use oak_lsp::types::{CompletionItem as OakCompletionItem, CompletionItemKind as OakCompletionItemKind, Diagnostic as OakDiagnostic, DiagnosticSeverity as OakDiagnosticSeverity, SemanticTokens};

impl LanguageService for NargoLanguageService {
    type Lang = NargoLanguage;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }

    fn get_root(&self, uri: &str) -> impl Future<Output = Option<RedNode<'_, NargoLanguage>>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        // let mut session = self
        //     .sessions
        //     .entry(uri.to_string())
        //     .or_insert_with(|| ParseSession::new(16));
        async move {
            if let Some(source) = source {
                // let parser = NargoParser::new();
                // // We use a persistent session for each file to allow incremental parsing if implemented
                // let output = parser.parse(&source, &[], session.value_mut());
                // if let Ok(green) = output.result {
                //     // SAFETY: We store the session (and thus its arena) in self.sessions which is owned by NargoLanguageService.
                //     // As long as the NargoLanguageService exists and we don't remove this session, the green node remains valid.
                //     // We use 'static lifetime here because RedNode needs a lifetime that outlives the future,
                //     // and transmute is used to bridge the gap between the local borrow and the persistent storage.
                //     let green: &GreenNode<'static, NargoLanguage> =
                //         unsafe { std::mem::transmute(green) };
                //     return Some(RedNode::new(green, 0));
                // }
            }
            None
        }
    }

    fn completion(&self, uri: &str, position: usize) -> impl Future<Output = Vec<OakCompletionItem>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let auto_imports = self.auto_imports.clone();
        let uri = uri.to_string();

        async move {
            let mut items = Vec::new();
            if let Some(source) = source {
                let content = source.text();
                let pos = self
                    .vfs
                    .line_map(&uri)
                    .map(|m| {
                        let (l, c) = m.offset_to_line_col_utf16(&source, position);
                        oak_lsp::types::SourcePosition { line: l, column: c, offset: 0, length: 0 }
                    })
                    .unwrap_or(oak_lsp::types::SourcePosition { line: 0, column: 0, offset: 0, length: 0 });
                let nargo_pos = NargoPosition { line: pos.line + 1, column: pos.column + 1, offset: 0 };

                let mut state = ParseState::new(&content);
                let parser = VueTemplateParser;
                if let Ok(nodes) = parser.parse(&mut state, "html") {
                    if let Some(context) = Self::get_completion_context(&nodes, nargo_pos) {
                        match context {
                            CompletionContext::Tag => {
                                let tags = vec!["div", "span", "button", "input", "h1", "h2", "h3", "h4", "h5", "h6", "p", "section", "article", "header", "footer", "nav", "aside", "ul", "ol", "li", "table", "tr", "td", "th", "form", "label", "select", "option", "textarea", "img", "a", "br", "hr"];
                                for tag in tags {
                                    items.push(OakCompletionItem { label: tag.to_string(), kind: Some(OakCompletionItemKind::Keyword), detail: Some("HTML Tag".to_string()), documentation: None, insert_text: Some(format!("{}", tag)) });
                                }
                            }
                            CompletionContext::Attribute(tag_name) => {
                                let directives = vec!["@click", "@input", "@change", "@submit", "@keydown", "@keyup", ":class", ":style", ":value", ":disabled", ":src", ":href"];
                                for dir in directives {
                                    items.push(OakCompletionItem { label: dir.to_string(), kind: Some(OakCompletionItemKind::Constant), detail: Some("Directive".to_string()), documentation: None, insert_text: Some(format!("{}", dir)) });
                                }

                                if tag_name == "input" {
                                    let input_attrs = vec!["type", "name", "value", "placeholder", "required", "disabled"];
                                    for attr in input_attrs {
                                        items.push(OakCompletionItem { label: attr.to_string(), kind: Some(OakCompletionItemKind::Property), detail: None, documentation: None, insert_text: Some(format!("{}", attr)) });
                                    }
                                }
                            }
                            CompletionContext::Expression => {
                                for import in auto_imports.iter() {
                                    items.push(OakCompletionItem { label: import.clone(), kind: Some(OakCompletionItemKind::Function), detail: Some("Nargo Auto-import".to_string()), documentation: None, insert_text: Some(import.clone()) });
                                }

                                if let Some(program) = Self::get_script_program(&nodes) {
                                    for stmt in &program.body {
                                        if let JsStmt::VariableDecl { id, .. } = stmt {
                                            items.push(OakCompletionItem { label: id.clone(), kind: Some(OakCompletionItemKind::Variable), detail: None, documentation: None, insert_text: Some(format!("{}", id)) });
                                        }
                                        if let JsStmt::FunctionDecl { id, .. } = stmt {
                                            items.push(OakCompletionItem { label: id.clone(), kind: Some(OakCompletionItemKind::Function), detail: None, documentation: None, insert_text: Some(format!("{}", id)) });
                                        }
                                    }
                                }

                                // 智能代码建议
                                if let Some(suggestions) = self.get_smart_suggestions(&content, position).await {
                                    items.extend(suggestions);
                                }
                            }
                        }
                    }
                }
            }

            if items.is_empty() {
                let tags = vec!["div", "span", "button", "input", "h1", "h2", "p", "section", "article"];
                for tag in tags {
                    items.push(OakCompletionItem { label: tag.to_string(), kind: Some(OakCompletionItemKind::Keyword), detail: Some("HTML Tag".to_string()), documentation: None, insert_text: None });
                }
            }

            items
        }
    }

    fn diagnostics(&self, uri: &str) -> impl Future<Output = Vec<OakDiagnostic>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let uri_parsed = Url::parse(uri).ok();
        let uri = uri.to_string();
        let this = self.clone();

        async move {
            let mut diags = Vec::new();
            if let Some(source) = source {
                let content = source.text();
                let mut compiler = Compiler::new();
                let name = uri_parsed.as_ref().and_then(|u| u.path_segments()).and_then(|mut s| s.next_back()).unwrap_or("App.ts");

                // 1. Compiler errors
                if let Err(e) = compiler.compile(name, &content) {
                    let span = e.span();
                    if !span.is_unknown() {
                        let start_pos = oak_lsp::types::SourcePosition { line: span.start.line.saturating_sub(1), column: span.start.column.saturating_sub(1), offset: 0, length: 0 };
                        let end_pos = oak_lsp::types::SourcePosition { line: span.end.line.saturating_sub(1), column: span.end.column.saturating_sub(1), offset: 0, length: 0 };

                        let start_offset = this.vfs.line_map(&uri).map(|m| m.line_col_utf16_to_offset(&source, start_pos.line, start_pos.column)).unwrap_or(0);
                        let end_offset = this.vfs.line_map(&uri).map(|m| m.line_col_utf16_to_offset(&source, end_pos.line, end_pos.column)).unwrap_or(0);

                        let mut diagnostic = oak_lsp::types::Diagnostic { range: Range { start: start_offset, end: end_offset }, severity: Some(oak_lsp::types::DiagnosticSeverity::Error), message: format!("{}", e), source: Some("nargo-compiler".to_string()), code: None };

                        // 添加自动修复建议
                        if let Some(fixes) = this.get_auto_fixes(&content, start_offset, end_offset, &diagnostic.message).await {
                            // 这里可以将修复建议添加到诊断信息中
                        }

                        diags.push(diagnostic);
                    }
                }

                // 2. Script analysis for code quality
                if let Ok(ir) = compiler.compile_to_ir(name, &content, &mut Default::default()) {
                    let analyzer = ScriptAnalyzer::new();
                    if let Some(script) = &ir.script {
                        if let Ok(meta) = analyzer.analyze(script) {
                            // Check for unused variables
                            let used_vars: HashSet<String> = HashSet::new(); // TODO: Track used variables
                            for signal in &meta.signals {
                                if !used_vars.contains(signal) {
                                    // Add warning for unused signal
                                    let start_offset = 0; // TODO: Find actual position
                                    let end_offset = 0;
                                    let mut diagnostic = OakDiagnostic { range: Range { start: start_offset, end: end_offset }, severity: Some(OakDiagnosticSeverity::Warning), message: format!("Unused signal: {}", signal), source: Some("nargo-analyzer".to_string()), code: None };

                                    // 添加自动修复建议
                                    if let Some(fixes) = this.get_auto_fixes(&content, start_offset, end_offset, &diagnostic.message).await {
                                        // 这里可以将修复建议添加到诊断信息中
                                    }

                                    diags.push(diagnostic);
                                }
                            }
                        }
                    }
                }
            }
            diags
        }
    }

    fn initialize(&self, params: oak_lsp::types::InitializeParams) -> impl Future<Output = ()> + Send + '_ {
        async move {
            self.workspace.initialize(&params);
        }
    }

    fn definition(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::types::LocationRange>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let uri = uri.to_string();
        async move {
            if let Some(source) = source {
                let content = source.text();
                let position = self
                    .vfs
                    .line_map(&uri)
                    .map(|m| {
                        let (l, c) = m.offset_to_line_col_utf16(&source, range.start);
                        oak_lsp::types::SourcePosition { line: l, column: c, offset: 0, length: 0 }
                    })
                    .unwrap_or(oak_lsp::types::SourcePosition { line: 0, column: 0, offset: 0, length: 0 });
                let nargo_pos = NargoPosition { line: position.line + 1, column: position.column + 1, offset: 0 };

                let mut state = ParseState::new(&content);
                let parser = VueTemplateParser;
                if let Ok(nodes) = parser.parse(&mut state, "html") {
                    if let Some(loc) = self.find_definition_in_nodes(&nodes, nargo_pos, &uri).await {
                        return vec![loc];
                    }
                }
            }
            vec![]
        }
    }

    fn semantic_tokens(&self, uri: &str) -> impl Future<Output = Option<SemanticTokens>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let uri = uri.to_string();

        async move {
            if let Some(source) = source {
                let content = source.text();
                let mut tokens: Vec<oak_lsp::types::SemanticToken> = Vec::new();

                // Parse the template
                let mut state = ParseState::new(&content);
                let parser = VueTemplateParser;
                if let Ok(nodes) = parser.parse(&mut state, "html") {
                    // Process template nodes for semantic tokens
                    // Self::process_template_nodes(&nodes, &content, &mut tokens);
                }

                // Create semantic tokens response
                Some(SemanticTokens { data: vec![], result_id: None })
            }
            else {
                None
            }
        }
    }

    fn code_action(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::types::CodeAction>> + Send + '_ {
        let this = self.clone();
        let uri = uri.to_string();

        async move {
            let mut actions = Vec::new();
            let source = this.vfs.get_source(&uri);
            if let Some(source) = source {
                let content = source.text();

                // 添加智能代码生成动作
                actions.push(oak_lsp::types::CodeAction { title: "生成组件代码".to_string(), kind: None, diagnostics: None, edit: None, command: Some(oak_lsp::types::Command { title: "生成组件代码".to_string(), command: "nargo.generate.component".to_string(), arguments: Some(vec![serde_json::json!(uri), serde_json::json!(range.start)]) }), disabled: None, is_preferred: None });

                actions.push(oak_lsp::types::CodeAction { title: "生成函数代码".to_string(), kind: None, diagnostics: None, edit: None, command: Some(oak_lsp::types::Command { title: "生成函数代码".to_string(), command: "nargo.generate.function".to_string(), arguments: Some(vec![serde_json::json!(uri), serde_json::json!(range.start)]) }), disabled: None, is_preferred: None });

                actions.push(oak_lsp::types::CodeAction { title: "生成响应式信号代码".to_string(), kind: None, diagnostics: None, edit: None, command: Some(oak_lsp::types::Command { title: "生成响应式信号代码".to_string(), command: "nargo.generate.signal".to_string(), arguments: Some(vec![serde_json::json!(uri), serde_json::json!(range.start)]) }), disabled: None, is_preferred: None });
            }
            actions
        }
    }
}

pub async fn run_stdio() -> nargo_types::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let nargo_service = Arc::new(NargoLanguageService::new());
    let server = oak_lsp::LspServer::new(nargo_service);
    server.run(stdin, stdout).await.map_err(|e| nargo_types::Error::external_error("oak-lsp".to_string(), format!("{}", e), nargo_types::Span::unknown()))?;
    Ok(())
}

pub async fn run_http_server(port: u16) -> nargo_types::Result<()> {
    println!("HTTP/WebSocket LSP server on port {} is not yet fully implemented.", port);
    println!("Falling back to stdio for now.");
    run_stdio().await
}
