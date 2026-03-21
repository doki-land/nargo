use nargo_ir::{AttributeIR, ElementIR, IRModule, TemplateNodeIR};
use nargo_types::Result;

/// HTML 写入器
#[derive(Default)]
pub struct HtmlWriter {
    inner: String,
}

impl HtmlWriter {
    /// 创建新的 HTML 写入器
    pub fn new() -> Self {
        Self { inner: String::new() }
    }

    /// 写入文本
    pub fn write(&mut self, text: &str) {
        self.inner.push_str(text);
    }

    /// 写入一行文本
    pub fn write_line(&mut self, text: &str) {
        self.inner.push_str(text);
        self.inner.push_str("\n");
    }

    /// 写入换行符
    pub fn newline(&mut self) {
        self.inner.push_str("\n");
    }

    /// 增加缩进
    pub fn indent(&mut self) {
        // 手动缩进处理
    }

    /// 减少缩进
    pub fn dedent(&mut self) {
        // 手动缩进处理
    }

    /// 写入带有属性的标签开始: <tag attr1="val1">
    pub fn write_tag_start(&mut self, tag: &str, attributes: &[(&str, Option<&str>)]) {
        self.inner.push_str("<");
        self.inner.push_str(tag);
        for (name, value) in attributes {
            self.inner.push_str(" ");
            self.inner.push_str(name);
            if let Some(val) = value {
                self.inner.push_str("=");
                self.inner.push_str("\"");
                self.inner.push_str(val);
                self.inner.push_str("\"");
            }
        }
        self.inner.push_str(">");
    }

    /// 写入自闭合标签: <tag attr1="val1" />
    pub fn write_self_closing_tag(&mut self, tag: &str, attributes: &[(&str, Option<&str>)]) {
        self.inner.push_str("<");
        self.inner.push_str(tag);
        for (name, value) in attributes {
            self.inner.push_str(" ");
            self.inner.push_str(name);
            if let Some(val) = value {
                self.inner.push_str("=");
                self.inner.push_str("\"");
                self.inner.push_str(val);
                self.inner.push_str("\"");
            }
        }
        self.inner.push_str(" />");
    }

    /// 写入结束标签: </tag>
    pub fn write_tag_end(&mut self, tag: &str) {
        self.inner.push_str("</");
        self.inner.push_str(tag);
        self.inner.push_str(">");
    }

    /// 写入注释: <!-- content -->
    pub fn write_comment(&mut self, content: &str) {
        self.inner.push_str("<!-- ");
        self.inner.push_str(content);
        self.inner.push_str(" -->");
    }

    /// 写入模板节点
    pub fn write_node(&mut self, node: &TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => self.write_element(el),
            TemplateNodeIR::If(if_node) => {
                // 简化的 HTML 表示，用于 SSR/Static
                self.write("<!-- if: ");
                self.write(&if_node.condition.code);
                self.write(" -->");
                for child in &if_node.consequent {
                    self.write_node(child);
                }
                if let Some(alt) = &if_node.alternate {
                    self.write("<!-- else -->");
                    for child in alt {
                        self.write_node(child);
                    }
                }
                self.write("<!-- /if -->");
            }
            TemplateNodeIR::For(for_node) => {
                self.write("<!-- for: ");
                self.write(&for_node.iterator.item);
                if let Some(idx) = &for_node.iterator.index {
                    self.write(&format!(", {}", idx));
                }
                self.write(" in ");
                self.write(&for_node.iterator.collection.code);
                self.write(" -->");
                for child in &for_node.body {
                    self.write_node(child);
                }
                self.write("<!-- /for -->");
            }
            TemplateNodeIR::Text(text, _, _) => self.write(text),
            TemplateNodeIR::Interpolation(expr) => {
                self.write("{{ ");
                self.write(&expr.code);
                self.write(" }}");
            }
            TemplateNodeIR::Comment(comment, _, _) => {
                self.write("<!-- ");
                self.write(comment);
                self.write(" -->");
            }
            TemplateNodeIR::Hoisted(_) => {}
        }
    }

    /// 写入元素
    pub fn write_element(&mut self, el: &ElementIR) {
        if el.children.is_empty() && nargo_types::is_void_element(&el.tag) {
            self.inner.push_str("<");
            self.inner.push_str(&el.tag);
            for attr in &el.attributes {
                self.write_attribute(attr);
            }
            self.inner.push_str(" />");
        }
        else {
            self.inner.push_str("<");
            self.inner.push_str(&el.tag);
            for attr in &el.attributes {
                self.write_attribute(attr);
            }
            self.inner.push_str(">");

            if el.children.len() == 1 {
                let child = &el.children[0];
                match child {
                    TemplateNodeIR::Text(text, _, _) => self.write(text),
                    TemplateNodeIR::Interpolation(expr) => {
                        self.write("{{ ");
                        self.write(&expr.code);
                        self.write(" }}");
                    }
                    _ => {
                        self.newline();
                        self.write_node(child);
                        self.newline();
                    }
                }
            }
            else if !el.children.is_empty() {
                self.newline();
                for child in &el.children {
                    self.write_node(child);
                    self.newline();
                }
            }
            self.write_tag_end(&el.tag);
        }
    }

    /// 写入属性
    fn write_attribute(&mut self, attr: &AttributeIR) {
        self.write(" ");
        self.write(&attr.name);
        if let Some(arg) = &attr.argument {
            self.write(":");
            self.write(arg);
        }
        if let Some(value) = &attr.value {
            self.write("=");
            self.write("\"");
            self.write(value);
            self.write("\"");
        }
    }

    /// 完成写入并返回结果
    pub fn finish(self) -> String {
        self.inner
    }
}

/// HTML 后端
#[derive(Default)]
pub struct HtmlBackend;

impl HtmlBackend {
    /// 创建新的 HTML 后端
    pub fn new() -> Self {
        Self
    }

    /// 生成 HTML 代码
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = HtmlWriter::new();

        if let Some(template) = &ir.template {
            for node in &template.nodes {
                writer.write_node(node);
                writer.newline();
            }
        }

        Ok(writer.finish())
    }
}
