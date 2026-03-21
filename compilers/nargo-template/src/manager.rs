#![warn(missing_docs)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::engine::TemplateEngine;
use crate::error::{TemplateError, TemplateResult};
use crate::frontend::UnifiedTemplateEngine;
use nargo_types::NargoValue;

struct TemplateFileInfo {
    path: PathBuf,
    last_modified: SystemTime,
}

/// 模板引擎管理器
///
/// 管理多个模板引擎实例，支持按名称注册、查询和渲染。
pub struct TemplateEngineManager {
    engines: HashMap<String, Box<dyn UnifiedTemplateEngine>>,
    default_engine: Option<String>,
}

impl TemplateEngineManager {
    /// 创建新的模板引擎管理器
    pub fn new() -> Self {
        Self { engines: HashMap::new(), default_engine: None }
    }

    /// 注册模板引擎
    pub fn register_engine(&mut self, name: &str, engine: Box<dyn UnifiedTemplateEngine>) {
        self.engines.insert(name.to_string(), engine);
        if self.default_engine.is_none() {
            self.default_engine = Some(name.to_string());
        }
    }

    /// 设置默认模板引擎
    pub fn set_default_engine(&mut self, name: &str) {
        if self.engines.contains_key(name) {
            self.default_engine = Some(name.to_string());
        }
    }

    /// 获取模板引擎
    pub fn get_engine(&self, name: Option<&str>) -> Option<&dyn UnifiedTemplateEngine> {
        let engine_name = name.or(self.default_engine.as_deref());
        engine_name.and_then(|n| self.engines.get(n).map(|e| e.as_ref()))
    }

    /// 获取可变模板引擎
    pub fn get_engine_mut(&mut self, name: Option<&str>) -> Option<&mut dyn UnifiedTemplateEngine> {
        let engine_name = name.or(self.default_engine.as_deref())?;
        match self.engines.get_mut(engine_name) {
            Some(boxed) => Some(boxed.as_mut()),
            None => None,
        }
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, context: &NargoValue, engine_name: Option<&str>) -> TemplateResult<String> {
        self.get_engine(engine_name).ok_or_else(|| TemplateError::Render("No template engine available".to_string()))?.render(template_name, context)
    }

    /// 异步渲染模板
    pub async fn render_async(&self, template_name: &str, context: &NargoValue, engine_name: Option<&str>) -> TemplateResult<String> {
        self.get_engine(engine_name).ok_or_else(|| TemplateError::Render("No template engine available".to_string()))?.render_async(template_name, context).await
    }
}

impl Default for TemplateEngineManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 模板管理器
///
/// 负责管理模板引擎、注册模板、加载模板文件以及渲染模板
pub struct TemplateManager {
    /// 模板引擎管理器
    engine_manager: TemplateEngineManager,
    /// 模板目录列表
    template_dirs: Vec<PathBuf>,
    /// 模板文件信息映射
    template_files: HashMap<(TemplateEngine, String), TemplateFileInfo>,
    /// 是否启用热加载
    hot_reload: bool,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> Self {
        Self { engine_manager: TemplateEngineManager::new(), template_dirs: Vec::new(), template_files: HashMap::new(), hot_reload: true }
    }

    /// 启用或禁用热加载
    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.hot_reload = enabled;
    }

    /// 注册模板
    pub fn register_template(&mut self, engine: TemplateEngine, name: &str, content: &str) -> TemplateResult<()> {
        let _ = (engine, name, content);
        Ok(())
    }

    /// 注册模板文件
    pub fn register_template_file(&mut self, engine: TemplateEngine, name: &str, path: &Path) -> TemplateResult<()> {
        let last_modified = std::fs::metadata(path)?.modified()?;
        self.template_files.insert((engine, name.to_string()), TemplateFileInfo { path: path.to_path_buf(), last_modified });
        Ok(())
    }

    /// 添加模板目录
    pub fn add_template_dir<P: AsRef<Path>>(&mut self, dir: P) {
        self.template_dirs.push(dir.as_ref().to_path_buf());
    }

    /// 从所有已注册的模板目录加载模板
    pub fn load_templates(&mut self, engine: TemplateEngine) -> TemplateResult<()> {
        let ext = engine.default_extension();
        let dirs: Vec<_> = self.template_dirs.clone();

        for dir in dirs {
            if dir.exists() {
                for entry in walkdir::WalkDir::new(&dir) {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_file() {
                        if let Some(file_ext) = path.extension() {
                            if file_ext == ext {
                                let relative_path = path.strip_prefix(&dir).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                                let template_name = relative_path.with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
                                let last_modified = std::fs::metadata(path)?.modified()?;
                                self.template_files.insert((engine, template_name.clone()), TemplateFileInfo { path: path.to_path_buf(), last_modified });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn check_and_reload_templates(&mut self) -> TemplateResult<()> {
        if !self.hot_reload {
            return Ok(());
        }

        let mut templates_to_reload = Vec::new();
        for ((engine, name), info) in &self.template_files {
            if let Ok(current_modified) = std::fs::metadata(&info.path).and_then(|m| m.modified()) {
                if current_modified > info.last_modified {
                    templates_to_reload.push((*engine, name.clone(), info.path.clone()));
                }
            }
        }

        for (engine, name, path) in templates_to_reload {
            if let Ok(new_modified) = std::fs::metadata(&path).and_then(|m| m.modified()) {
                if let Some(info) = self.template_files.get_mut(&(engine, name)) {
                    info.last_modified = new_modified;
                }
            }
        }

        Ok(())
    }

    /// 使用指定引擎渲染模板
    pub fn render(&mut self, engine: TemplateEngine, template_name: &str, context: &NargoValue) -> TemplateResult<String> {
        self.check_and_reload_templates()?;
        let _ = (engine, template_name, context);
        Err(TemplateError::Render("TemplateManager::render not yet implemented with new API".to_string()))
    }

    /// 异步渲染模板
    pub async fn render_async(&mut self, engine: TemplateEngine, template_name: &str, context: &NargoValue) -> TemplateResult<String> {
        self.check_and_reload_templates()?;
        let _ = (engine, template_name, context);
        Err(TemplateError::Render("TemplateManager::render_async not yet implemented with new API".to_string()))
    }

    /// 统一渲染接口，自动根据文件扩展名选择引擎
    pub fn render_by_extension(&mut self, template_path: &str, context: &NargoValue) -> TemplateResult<String> {
        self.check_and_reload_templates()?;

        let engine = match Path::new(template_path).extension() {
            Some(ext) => match ext.to_str() {
                #[cfg(feature = "dejavu")]
                Some("dejavu") => TemplateEngine::DejaVu,
                #[cfg(feature = "jinja")]
                Some("jinja2") | Some("j2") | Some("jinja") => TemplateEngine::Jinja2,
                #[cfg(feature = "liquid")]
                Some("liquid") => TemplateEngine::Liquid,
                #[cfg(feature = "handlebars")]
                Some("hbs") | Some("handlebars") => TemplateEngine::Handlebars,
                _ => TemplateEngine::Custom,
            },
            None => return Err(TemplateError::Render("Template path must have an extension".to_string())),
        };

        let template_name = Path::new(template_path).with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        self.render(engine, &template_name, context)
    }

    /// 异步统一渲染接口
    pub async fn render_by_extension_async(&mut self, template_path: &str, context: &NargoValue) -> TemplateResult<String> {
        self.check_and_reload_templates()?;

        let engine = match Path::new(template_path).extension() {
            Some(ext) => match ext.to_str() {
                #[cfg(feature = "dejavu")]
                Some("dejavu") => TemplateEngine::DejaVu,
                #[cfg(feature = "jinja")]
                Some("jinja2") | Some("j2") | Some("jinja") => TemplateEngine::Jinja2,
                #[cfg(feature = "liquid")]
                Some("liquid") => TemplateEngine::Liquid,
                #[cfg(feature = "handlebars")]
                Some("hbs") | Some("handlebars") => TemplateEngine::Handlebars,
                _ => TemplateEngine::Custom,
            },
            None => return Err(TemplateError::Render("Template path must have an extension".to_string())),
        };

        let template_name = Path::new(template_path).with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        self.render_async(engine, &template_name, context).await
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
