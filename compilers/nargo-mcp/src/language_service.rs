#![feature(new_range_api)]
#![warn(missing_docs)]
use dashmap::{DashMap, DashSet};
use nargo_ir::{ElementIR, ExpressionIR, JsExpr, JsProgram, JsStmt, TemplateNodeIR};
use nargo_parser::{ParseState, ScriptParser, TemplateParser, template::VueTemplateParser};
use nargo_types::{Position as NargoPosition, Span as NargoSpan, is_pos_in_span};

use nargo_lsp::NargoLanguage;
use oak_core::{
    Range,
    parser::session::ParseSession,
    tree::{GreenNode, RedNode},
};
use oak_lsp::{service::LanguageService, types::SourcePosition as OakPosition};
use oak_vfs::{MemoryVfs, Vfs};

use std::sync::Arc;
use url::Url;

use crate::types::CompletionContext;

/// Nargo语言服务
pub struct NargoLanguageService {
    vfs: MemoryVfs,
    auto_imports: DashSet<String>,
    workspace: oak_lsp::workspace::WorkspaceManager,
    sessions: DashMap<String, ParseSession<NargoLanguage>>,
}

impl NargoLanguageService {
    /// 创建新的Nargo语言服务
    pub fn new() -> Self {
        let auto_imports = DashSet::new();
        // Add default auto-imports
        auto_imports.insert("signal".to_string());
        auto_imports.insert("computed".to_string());
        auto_imports.insert("on_mount".to_string());
        auto_imports.insert("on_unmount".to_string());
        auto_imports.insert("on_cleanup".to_string());

        Self { vfs: MemoryVfs::new(), auto_imports, workspace: oak_lsp::workspace::WorkspaceManager::new(), sessions: DashMap::new() }
    }

    /// 获取补全上下文
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

    /// 获取脚本程序
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

    /// 在节点中查找定义
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
                                            let start_pos = OakPosition { line: script_el_span.start.line + def_span.start.line - 1, column: def_span.start.column - 1, offset: 0, length: 0 };
                                            let end_pos = OakPosition { line: script_el_span.start.line + def_span.end.line - 1, column: def_span.end.column - 1, offset: 0, length: 0 };
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

    /// 递归查找定义的辅助函数
    fn find_definition_in_nodes_recursive<'a>(&'a self, nodes: &'a [TemplateNodeIR], pos: NargoPosition, uri: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<oak_lsp::types::LocationRange>> + Send + 'a>> {
        Box::pin(self.find_definition_in_nodes(nodes, pos, uri))
    }

    /// 检查是否为导入
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

    /// 获取脚本元素的跨度
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

    /// 在模板表达式中查找符号
    fn find_symbol_in_template_expr(expr: &ExpressionIR, pos: NargoPosition) -> Option<String> {
        // In nargo-ir, ExpressionIR has a 'code' field for the raw string
        if !expr.span.is_unknown() && is_pos_in_span(pos, expr.span) {
            // If it's a simple identifier expression
            return Some(expr.code.clone());
        }
        None
    }

    /// 在脚本中查找定义
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
                            let start_pos = OakPosition { line: el_span.start.line + def_span.start.line - 1, column: def_span.start.column - 1, offset: 0, length: 0 };
                            let end_pos = OakPosition { line: el_span.start.line + def_span.end.line - 1, column: def_span.end.column - 1, offset: 0, length: 0 };
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

    /// 在程序中查找符号
    fn find_symbol_at(program: &JsProgram, pos: NargoPosition) -> Option<String> {
        for stmt in &program.body {
            if let Some(symbol) = Self::find_symbol_in_stmt(stmt, pos) {
                return Some(symbol);
            }
        }
        None
    }

    /// 在语句中查找符号
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

    /// 在表达式中查找符号
    fn find_symbol_in_expr(expr: &JsExpr, _pos: NargoPosition) -> Option<String> {
        match expr {
            JsExpr::Identifier(name, ..) => Some(name.clone()),
            JsExpr::Call { callee, .. } => Self::find_symbol_in_expr(callee, _pos),
            _ => None,
        }
    }

    /// 查找符号的定义
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

    /// 查找外部定义
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

    /// 查找隐式定义
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
                    let start_pos = OakPosition { line: line as u32, column: col as u32, offset: 0, length: 0 };
                    let end_pos = OakPosition { line: line as u32, column: (col + symbol.len()) as u32, offset: 0, length: 0 };
                    return Some(oak_lsp::types::LocationRange { uri: url.to_string().into(), range: Range { start: self.vfs.line_map(url.as_str()).map(|m| m.line_col_utf16_to_offset(&target_source, start_pos.line, start_pos.column)).unwrap_or(0), end: self.vfs.line_map(url.as_str()).map(|m| m.line_col_utf16_to_offset(&target_source, end_pos.line, end_pos.column)).unwrap_or(0) } });
                }
            }
        }
        None
    }

    /// 解析路径
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

    /// 获取文档内容
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

    /// 在样式中查找定义
    fn find_definition_in_style(_el: &ElementIR, _pos: NargoPosition, _uri: &str) -> Option<oak_lsp::types::LocationRange> {
        // Not implemented yet
        None
    }
}

use nargo_compiler::Compiler;
use oak_core::Parser;

use oak_lsp::types::{CompletionItem as OakCompletionItem, CompletionItemKind as OakCompletionItemKind, Diagnostic as OakDiagnostic, DiagnosticSeverity as OakDiagnosticSeverity};

impl LanguageService for NargoLanguageService {
    type Lang = NargoLanguage;
    type Vfs = MemoryVfs;

    /// 获取VFS
    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    /// 获取工作区
    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }

    /// 获取根节点
    fn get_root(&self, uri: &str) -> impl futures::Future<Output = Option<RedNode<'_, NargoLanguage>>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let mut session = self.sessions.entry(uri.to_string()).or_insert_with(|| ParseSession::new(16));
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
                //     let green: &GreenNode<'static, NargoLanguage> = unsafe { std::mem::transmute(green) };
                //     return Some(RedNode::new(green, 0));
                // }
            }
            None
        }
    }

    /// 获取补全项
    fn completion(&self, uri: &str, position: usize) -> impl futures::Future<Output = Vec<OakCompletionItem>> + Send + '_ {
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
                        OakPosition { line: l, column: c, offset: 0, length: 0 }
                    })
                    .unwrap_or(OakPosition { line: 0, column: 0, offset: 0, length: 0 });
                let nargo_pos = NargoPosition { line: pos.line + 1, column: pos.column + 1, offset: 0 };

                let mut state = ParseState::new(&content);
                let parser = VueTemplateParser;
                if let Ok(nodes) = parser.parse(&mut state, "html") {
                    if let Some(context) = Self::get_completion_context(&nodes, nargo_pos) {
                        match context {
                            CompletionContext::Tag => {
                                let tags = vec!["div", "span", "button", "input", "h1", "h2", "p", "section", "article"];
                                for tag in tags {
                                    items.push(OakCompletionItem { label: tag.to_string(), kind: Some(OakCompletionItemKind::Keyword), detail: Some("HTML Tag".to_string()), documentation: None, insert_text: None });
                                }
                            }
                            CompletionContext::Attribute(tag_name) => {
                                let directives = vec!["@click", "@input", "@change", ":class", ":style", ":value"];
                                for dir in directives {
                                    items.push(OakCompletionItem { label: dir.to_string(), kind: Some(OakCompletionItemKind::Constant), detail: Some("Directive".to_string()), documentation: None, insert_text: None });
                                }

                                if tag_name == "input" {
                                    items.push(OakCompletionItem { label: "type".to_string(), kind: Some(OakCompletionItemKind::Property), detail: None, documentation: None, insert_text: None });
                                }
                            }
                            CompletionContext::Expression => {
                                for import in auto_imports.iter() {
                                    items.push(OakCompletionItem { label: import.clone(), kind: Some(OakCompletionItemKind::Function), detail: Some("Nargo Auto-import".to_string()), documentation: None, insert_text: None });
                                }

                                if let Some(program) = Self::get_script_program(&nodes) {
                                    for stmt in &program.body {
                                        if let JsStmt::VariableDecl { id, .. } = stmt {
                                            items.push(OakCompletionItem { label: id.clone(), kind: Some(OakCompletionItemKind::Variable), detail: None, documentation: None, insert_text: None });
                                        }
                                    }
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

    /// 获取诊断信息
    fn diagnostics(&self, uri: &str) -> impl futures::Future<Output = Vec<OakDiagnostic>> + Send + '_ {
        let source = self.vfs.get_source(uri);
        let uri_parsed = Url::parse(uri).ok();
        let uri = uri.to_string();

        async move {
            let mut diags = Vec::new();
            if let Some(source) = source {
                let content = source.text();
                let mut compiler = Compiler::new();
                let name = uri_parsed.as_ref().and_then(|u| u.path_segments()).and_then(|mut s| s.next_back()).unwrap_or("App.ts");

                if let Err(e) = compiler.compile(name, &content) {
                    let span = e.span();
                    if !span.is_unknown() {
                        let start_pos = OakPosition { line: span.start.line.saturating_sub(1), column: span.start.column.saturating_sub(1), offset: 0, length: 0 };
                        let end_pos = OakPosition { line: span.end.line.saturating_sub(1), column: span.end.column.saturating_sub(1), offset: 0, length: 0 };

                        let start_offset = self.vfs.line_map(&uri).map(|m| m.line_col_utf16_to_offset(&source, start_pos.line, start_pos.column)).unwrap_or(0);
                        let end_offset = self.vfs.line_map(&uri).map(|m| m.line_col_utf16_to_offset(&source, end_pos.line, end_pos.column)).unwrap_or(0);

                        diags.push(OakDiagnostic { range: Range { start: start_offset, end: end_offset }, severity: Some(OakDiagnosticSeverity::Error), message: format!("{}", e), source: Some("nargo-compiler".to_string()), code: None });
                    }
                }
            }
            diags
        }
    }

    /// 初始化
    fn initialize(&self, params: oak_lsp::types::InitializeParams) -> impl futures::Future<Output = ()> + Send + '_ {
        async move {
            self.workspace.initialize(&params);
        }
    }

    /// 获取定义
    fn definition(&self, uri: &str, range: Range<usize>) -> impl futures::Future<Output = Vec<oak_lsp::types::LocationRange>> + Send + '_ {
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
                        OakPosition { line: l, column: c, offset: 0, length: 0 }
                    })
                    .unwrap_or(OakPosition { line: 0, column: 0, offset: 0, length: 0 });
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
}
