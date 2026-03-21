use nargo_ir::{IRModule, JsProgram};
use nargo_types::{CompileMode, Result};
use std::collections::{HashMap, HashSet};

use crate::targets::source_map::{SourceMap, SourceMapBuilder};

pub mod body;
pub mod expr;
pub mod node;
pub mod stmt;
pub mod writer;

pub use body::collect_const_bindings;
pub use writer::JsWriter;

/// JavaScript 后端
pub struct JsBackend {
    pub minify: bool,
    pub is_prod: bool,
    pub target: Option<String>,
    pub runtime_path: String,
    pub mode: CompileMode,
}

impl JsBackend {
    /// 创建新的 JavaScript 后端
    pub fn new(minify: bool, is_prod: bool, target: Option<String>, mode: CompileMode) -> Self {
        let runtime_path = match mode {
            CompileMode::Vue2 => "nargo".to_string(),
            CompileMode::Vue => "vue".to_string(),
        };
        Self { minify, is_prod, target, runtime_path, mode }
    }

    /// 生成代码
    pub fn generate(&self, ir: &IRModule) -> Result<(String, SourceMap)> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();
        let mut used_dom = HashSet::new();

        // 0. 预收集常量绑定以进行静态评估
        let bindings = collect_const_bindings(ir);

        // 1. 先生成组件主体以跟踪使用的功能
        let mut body_writer = JsWriter::new();
        JsWriter::generate_component_body(ir, &mut body_writer, &mut used_core, &mut used_dom, &bindings)?;

        // 2. 根据使用的功能生成导入
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            let specifiers: Vec<&str> = imports.iter().map(|s| s.as_str()).collect();
            let import_source = if self.mode == CompileMode::Vue { "vue".to_string() } else { format!("{}/core", self.runtime_path) };
            writer.write_import(&specifiers, &import_source);
        }
        if !used_dom.is_empty() {
            let mut imports: Vec<_> = used_dom.into_iter().collect();
            imports.sort();
            let specifiers: Vec<&str> = imports.iter().map(|s| s.as_str()).collect();
            let import_source = if self.mode == CompileMode::Vue { "vue".to_string() } else { format!("{}/dom", self.runtime_path) };
            writer.write_import(&specifiers, &import_source);
        }
        writer.newline();

        // 3. 追加主体
        writer.append(body_writer);

        let (mut code, _mappings) = writer.finish();

        // 4. 优化生成的代码
        if self.minify {
            code = self.minify_code(&code);
        }

        let mut builder = SourceMapBuilder::new();

        Ok((code, builder.finish()))
    }

    /// 压缩代码
    fn minify_code(&self, code: &str) -> String {
        // 简单的代码压缩
        code.replace("\n", "").replace("\t", "").replace("  ", "").replace(" {", "{").replace("{ ", "{").replace(" }", "}").replace("} ", "}").replace(" : ", ":").replace(" , ", ",").replace("; ", ";").replace("( ", "(").replace(" )", ")").replace("[ ", "[").replace(" ]", "]").replace("+ ", "+").replace(" +", "+").replace("- ", "-").replace(" -", "-").replace("* ", "*").replace(" *", "*").replace("/ ", "/").replace(" /", "/").replace("% ", "%").replace(" %", "%").replace("== ", "==").replace(" ==", "==").replace("=== ", "===").replace(" ===", "===").replace("!= ", "!").replace(" !=", "!").replace("!== ", "!==").replace(" !==", "!==").replace("> ", ">").replace(" >", ">").replace("< ", "<").replace(" <", "<").replace(">= ", ">").replace(" >=", ">").replace("<= ", "<=").replace(" <=", "<=").replace("&& ", "&&").replace(" &&", "&&").replace("|| ", "||").replace(" ||", "||").replace("! ", "!").replace(" !", "!")
    }

    /// 生成程序
    pub fn generate_program(&self, program: &JsProgram) -> Result<(String, SourceMap)> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();
        let mut used_dom = HashSet::new();

        for stmt in &program.body {
            JsWriter::generate_stmt(stmt, &mut writer, &IRModule::default(), &mut used_core, &mut used_dom, false)?;
        }

        let (code, _mappings) = writer.finish();
        let mut builder = SourceMapBuilder::new();

        Ok((code, builder.finish()))
    }

    /// 生成可恢复的代码
    pub fn generate_resumable(&self, ir: &IRModule) -> Result<String> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();
        let mut used_dom = HashSet::new();

        if let Some(script) = &ir.script {
            for stmt in &script.body {
                match stmt {
                    nargo_ir::JsStmt::FunctionDecl { id, params, body, .. } => {
                        // 导出函数以便加载器可以懒加载
                        writer.write("export ");
                        writer.write_block(&format!("function {}(event, ctx) ", id), |writer| {
                            // 为方便起见解构 ctx，但在原型中只使用它
                            for stmt in body {
                                JsWriter::generate_stmt(stmt, writer, ir, &mut used_core, &mut used_dom, true)?;
                            }
                            Ok(())
                        });
                        writer.newline();
                    }
                    _ => {
                        JsWriter::generate_stmt(stmt, &mut writer, ir, &mut used_core, &mut used_dom, false)?;
                    }
                }
            }
        }

        Ok(writer.finish().0)
    }
}
