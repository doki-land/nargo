use nargo_ir::{IRModule, TemplateNodeIR};
use nargo_types::{CompileMode, Result};
use std::collections::{HashMap, HashSet};

use crate::targets::js::{JsBackend, JsWriter};

/// SSR 渲染缓存键
type CacheKey = String;

/// SSR 渲染缓存值
type CacheValue = String;

/// SSR 后端
pub struct SsrBackend {
    /// 运行时路径
    pub runtime_path: String,
    /// 是否支持可恢复性
    pub resumable: bool,
    /// 编译模式
    pub mode: CompileMode,
    /// 渲染缓存
    pub cache: HashMap<CacheKey, CacheValue>,
    /// 是否启用缓存
    pub cache_enabled: bool,
}

impl SsrBackend {
    /// 创建新的 SSR 后端
    pub fn new(mode: CompileMode) -> Self {
        let runtime_path = match mode {
            CompileMode::Vue2 => "nargo".to_string(),
            CompileMode::Vue => "vue".to_string(),
        };
        Self { runtime_path, resumable: false, mode, cache: HashMap::new(), cache_enabled: true }
    }

    /// 设置是否支持可恢复性
    pub fn with_resumable(mut self, resumable: bool) -> Self {
        self.resumable = resumable;
        self
    }

    /// 设置是否启用缓存
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for SsrBackend {
    /// 默认创建 Nargo 模式的 SSR 后端
    fn default() -> Self {
        Self::new(CompileMode::Vue2)
    }
}

impl SsrBackend {
    /// 生成 SSR 代码
    pub fn generate(&mut self, ir: &IRModule) -> Result<String> {
        // 生成缓存键
        let cache_key = self.generate_cache_key(ir);

        // 检查缓存
        if self.cache_enabled {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();

        // 1. 生成 SSR 函数主体
        let mut body_writer = JsWriter::new();
        self.generate_ssr_body(ir, &mut body_writer, &mut used_core)?;

        // 2. 生成导入
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            let import_source = if self.mode == CompileMode::Vue { "vue".to_string() } else { format!("{}/core", self.runtime_path) };
            writer.write_line(&format!("import {{ {} }} from '{}';", imports.join(", "), import_source));
            writer.newline();
        }

        // 3. 追加主体
        writer.append(body_writer);

        let result = writer.finish().0;

        // 缓存结果
        if self.cache_enabled {
            self.cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// 生成缓存键
    fn generate_cache_key(&self, ir: &IRModule) -> CacheKey {
        // 基于模块名称和模板内容生成缓存键
        let mut key = format!("{}-{:?}", ir.name, self.mode);
        if let Some(template) = &ir.template {
            key.push_str(&format!("-{}", template.nodes.len()));
        }
        key
    }

    /// 生成 SSR 函数主体
    fn generate_ssr_body(&self, ir: &IRModule, writer: &mut JsWriter, used_core: &mut HashSet<String>) -> Result<()> {
        writer.write_block("export function render(ctx)", |writer| {
            // 创建 JsBackend 实例用于生成语句
            let js_backend = JsBackend::new(false, false, None, self.mode);
            let mut used_dom = HashSet::new();

            // 包含服务器端和普通脚本
            if let Some(script) = &ir.script_server {
                for stmt in &script.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false)?;
                }
            }
            if let Some(script) = &ir.script {
                for stmt in &script.body {
                    JsWriter::generate_stmt(stmt, writer, ir, used_core, &mut used_dom, false)?;
                }
            }

            // 优化：使用更高效的字符串构建方式
            writer.write("let html = [];");
            writer.newline();

            if self.resumable {
                // 序列化状态以支持可恢复性
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

    /// 优化的 SSR 节点生成
    fn generate_node_ssr_optimized(&self, node: &TemplateNodeIR, writer: &mut JsWriter, node_index: &mut usize) {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                // 预分配字符串容量，减少内存分配
                let mut start_tag = String::with_capacity(64);
                start_tag.push_str("<");
                start_tag.push_str(&el.tag);

                // 为非静态元素或具有动态内容的元素添加 data-nargo-id
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
                        // 处理事件指令以支持可恢复性
                        if attr.is_directive && (attr.name == "on" || attr.name.starts_with("@")) {
                            let event_name = if attr.name == "on" {
                                attr.argument.as_deref().unwrap_or("")
                            }
                            else {
                                // 处理 @click 语法
                                attr.name.trim_start_matches('@')
                            };

                            if !event_name.is_empty() {
                                // 对于原型，我们假设处理程序在以组件命名的 chunk 中
                                // 并且处理程序名称是属性的值。
                                let handler = attr.value.as_deref().unwrap_or("");
                                if !handler.is_empty() {
                                    start_tag.push_str(&format!(" on:{}=\"/assets/{}.js#{}  ", event_name, "test", handler));
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
                // 预计算字符串长度，减少内存分配
                let escaped_text = text.replace("'", "\\'");
                writer.write_line(&format!("html.push('{}');", escaped_text));
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;
                // 为水合包装插值在带有 ID 的 span 中
                writer.write_line(&format!("html.push('<span data-nargo-id=\"{}\">');", current_index));
                writer.write_line(&format!("html.push({});", expr.code));
                writer.write_line(&format!("html.push('</span>');"));
            }
            TemplateNodeIR::Comment(comment, _, _) => {
                *node_index += 1;
                let escaped_comment = comment.replace("'", "\\'");
                writer.write_line(&format!("html.push('<!-- {} -->');", escaped_comment));
            }
            TemplateNodeIR::Hoisted(id) => {
                *node_index += 1;
                writer.write_line(&format!("html.push({});", id));
            }
            TemplateNodeIR::If(if_node) => {
                let current_index = *node_index;
                *node_index += 1;

                // 对于 SSR，我们生成 JS if-else 结构
                writer.write_line(&format!("if ({}) {{  ", if_node.condition.code));
                writer.indent();
                for child in &if_node.consequent {
                    self.generate_node_ssr_optimized(child, writer, node_index);
                }
                writer.dedent();

                for (condition, nodes) in &if_node.else_ifs {
                    writer.write_line(&format!("}} else if ({}) {{  ", condition.code));
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

                // 对于 SSR，我们生成 JS 循环
                writer.write_line(&format!("({}).forEach(({}, {}) => {{  ", for_node.iterator.collection.code, for_node.iterator.item, for_node.iterator.index.as_deref().unwrap_or("_i")));
                writer.indent();
                for child in &for_node.body {
                    self.generate_node_ssr_optimized(child, writer, node_index);
                }
                writer.dedent();
                writer.write_line("});");
            }
        }
    }
}
