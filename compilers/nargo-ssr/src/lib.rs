#![warn(missing_docs)]

mod prefetch;
mod router;

use nargo_bundler::targets::js::{JsBackend, JsWriter};
use nargo_ir::{IRModule, TemplateNodeIR};
use nargo_types::{CompileMode, Result};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

pub use prefetch::PrefetchManager;
pub use router::{RouteHandler, RouteMatch, Router};

/// SSR 渲染缓存键
type CacheKey = String;

/// SSR 渲染缓存值
type CacheValue = String;

/// 缓存项，包含渲染结果和元数据
#[derive(Debug, Clone)]
pub struct CacheItem {
    /// 渲染结果
    pub value: CacheValue,
    /// 缓存时间戳
    pub timestamp: u128,
    /// 模板节点数量，用于判断模板复杂度
    pub node_count: usize,
}

pub struct SsrBackend {
    pub runtime_path: String,
    pub resumable: bool,
    pub mode: CompileMode,
    pub cache: HashMap<CacheKey, CacheItem>,
    pub cache_enabled: bool,
    /// 缓存大小限制（字节）
    pub cache_size_limit: u64,
    /// 当前缓存大小（字节）
    pub current_cache_size: u64,
    /// 缓存项最大数量
    pub cache_item_limit: usize,
}

impl SsrBackend {
    pub fn new(mode: CompileMode) -> Self {
        let runtime_path = match mode {
            CompileMode::Vue2 => "nargo".to_string(),
            CompileMode::Vue => "vue".to_string(),
        };
        Self {
            runtime_path,
            resumable: false,
            mode,
            cache: HashMap::new(),
            cache_enabled: true,
            cache_size_limit: 1024 * 1024 * 10, // 10MB
            current_cache_size: 0,
            cache_item_limit: 1000,
        }
    }

    pub fn with_resumable(mut self, resumable: bool) -> Self {
        self.resumable = resumable;
        self
    }

    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    pub fn with_cache_size_limit(mut self, limit: u64) -> Self {
        self.cache_size_limit = limit;
        self
    }

    pub fn with_cache_item_limit(mut self, limit: usize) -> Self {
        self.cache_item_limit = limit;
        self
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.current_cache_size = 0;
    }

    /// 清理过期或超出限制的缓存项
    fn cleanup_cache(&mut self) {
        if !self.cache_enabled {
            return;
        }

        // 检查缓存项数量限制
        if self.cache.len() > self.cache_item_limit {
            // 按时间戳排序，删除最旧的项
            let mut items: Vec<_> = self.cache.iter().collect();
            items.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));

            let items_to_remove = items.len() - self.cache_item_limit;
            // 收集需要删除的键
            let keys_to_remove: Vec<String> = items.iter().take(items_to_remove).map(|(k, _)| k.to_string()).collect();

            // 然后删除这些键
            for key in keys_to_remove {
                if let Some(item) = self.cache.remove(&key) {
                    self.current_cache_size -= (key.len() + item.value.len()) as u64;
                }
            }
        }

        // 检查缓存大小限制
        while self.current_cache_size > self.cache_size_limit && !self.cache.is_empty() {
            // 按时间戳排序，删除最旧的项
            let mut items: Vec<_> = self.cache.iter().collect();
            items.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));

            if let Some((key, _)) = items.first() {
                let key_to_remove = key.to_string();
                if let Some(item) = self.cache.remove(&key_to_remove) {
                    self.current_cache_size -= (key_to_remove.len() + item.value.len()) as u64;
                }
            }
        }
    }
}

impl Default for SsrBackend {
    fn default() -> Self {
        Self::new(CompileMode::Vue2)
    }
}

impl SsrBackend {
    pub fn generate(&mut self, ir: &IRModule) -> Result<String> {
        // 生成缓存键
        let cache_key = self.generate_cache_key(ir);

        // 检查缓存
        if self.cache_enabled {
            // 尝试获取缓存项
            if let Some(cached) = self.cache.get(&cache_key) {
                // 克隆缓存值
                let value_clone = cached.value.clone();

                // 注意：为了避免不可变借用和可变借用冲突，我们暂时不更新时间戳
                // 后续可以考虑使用更复杂的缓存策略

                // 返回缓存值
                return Ok(value_clone);
            }
        }

        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();

        // 1. Generate SSR Function Body
        let mut body_writer = JsWriter::new();
        self.generate_ssr_body(ir, &mut body_writer, &mut used_core)?;

        // 2. Generate Imports
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            let import_source = if self.mode == CompileMode::Vue { "vue".to_string() } else { format!("{}/core", self.runtime_path) };
            writer.write_line(&format!("import {{ {} }} from '{}';", imports.join(", "), import_source));
            writer.newline();
        }

        // 3. Append Body
        writer.append(body_writer);

        let result = writer.finish().0;

        // 缓存结果
        if self.cache_enabled {
            let node_count = ir.template.as_ref().map(|t| t.nodes.len()).unwrap_or(0);
            let cache_item = CacheItem { value: result.clone(), timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), node_count };

            // 计算缓存项大小
            let item_size = (cache_key.len() + result.len()) as u64;

            // 检查是否需要清理缓存
            if self.current_cache_size + item_size > self.cache_size_limit {
                self.cleanup_cache();
            }

            // 插入新缓存项
            let cache_key_clone = cache_key.clone();
            if let Some(old_item) = self.cache.insert(cache_key, cache_item) {
                // 减去旧项的大小
                self.current_cache_size -= (old_item.value.len() + cache_key_clone.len()) as u64;
            }
            // 加上新项的大小
            self.current_cache_size += item_size;
        }

        Ok(result)
    }

    fn generate_cache_key(&self, ir: &IRModule) -> CacheKey {
        // 基于模块名称、模板内容和脚本生成更准确的缓存键
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // 模块名称
        ir.name.hash(&mut hasher);
        // 编译模式
        format!("{:?}", self.mode).hash(&mut hasher);
        // 模板节点数量
        if let Some(template) = &ir.template {
            template.nodes.len().hash(&mut hasher);
            // 简单哈希模板结构
            for node in &template.nodes {
                match node {
                    TemplateNodeIR::Element(el) => el.tag.hash(&mut hasher),
                    TemplateNodeIR::Text(text, _, _) => text.hash(&mut hasher),
                    TemplateNodeIR::Interpolation(expr) => expr.code.hash(&mut hasher),
                    TemplateNodeIR::Comment(comment, _, _) => comment.hash(&mut hasher),
                    TemplateNodeIR::Hoisted(id) => id.hash(&mut hasher),
                    TemplateNodeIR::If(if_node) => if_node.condition.code.hash(&mut hasher),
                    TemplateNodeIR::For(for_node) => for_node.iterator.collection.code.hash(&mut hasher),
                }
            }
        }
        // 服务器脚本
        if let Some(script) = &ir.script_server {
            script.body.len().hash(&mut hasher);
        }
        // 普通脚本
        if let Some(script) = &ir.script {
            script.body.len().hash(&mut hasher);
        }

        format!("{}-{:x}", ir.name, hasher.finish())
    }

    fn generate_ssr_body(&self, ir: &IRModule, writer: &mut JsWriter, used_core: &mut HashSet<String>) -> Result<()> {
        let _ = writer.write_block("export function render(ctx)", |writer| {
            // Create a JsBackend instance for generating statements
            let js_backend = JsBackend::new(false, false, None, self.mode);
            let mut used_dom = HashSet::new();

            // Include server and normal scripts
            if let Some(script) = &ir.script_server {
                for stmt in &script.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false);
                }
            }
            if let Some(script) = &ir.script {
                for stmt in &script.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false);
                }
            }

            // 优化：使用更高效的字符串构建方式
            writer.write("let html = [];");
            writer.newline();

            if self.resumable {
                // Serialize state for resumability
                writer.write_line("html.push(`<script type=\"nargo/state\">${JSON.stringify(ctx)}</script>`);");
            }

            if let Some(template) = &ir.template {
                let mut node_index = 0;
                for node in &template.nodes {
                    self.generate_node_ssr_optimized(node, writer, &mut node_index);
                }
            }

            // 优化：使用 join 方法拼接字符串，减少内存分配
            writer.write_line("return html.join('');");
            Ok(())
        });
        Ok(())
    }

    fn generate_node_ssr_optimized(&self, node: &TemplateNodeIR, writer: &mut JsWriter, node_index: &mut usize) {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                // 预分配更大的字符串容量，减少内存分配
                let mut start_tag = String::with_capacity(128);
                start_tag.push_str("<");
                start_tag.push_str(&el.tag);

                // Add data-nargo-id for non-static elements or elements with dynamic content
                if !el.is_static {
                    start_tag.push_str(&format!(" data-nargo-id=\"{}\"", current_index));
                }

                for attr in &el.attributes {
                    if !attr.is_directive {
                        if attr.is_dynamic {
                            writer.write_line(&format!("html.push('{}');", start_tag));
                            writer.write_line(&format!("html.push('{}=\"' + ({}) + '\"');", attr.name, attr.value.as_deref().unwrap_or("")));
                            start_tag.clear();
                        }
                        else {
                            match &attr.value {
                                Some(v) => start_tag.push_str(&format!(" {}=\"{}\"", attr.name, v)),
                                None => start_tag.push_str(&format!(" {}", attr.name)),
                            }
                        }
                    }
                    else if self.resumable {
                        // Handle event directives for resumability
                        if attr.is_directive && (attr.name == "on" || attr.name.starts_with("@")) {
                            let event_name = if attr.name == "on" {
                                attr.argument.as_deref().unwrap_or("")
                            }
                            else {
                                // Handle @click syntax
                                attr.name.trim_start_matches('@')
                            };

                            if !event_name.is_empty() {
                                // For the prototype, we assume the handler is in a chunk named after the component
                                // and the handler name is the value of the attribute.
                                let handler = attr.value.as_deref().unwrap_or("");
                                if !handler.is_empty() {
                                    start_tag.push_str(&format!(" on:{}=\"/assets/{}.js#{}\"", event_name, "test", handler));
                                }
                            }
                        }
                    }
                }

                if !start_tag.is_empty() {
                    start_tag.push_str(">\n");
                    writer.write_line(&format!("html.push('{}');", start_tag));
                }

                for child in &el.children {
                    self.generate_node_ssr_optimized(child, writer, node_index);
                }

                writer.write_line(&format!("html.push('</{}>');", el.tag));
            }
            TemplateNodeIR::Text(text, _, _) => {
                *node_index += 1;
                // 更高效的字符串转义
                let mut escaped_text = String::with_capacity(text.len() + 10);
                for c in text.chars() {
                    match c {
                        '\'' => escaped_text.push_str("\\'"),
                        '\\' => escaped_text.push_str("\\\\"),
                        _ => escaped_text.push(c),
                    }
                }
                writer.write_line(&format!("html.push('{}');", escaped_text));
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;
                // Wrap interpolation in a span with ID for hydration
                writer.write_line(&format!("html.push('<span data-nargo-id=\"{}\">');", current_index));
                writer.write_line(&format!("html.push(String({}));", expr.code));
                writer.write_line(&format!("html.push('</span>');"));
            }
            TemplateNodeIR::Comment(comment, _, _) => {
                *node_index += 1;
                // 更高效的字符串转义
                let mut escaped_comment = String::with_capacity(comment.len() + 10);
                for c in comment.chars() {
                    match c {
                        '\'' => escaped_comment.push_str("\\'"),
                        '\\' => escaped_comment.push_str("\\\\"),
                        _ => escaped_comment.push(c),
                    }
                }
                writer.write_line(&format!("html.push('<!-- {} -->');", escaped_comment));
            }
            TemplateNodeIR::Hoisted(id) => {
                *node_index += 1;
                writer.write_line(&format!("html.push({});", id));
            }
            TemplateNodeIR::If(if_node) => {
                let current_index = *node_index;
                *node_index += 1;

                // For SSR, we generate a JS if-else structure
                writer.write_line(&format!("if ({}) {{", if_node.condition.code));
                writer.indent();
                for child in &if_node.consequent {
                    self.generate_node_ssr_optimized(child, writer, node_index);
                }
                writer.dedent();

                for (condition, nodes) in &if_node.else_ifs {
                    writer.write_line(&format!("}} else if ({}) {{", condition.code));
                    writer.indent();
                    for child in nodes {
                        self.generate_node_ssr_optimized(child, writer, node_index);
                    }
                    writer.dedent();
                }

                if let Some(alt) = &if_node.alternate {
                    writer.write_line("} else {");
                    writer.indent();
                    for child in alt {
                        self.generate_node_ssr_optimized(child, writer, node_index);
                    }
                    writer.dedent();
                }
                writer.write_line("}");
            }
            TemplateNodeIR::For(for_node) => {
                let current_index = *node_index;
                *node_index += 1;

                // For SSR, we generate a JS loop
                writer.write_line(&format!("({}).forEach(({}, {}) => {{", for_node.iterator.collection.code, for_node.iterator.item, for_node.iterator.index.as_deref().unwrap_or("_i")));
                writer.indent();
                for child in &for_node.body {
                    self.generate_node_ssr_optimized(child, writer, node_index);
                }
                writer.dedent();
                writer.write_line("});");
            }
        }
    }

    fn generate_node_ssr(&self, node: &TemplateNodeIR, writer: &mut JsWriter, node_index: &mut usize) {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                let mut start_tag = format!("html += '<{}", el.tag);

                // Add data-nargo-id for non-static elements or elements with dynamic content
                if !el.is_static {
                    start_tag.push_str(&format!(" data-nargo-id=\"{}\"", current_index));
                }

                for attr in &el.attributes {
                    if !attr.is_directive {
                        if attr.is_dynamic {
                            start_tag.push_str(&format!(" {}=\"' + ({}) + '\"", attr.name, attr.value.as_deref().unwrap_or("")));
                        }
                        else {
                            match &attr.value {
                                Some(v) => start_tag.push_str(&format!(" {}=\"{}\"", attr.name, v)),
                                None => start_tag.push_str(&format!(" {}", attr.name)),
                            }
                        }
                    }
                    else if self.resumable {
                        // Handle event directives for resumability
                        if attr.is_directive && (attr.name == "on" || attr.name.starts_with("@")) {
                            let event_name = if attr.name == "on" {
                                attr.argument.as_deref().unwrap_or("")
                            }
                            else {
                                // Handle @click syntax
                                attr.name.trim_start_matches('@')
                            };

                            if !event_name.is_empty() {
                                // For the prototype, we assume the handler is in a chunk named after the component
                                // and the handler name is the value of the attribute.
                                let handler = attr.value.as_deref().unwrap_or("");
                                if !handler.is_empty() {
                                    start_tag.push_str(&format!(" on:{}=\"/assets/{}.js#{}\"", event_name, "test", handler));
                                }
                            }
                        }
                    }
                }
                start_tag.push_str(">';");
                writer.write_line(&start_tag);

                for child in &el.children {
                    self.generate_node_ssr(child, writer, node_index);
                }

                writer.write_line(&format!("html += '</{}>';", el.tag));
            }
            TemplateNodeIR::Text(text, _, _) => {
                *node_index += 1;
                writer.write_line(&format!("html += '{}';", text.replace("'", "\\'")));
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;
                // Wrap interpolation in a span with ID for hydration
                writer.write_line(&format!("html += '<span data-nargo-id=\"{}\">' + ({}) + '</span>';", current_index, expr.code));
            }
            TemplateNodeIR::Comment(comment, _, _) => {
                *node_index += 1;
                writer.write_line(&format!("html += '<!-- {} -->';", comment.replace("'", "\\'")));
            }
            TemplateNodeIR::Hoisted(id) => {
                *node_index += 1;
                writer.write_line(&format!("html += {};", id));
            }
            TemplateNodeIR::If(if_node) => {
                let current_index = *node_index;
                *node_index += 1;

                // For SSR, we generate a JS if-else structure
                writer.write_line(&format!("if ({}) {{", if_node.condition.code));
                writer.indent();
                for child in &if_node.consequent {
                    self.generate_node_ssr(child, writer, node_index);
                }
                writer.dedent();

                for (condition, nodes) in &if_node.else_ifs {
                    writer.write_line(&format!("}} else if ({}) {{", condition.code));
                    writer.indent();
                    for child in nodes {
                        self.generate_node_ssr(child, writer, node_index);
                    }
                    writer.dedent();
                }

                if let Some(alt) = &if_node.alternate {
                    writer.write_line("} else {");
                    writer.indent();
                    for child in alt {
                        self.generate_node_ssr(child, writer, node_index);
                    }
                    writer.dedent();
                }
                writer.write_line("}");
            }
            TemplateNodeIR::For(for_node) => {
                let current_index = *node_index;
                *node_index += 1;

                // For SSR, we generate a JS loop
                writer.write_line(&format!("({}).forEach(({}, {}) => {{", for_node.iterator.collection.code, for_node.iterator.item, for_node.iterator.index.as_deref().unwrap_or("_i")));
                writer.indent();
                for child in &for_node.body {
                    self.generate_node_ssr(child, writer, node_index);
                }
                writer.dedent();
                writer.write_line("});");
            }
        }
    }
}
