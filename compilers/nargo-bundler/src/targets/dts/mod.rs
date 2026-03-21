use nargo_ir::IRModule;
use nargo_types::Result;

/// DTS 写入器
#[derive(Default)]
pub struct DtsWriter {
    inner: String,
}

impl DtsWriter {
    /// 创建新的 DTS 写入器
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

    /// 写入接口定义
    pub fn write_interface<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.inner.push_str(&format!("export interface {} {{\n", name));
        f(self);
        self.inner.push_str("}\n");
    }

    /// 写入类型别名
    pub fn write_type_alias(&mut self, name: &str, value: &str) {
        self.inner.push_str(&format!("export type {} = {};\n", name, value));
    }

    /// 写入导入语句
    pub fn write_import(&mut self, names: &[&str], source: &str) {
        self.inner.push_str(&format!("import {{ {} }} from '{}';\n", names.join(", "), source));
    }

    /// 写入常量声明
    pub fn write_declare_const(&mut self, name: &str, type_name: &str) {
        self.inner.push_str(&format!("declare const {}: {};\n", name, type_name));
    }

    /// 写入默认导出
    pub fn write_export_default(&mut self, name: &str) {
        self.inner.push_str(&format!("export default {};\n", name));
    }

    /// 完成写入并返回结果
    pub fn finish(self) -> String {
        self.inner
    }
}

/// DTS 后端
#[derive(Default)]
pub struct DtsBackend;

impl DtsBackend {
    /// 创建新的 DTS 后端
    pub fn new() -> Self {
        Self
    }

    /// 生成 TypeScript 类型定义
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = DtsWriter::new();

        // 1. 从 script_meta 提取元数据
        let mut props = Vec::new();
        let mut emits = Vec::new();
        let mut signals = Vec::new();

        if let Some(meta) = &ir.script_meta {
            if let Some(props_val) = meta.get("props").and_then(|v| v.as_array()) {
                for p in props_val {
                    if let Some(s) = p.as_str() {
                        props.push(s.to_string());
                    }
                }
            }
            if let Some(emits_val) = meta.get("emits").and_then(|v| v.as_array()) {
                for e in emits_val {
                    if let Some(s) = e.as_str() {
                        emits.push(s.to_string());
                    }
                }
            }
            if let Some(signals_val) = meta.get("signals").and_then(|v| v.as_array()) {
                for s in signals_val {
                    if let Some(s_str) = s.as_str() {
                        signals.push(s_str.to_string());
                    }
                }
            }
        }

        // 2. 生成类型定义
        writer.write_import(&["VNode"], "@nargo/core");
        writer.newline();

        let props_name = format!("{}Props", ir.name);
        let emits_name = format!("{}Emits", ir.name);

        // Props 接口
        writer.write_interface(&props_name, |writer| {
            if props.is_empty() {
                writer.write_line("[key: string]: any;");
            }
            else {
                for prop in &props {
                    writer.write_line(&format!("{}?: any;", prop));
                }
            }
        });
        writer.newline();

        // Emits 接口
        if !emits.is_empty() {
            writer.write_interface(&emits_name, |writer| {
                for emit in &emits {
                    writer.write_line(&format!("(e: '{}', ...args: any[]): void;", emit));
                }
            });
            writer.newline();
        }

        // 组件实例（render 中的 'this' 或 'ctx'）
        writer.write_interface("ComponentInstance", |writer| {
            for signal in &signals {
                writer.write_line(&format!("{}: any;", signal));
            }
            // 也将 props 添加到实例中
            for prop in &props {
                writer.write_line(&format!("{}: any;", prop));
            }
            writer.write_line(&format!("$props: {};", props_name));
            if !emits.is_empty() {
                writer.write_line(&format!("$emit: {};", emits_name));
            }
        });
        writer.newline();

        // 组件定义
        writer.write_line("declare const component: {");
        writer.indent();
        writer.write_line(&format!("name: '{}';", ir.name));
        writer.write_line(&format!("setup(props: {}): ComponentInstance;", props_name));
        writer.write_line("render(ctx: ComponentInstance): VNode;");
        writer.dedent();
        writer.write_line("};");
        writer.newline();

        writer.write_export_default("component");

        Ok(writer.finish())
    }
}
