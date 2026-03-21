use nargo_ir::{IRModule, JsExpr, JsStmt, TemplateNodeIR};
use nargo_types::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};

use crate::targets::source_map::{SourceMap, SourceMapBuilder};

/// JavaScript 代码写入器
pub struct JsWriter {
    pub code: String,
    pub mappings: Vec<(usize, usize)>,
}

impl JsWriter {
    /// 创建新的 JavaScript 代码写入器
    pub fn new() -> Self {
        // 预分配更大的容量，减少内存分配
        Self { code: String::with_capacity(8192), mappings: Vec::with_capacity(128) }
    }

    /// 创建具有指定容量的 JavaScript 代码写入器
    pub fn with_capacity(capacity: usize) -> Self {
        Self { code: String::with_capacity(capacity), mappings: Vec::with_capacity(128) }
    }

    /// 写入字符串
    pub fn write(&mut self, s: &str) {
        self.code.push_str(s);
    }

    /// 写入字符
    pub fn write_char(&mut self, c: char) {
        self.code.push(c);
    }

    /// 写入换行符
    pub fn newline(&mut self) {
        self.code.push('\n');
    }

    /// 写入一行文本
    pub fn write_line(&mut self, s: &str) {
        self.code.push_str(s);
        self.code.push('\n');
    }

    /// 缩进
    pub fn indent(&mut self) {
        // 简单实现，实际项目中可能需要更复杂的缩进管理
        self.code.push_str("    ");
    }

    /// 取消缩进
    pub fn dedent(&mut self) {
        // 简单实现，实际项目中可能需要更复杂的缩进管理
    }

    /// 写入导入语句
    pub fn write_import(&mut self, specifiers: &[&str], source: &str) {
        if specifiers.is_empty() {
            return;
        }

        // 预计算容量，减少内存分配
        let mut total_len = 8; // "import {".len()
        for (i, spec) in specifiers.iter().enumerate() {
            if i > 0 {
                total_len += 2; // ", ".len()
            }
            total_len += spec.len();
        }
        total_len += 7 + source.len(); // "}} from '';\n".len()

        let mut import_str = String::with_capacity(total_len);
        import_str.push_str("import {");
        for (i, spec) in specifiers.iter().enumerate() {
            if i > 0 {
                import_str.push_str(", ");
            }
            import_str.push_str(spec);
        }
        import_str.push_str("} from '");
        import_str.push_str(source);
        import_str.push_str("';\n");
        self.code.push_str(&import_str);
    }

    /// 写入块
    pub fn write_block<F>(&mut self, header: &str, mut f: F) -> nargo_types::Result<()>
    where
        F: FnMut(&mut JsWriter) -> nargo_types::Result<()>,
    {
        self.code.push_str(header);
        self.code.push_str(" {");
        self.newline();
        f(self)?;
        self.code.push_str("}");
        self.newline();
        Ok(())
    }

    /// 追加另一个写入器的内容
    pub fn append(&mut self, other: JsWriter) {
        // 直接扩展，避免不必要的内存复制
        self.code.push_str(&other.code);
        self.mappings.extend(other.mappings);
    }

    /// 完成写入并返回代码和映射
    pub fn finish(self) -> (String, Vec<(usize, usize)>) {
        (self.code, self.mappings)
    }

    /// 预分配更多容量
    pub fn reserve(&mut self, additional: usize) {
        self.code.reserve(additional);
    }

    /// 生成组件主体
    pub fn generate_component_body(ir: &IRModule, writer: &mut JsWriter, used_core: &mut HashSet<String>, used_dom: &mut HashSet<String>, bindings: &HashMap<String, JsExpr>) -> nargo_types::Result<()> {
        // 生成组件代码
        writer.write("export default {");
        writer.newline();

        // 预分配容量，减少内存分配
        let name_len = ir.name.len() + 10; // "  name: '',".len()
        writer.reserve(name_len);
        writer.write("  name: '");
        writer.write(&ir.name);
        writer.write("',");
        writer.newline();

        // 生成 setup 函数
        if let Some(script) = &ir.script {
            writer.write("  setup() {");
            writer.newline();
            for stmt in &script.body {
                Self::generate_stmt(stmt, writer, ir, used_core, used_dom, false)?;
            }

            // 预分配容量
            let return_len = ir.name.len() + 15; // "    return ;".len()
            writer.reserve(return_len);
            writer.write("    return ");
            writer.write(&ir.name);
            writer.write(";\n");
            writer.write("  },");
            writer.newline();
        }

        // 生成 render 函数
        if let Some(template) = &ir.template {
            writer.write("  render() {");
            writer.newline();
            writer.write("    return h('div', null, [");
            writer.newline();
            for node in &template.nodes {
                Self::generate_node(node, writer, ir, used_core, used_dom, false)?;
            }
            writer.write("    ]);");
            writer.newline();
            writer.write("  },");
            writer.newline();
        }

        writer.write("}");
        writer.newline();
        Ok(())
    }

    /// 生成语句
    pub fn generate_stmt(stmt: &JsStmt, writer: &mut JsWriter, ir: &IRModule, used_core: &mut HashSet<String>, used_dom: &mut HashSet<String>, is_resumable: bool) -> nargo_types::Result<()> {
        match stmt {
            JsStmt::Expr(expr, _, _) => {
                Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(";\n");
            }
            JsStmt::VariableDecl { kind, id, init, .. } => {
                // 预分配容量
                let mut decl_len = 8 + kind.len() + id.len(); // "    ".len() + kind.len() + " ".len() + id.len()
                if init.is_some() {
                    decl_len += 3; // " = ".len()
                }
                else {
                    decl_len += 9; // "undefined".len()
                }
                decl_len += 2; // ";\n".len()
                writer.reserve(decl_len);

                writer.write("    ");
                writer.write(kind);
                writer.write(" ");
                writer.write(id);
                writer.write(" = ");
                if let Some(expr) = init {
                    Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
                }
                else {
                    writer.write("undefined");
                }
                writer.write(";\n");
            }
            JsStmt::Return(expr, _, _) => {
                writer.write("    return ");
                if let Some(expr) = expr {
                    Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write(";\n");
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                writer.write("    if (");
                Self::generate_expr(test, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(") {");
                writer.newline();
                Self::generate_stmt(consequent, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                if let Some(alt) = alternate {
                    writer.write(" else {");
                    writer.newline();
                    Self::generate_stmt(alt, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("    }");
                }
                writer.newline();
            }
            JsStmt::While { test, body, .. } => {
                writer.write("    while (");
                Self::generate_expr(test, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(") {");
                writer.newline();
                Self::generate_stmt(body, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                writer.newline();
            }
            JsStmt::For { init, test, update, body, .. } => {
                writer.write("    for (");
                if let Some(init) = init {
                    Self::generate_stmt(init, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("; ");
                }
                else {
                    writer.write("; ");
                }
                if let Some(test) = test {
                    Self::generate_expr(test, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("; ");
                if let Some(update) = update {
                    Self::generate_expr(update, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write(") {");
                writer.newline();
                Self::generate_stmt(body, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                writer.newline();
            }
            JsStmt::ForIn { left, right, body, .. } => {
                writer.write("    for (");
                Self::generate_stmt(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" in ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(") {");
                writer.newline();
                Self::generate_stmt(body, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                writer.newline();
            }
            JsStmt::ForOf { left, right, body, .. } => {
                writer.write("    for (");
                Self::generate_stmt(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" of ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(") {");
                writer.newline();
                Self::generate_stmt(body, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                writer.newline();
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                writer.write("    try {");
                writer.newline();
                Self::generate_stmt(block, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("    }");
                if let Some((catch_id, catch_body)) = handler {
                    writer.write(" catch (");
                    writer.write(catch_id);
                    writer.write(") {");
                    writer.newline();
                    Self::generate_stmt(catch_body, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("    }");
                }
                if let Some(finally_body) = finalizer {
                    writer.write(" finally {");
                    writer.newline();
                    Self::generate_stmt(finally_body, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("    }");
                }
                writer.newline();
            }
            JsStmt::Block(stmts, _, _) => {
                writer.write("    {");
                writer.newline();
                for s in stmts {
                    Self::generate_stmt(s, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("    }");
                writer.newline();
            }
            JsStmt::FunctionDecl { id, params, body, is_async, .. } => {
                // 预分配容量
                let mut func_len = 14 + id.len(); // "    function ".len() + id.len()
                if *is_async {
                    func_len += 6; // "async ".len()
                }
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        func_len += 2; // ", ".len()
                    }
                    func_len += param.len();
                }
                func_len += 4; // "() {".len()
                writer.reserve(func_len);

                writer.write("    ");
                if *is_async {
                    writer.write("async ");
                }
                writer.write("function ");
                writer.write(id);
                writer.write("(");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write(param);
                }
                writer.write(") {");
                writer.newline();
                for s in body {
                    Self::generate_stmt(s, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("    }");
                writer.newline();
            }
            JsStmt::Export { declaration, .. } => {
                writer.write("    export ");
                Self::generate_stmt(declaration, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsStmt::ExportAll { source, .. } => {
                writer.write("    export * from '");
                writer.write(source);
                writer.write("';\n");
            }
            JsStmt::ExportNamed { source, specifiers, .. } => {
                if let Some(source) = source {
                    writer.write("    export {");
                    for (i, spec) in specifiers.iter().enumerate() {
                        if i > 0 {
                            writer.write(", ");
                        }
                        writer.write(spec);
                    }
                    writer.write("} from '");
                    writer.write(source);
                    writer.write("';\n");
                }
                else {
                    writer.write("    export {");
                    for (i, spec) in specifiers.iter().enumerate() {
                        if i > 0 {
                            writer.write(", ");
                        }
                        writer.write(spec);
                    }
                    writer.write("};\n");
                }
            }
            JsStmt::Break(_, _) => {
                writer.write("    break;\n");
            }
            JsStmt::Continue(_, _) => {
                writer.write("    continue;\n");
            }
            JsStmt::Throw(expr, _, _) => {
                writer.write("    throw ");
                Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(";\n");
            }
            JsStmt::Switch { discriminant, cases, .. } => {
                writer.write("    switch (");
                Self::generate_expr(discriminant, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(") {");
                writer.newline();
                for (test, stmts) in cases {
                    if let Some(test_expr) = test {
                        writer.write("    case ");
                        Self::generate_expr(test_expr, writer, ir, used_core, used_dom, is_resumable)?;
                        writer.write(":");
                        writer.newline();
                    }
                    else {
                        writer.write("    default:");
                        writer.newline();
                    }
                    for stmt in stmts {
                        Self::generate_stmt(stmt, writer, ir, used_core, used_dom, is_resumable)?;
                    }
                }
                writer.write("    }");
                writer.newline();
            }
            JsStmt::Other(code, _, _) => {
                // 处理TypeScript特有的语法结构
                let processed_code = JsWriter::process_typescript_code(code);
                writer.write("    ");
                writer.write(&processed_code);
                writer.write("\n");
            }
            _ => {
                // 其他语句类型
            }
        }
        Ok(())
    }

    /// 生成表达式
    pub fn generate_expr(expr: &JsExpr, writer: &mut JsWriter, ir: &IRModule, used_core: &mut HashSet<String>, used_dom: &mut HashSet<String>, is_resumable: bool) -> nargo_types::Result<()> {
        match expr {
            JsExpr::Identifier(name, _, _) => {
                writer.write(name);
            }
            JsExpr::Literal(value, _, _) => {
                // 直接写入字符串，避免额外的内存分配
                match value {
                    nargo_types::NargoValue::String(s) => {
                        writer.write("'");
                        writer.write(s);
                        writer.write("'");
                    }
                    nargo_types::NargoValue::Number(n) => {
                        writer.write(&n.to_string());
                    }
                    nargo_types::NargoValue::Bool(b) => {
                        writer.write(if *b { "true" } else { "false" });
                    }
                    nargo_types::NargoValue::Null => {
                        writer.write("null");
                    }
                    _ => {
                        writer.write(&value.to_string());
                    }
                }
            }
            JsExpr::Call { callee, args, .. } => {
                Self::generate_expr(callee, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    Self::generate_expr(arg, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write(")");
            }
            JsExpr::Binary { left, right, op, .. } => {
                writer.write("(");
                Self::generate_expr(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" ");
                writer.write(op);
                writer.write(" ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(")");
            }
            JsExpr::Unary { argument, op, .. } => {
                writer.write(op);
                writer.write("(");
                Self::generate_expr(argument, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(")");
            }
            JsExpr::Array(exprs, _, _) => {
                writer.write("[");
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    Self::generate_expr(e, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("]");
            }
            JsExpr::Object(props, _, _) => {
                writer.write("{");
                for (i, (key, value)) in props.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write(key);
                    writer.write(": ");
                    Self::generate_expr(value, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("}");
            }
            JsExpr::ArrowFunction { params, body, .. } => {
                // 预分配容量
                let mut arrow_len = 2; // "(".len()
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        arrow_len += 2; // ", ".len()
                    }
                    arrow_len += param.len();
                }
                arrow_len += 6; // ") => ".len()
                writer.reserve(arrow_len);

                writer.write("(");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write(param);
                }
                writer.write(") => ");
                Self::generate_expr(body, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                Self::generate_expr(test, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" ? ");
                Self::generate_expr(consequent, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" : ");
                Self::generate_expr(alternate, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::TemplateLiteral { expressions, quasis, .. } => {
                writer.write("`");
                for (i, quasi) in quasis.iter().enumerate() {
                    writer.write(quasi);
                    if i < expressions.len() {
                        writer.write("${");
                        Self::generate_expr(&expressions[i], writer, ir, used_core, used_dom, is_resumable)?;
                        writer.write("}");
                    }
                }
                writer.write("`");
            }
            JsExpr::Member { object, property, computed, .. } => {
                Self::generate_expr(object, writer, ir, used_core, used_dom, is_resumable)?;
                if *computed {
                    writer.write("[");
                    Self::generate_expr(property, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("]");
                }
                else {
                    writer.write(".");
                    Self::generate_expr(property, writer, ir, used_core, used_dom, is_resumable)?;
                }
            }
            JsExpr::OptionalMember { object, property, computed, .. } => {
                Self::generate_expr(object, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("?.");
                if *computed {
                    writer.write("[");
                    Self::generate_expr(property, writer, ir, used_core, used_dom, is_resumable)?;
                    writer.write("]");
                }
                else {
                    Self::generate_expr(property, writer, ir, used_core, used_dom, is_resumable)?;
                }
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                Self::generate_expr(callee, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write("?.(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    Self::generate_expr(arg, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write(")");
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                Self::generate_expr(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" ?? ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::LogicalAssignment { op, left, right, .. } => {
                Self::generate_expr(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" ");
                writer.write(op);
                writer.write("= ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::Spread(expr, _, _) => {
                writer.write("...");
                Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::TypeOf(expr, _, _) => {
                writer.write("typeof ");
                Self::generate_expr(expr, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::InstanceOf { left, right, .. } => {
                Self::generate_expr(left, writer, ir, used_core, used_dom, is_resumable)?;
                writer.write(" instanceof ");
                Self::generate_expr(right, writer, ir, used_core, used_dom, is_resumable)?;
            }
            JsExpr::TseElement { tag, attributes, children, .. } => {
                writer.write("<");
                writer.write(tag);
                for attr in attributes {
                    writer.write(" ");
                    writer.write(&attr.name);
                    if let Some(value) = &attr.value {
                        writer.write("=");
                        Self::generate_expr(value, writer, ir, used_core, used_dom, is_resumable)?;
                    }
                }
                writer.write(">");
                for child in children {
                    Self::generate_expr(child, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("</");
                writer.write(tag);
                writer.write(">");
            }
            JsExpr::Other(code, _, _) => {
                writer.write(code);
            }
        }
        Ok(())
    }

    /// 处理TypeScript特有的语法结构
    fn process_typescript_code(code: &str) -> String {
        // 移除类型注解
        let mut result = code.to_string();

        // 移除变量类型注解: let x: number = 5;
        result = regex::Regex::new(r"(:\s*[a-zA-Z0-9_\[\]\|\&\<\>\.]+)").unwrap().replace_all(&result, "").to_string();

        // 移除函数参数类型注解: function foo(x: number) {}
        result = regex::Regex::new(r"(\w+)\s*:\s*[a-zA-Z0-9_\[\]\|\&\<\>\.]+").unwrap().replace_all(&result, "$1").to_string();

        // 移除函数返回类型注解: function foo(): number {}
        result = regex::Regex::new(r"\)\s*:\s*[a-zA-Z0-9_\[\]\|\&\<\>\.]+").unwrap().replace_all(&result, ")").to_string();

        // 移除接口定义
        result = regex::Regex::new(r"interface\s+\w+\s*\{[^\}]*\}").unwrap().replace_all(&result, "").to_string();

        // 移除类型别名
        result = regex::Regex::new(r"type\s+\w+\s*=\s*[^{;]+;").unwrap().replace_all(&result, "").to_string();

        // 移除枚举定义
        result = regex::Regex::new(r"enum\s+\w+\s*\{[^\}]*\}").unwrap().replace_all(&result, "").to_string();

        // 移除泛型类型参数: function foo<T>(x: T) {}
        result = regex::Regex::new(r"<[a-zA-Z0-9_\,\s]+>").unwrap().replace_all(&result, "").to_string();

        result
    }

    /// 生成模板节点
    pub fn generate_node(node: &TemplateNodeIR, writer: &mut JsWriter, ir: &IRModule, used_core: &mut HashSet<String>, used_dom: &mut HashSet<String>, is_resumable: bool) -> nargo_types::Result<()> {
        match node {
            TemplateNodeIR::Element(el) => {
                writer.write("      h('*");
                writer.write(&el.tag);
                writer.write("', {");
                for (i, attr) in el.attributes.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write(&attr.name);
                    writer.write(": ");
                    if let Some(ast) = &attr.value_ast {
                        Self::generate_expr(ast, writer, ir, used_core, used_dom, is_resumable)?;
                    }
                    else {
                        writer.write("'*");
                        writer.write(attr.value.as_deref().unwrap_or(""));
                        writer.write("'");
                    }
                }
                writer.write("}, [");
                writer.newline();
                for child in &el.children {
                    Self::generate_node(child, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write("      ]),");
                writer.newline();
            }
            TemplateNodeIR::Text(text, _, _) => {
                writer.write("      '*");
                writer.write(text);
                writer.write("',");
                writer.newline();
            }
            TemplateNodeIR::Interpolation(expr) => {
                writer.write("      ");
                if let Some(ast) = &expr.ast {
                    Self::generate_expr(ast, writer, ir, used_core, used_dom, is_resumable)?;
                }
                writer.write(",");
                writer.newline();
            }
            _ => {
                // 其他节点类型
            }
        }
        Ok(())
    }
}
