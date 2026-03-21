#![warn(missing_docs)]

/// DejaVu 模板前端
#[cfg(feature = "dejavu")]
pub mod dejavu;
/// EJS 模板前端
#[cfg(feature = "ejs")]
pub mod ejs;
/// Handlebars 模板前端
#[cfg(feature = "handlebars")]
pub mod handlebars;
/// Jinja2 模板前端
#[cfg(feature = "jinja")]
pub mod jinja2;
/// Liquid 模板前端
#[cfg(feature = "liquid")]
pub mod liquid;

use crate::error::TemplateResult;
use crate::ir::TemplateIR;
use crate::vm::VM;
use nargo_types::NargoValue;
use std::collections::HashMap;
use std::path::Path;

/// 语言前端 trait
///
/// 每个模板语言前端实现此 trait，只负责将模板源码编译为 TemplateIR。
pub trait Frontend: Send + Sync {
    /// 将模板源码编译为 TemplateIR
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR>;

    /// 前端名称
    fn name(&self) -> &str;

    /// 默认文件扩展名
    fn default_extension(&self) -> &str;
}

/// 泛型模板适配器
///
/// 组合 Frontend（编译）和 VM（执行），统一实现 UnifiedTemplateEngine trait。
pub struct TemplateAdapter<F: Frontend> {
    /// 语言前端
    frontend: F,
    /// 虚拟机
    vm: VM,
    /// 已注册的模板 IR
    templates: HashMap<String, TemplateIR>,
}

impl<F: Frontend> TemplateAdapter<F> {
    /// 创建新的模板适配器
    pub fn new(frontend: F) -> Self {
        Self {
            frontend,
            vm: VM::new(),
            templates: HashMap::new(),
        }
    }
}

/// 将 walkdir::Error 转换为 std::io::Error
fn walkdir_error_to_io(e: walkdir::Error) -> std::io::Error {
    if let Some(io_err) = e.io_error() {
        std::io::Error::new(io_err.kind(), e.to_string())
    } else {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

/// 模板引擎抽象接口
#[async_trait::async_trait]
pub trait UnifiedTemplateEngine: Send + Sync {
    /// 渲染模板
    fn render(&self, template_name: &str, context: &NargoValue) -> TemplateResult<String>;

    /// 异步渲染模板
    async fn render_async(&self, template_name: &str, context: &NargoValue) -> TemplateResult<String> {
        Ok(self.render(template_name, context)?)
    }

    /// 注册模板
    fn register_template(&mut self, name: &str, content: &str) -> TemplateResult<()>;

    /// 注册模板文件
    fn register_template_file(&mut self, name: &str, path: &Path) -> TemplateResult<()>;

    /// 从目录注册模板
    fn register_templates_from_dir(&mut self, dir: &Path, extension: Option<&str>) -> TemplateResult<()>;

    /// 获取模板引擎名称
    fn name(&self) -> &str;
}

#[async_trait::async_trait]
impl<F: Frontend + 'static> UnifiedTemplateEngine for TemplateAdapter<F> {
    fn render(&self, template_name: &str, context: &NargoValue) -> TemplateResult<String> {
        let ir = self.templates.get(template_name)
            .ok_or_else(|| crate::error::TemplateError::TemplateNotFound(template_name.to_string()))?;
        self.vm.execute(ir, context)
    }

    async fn render_async(&self, template_name: &str, context: &NargoValue) -> TemplateResult<String> {
        Ok(self.render(template_name, context)?)
    }

    fn register_template(&mut self, name: &str, content: &str) -> TemplateResult<()> {
        let ir = self.frontend.compile(content)?;
        self.vm.register_ir(name, ir.clone());
        self.templates.insert(name.to_string(), ir);
        Ok(())
    }

    fn register_template_file(&mut self, name: &str, path: &Path) -> TemplateResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(crate::error::TemplateError::Io)?;
        self.register_template(name, &content)
    }

    fn register_templates_from_dir(&mut self, dir: &Path, extension: Option<&str>) -> TemplateResult<()> {
        let ext = match extension {
            Some(e) => e.to_string(),
            None => self.frontend.default_extension().to_string(),
        };
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry
                .map_err(walkdir_error_to_io)
                .map_err(crate::error::TemplateError::Io)?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_ext) = path.extension() {
                    if file_ext == ext.as_str() {
                        let relative_path = path.strip_prefix(dir).map_err(|e| {
                            crate::error::TemplateError::Render(
                                format!("Failed to strip prefix: {}", e)
                            )
                        })?;
                        let template_name = relative_path
                            .with_extension("")
                            .to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, "/");
                        self.register_template_file(&template_name, path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        self.frontend.name()
    }
}
