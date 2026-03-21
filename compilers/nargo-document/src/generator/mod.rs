pub mod html;
pub mod markdown;
pub mod pdf;
pub mod static_;

use crate::{
    config::Config,
    generator::{html::HtmlGenerator, markdown::MarkdownRenderer, pdf::PdfGenerator, static_::StaticProcessor},
    plugin::{PluginContext, PluginRegistry},
};
use std::{fs, path::Path};

/// 文档生成器核心组件（可克隆）
#[derive(Clone)]
struct CoreGenerator {
    /// 配置信息
    config: Config,
    /// Markdown 渲染器
    markdown_renderer: MarkdownRenderer,
    /// HTML 生成器
    html_generator: HtmlGenerator,
    /// PDF 生成器
    pdf_generator: PdfGenerator,
    /// 静态资源处理器
    static_processor: StaticProcessor,
}

/// 文档生成器，负责将 Markdown 文件转换为多种格式的文档
pub struct Generator {
    /// 核心组件
    core: CoreGenerator,
    /// 插件注册表
    plugin_registry: PluginRegistry,
}

impl Generator {
    /// 创建新的文档生成器
    ///
    /// # 参数
    /// * `config` - 文档配置信息
    ///
    /// # 返回值
    /// 返回一个新的文档生成器实例
    pub fn new(config: Config) -> Self {
        Self { core: CoreGenerator { config: config.clone(), markdown_renderer: MarkdownRenderer::new(), html_generator: HtmlGenerator::with_config(config), pdf_generator: PdfGenerator::new(), static_processor: StaticProcessor::new() }, plugin_registry: PluginRegistry::new() }
    }

    /// 获取插件注册表的可变引用
    ///
    /// # 返回值
    /// 返回插件注册表的可变引用
    pub fn plugin_registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.plugin_registry
    }

    /// 克隆核心组件
    fn clone_core(&self) -> CoreGenerator {
        self.core.clone()
    }

    /// 生成文档
    ///
    /// # 参数
    /// * `input_dir` - 输入目录，包含 Markdown 文件
    /// * `output_dir` - 输出目录，用于存放生成的文档
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    pub fn generate(&mut self, input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating documentation from {} to {}", input_dir, output_dir);

        // 1. 创建输出目录
        fs::create_dir_all(output_dir)?;

        // 2. 初始化所有插件
        self.plugin_registry.setup_all(None);
        println!("Initialized {} plugins", self.plugin_registry.plugin_count());

        // 3. 扫描 Markdown 文件
        let markdown_files = self.find_markdown_files(input_dir)?;
        println!("Found {} Markdown files", markdown_files.len());

        // 4. 顺序处理每个 Markdown 文件
        let input_dir = input_dir.to_string();
        let output_dir = output_dir.to_string();

        for file in markdown_files.iter() {
            if let Err(e) = self.process_markdown_file(file, &input_dir, &output_dir) {
                eprintln!("Error processing file {}: {}", file, e);
            }
        }

        // 5. 处理静态资源
        self.core.static_processor.copy_static_files(&input_dir, &output_dir)?;

        // 6. 生成索引文件
        self.generate_index(&output_dir)?;

        Ok(())
    }

    /// 查找所有 Markdown 文件
    ///
    /// # 参数
    /// * `dir` - 要搜索的目录
    ///
    /// # 返回值
    /// 返回包含所有 Markdown 文件路径的 Vec
    fn find_markdown_files(&self, dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        self.scan_dir(Path::new(dir), &mut files)?;
        Ok(files)
    }

    /// 递归扫描目录，查找 Markdown 文件
    ///
    /// # 参数
    /// * `path` - 当前扫描的路径
    /// * `files` - 用于存储找到的 Markdown 文件路径
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    fn scan_dir(&self, path: &Path, files: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    self.scan_dir(&entry_path, files)?;
                }
                else if let Some(ext) = entry_path.extension() {
                    if ext == "md" || ext == "markdown" {
                        files.push(entry_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理单个 Markdown 文件
    ///
    /// # 参数
    /// * `file` - Markdown 文件路径
    /// * `input_dir` - 输入目录
    /// * `output_dir` - 输出目录
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    fn process_markdown_file(&self, file: &str, input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 读取 Markdown 文件
        let content = fs::read_to_string(file)?;

        // 创建插件上下文
        let context = PluginContext::from_content(content.clone(), file.to_string());

        // 调用 before_render 钩子
        let before_context = self.plugin_registry.before_render_all(context);

        // 渲染 Markdown 为 HTML
        let markdown_html = self.core.markdown_renderer.render(&before_context.content)?;

        // 生成完整的 HTML 页面
        let title = self.extract_title(&before_context.content).unwrap_or_else(|| "Documentation".to_string());
        let full_html = self.core.html_generator.generate(&markdown_html, &title);

        // 创建 after_render 上下文
        let after_context = PluginContext::new(full_html.clone(), before_context.frontmatter.clone(), file.to_string());

        // 调用 after_render 钩子
        let final_context = self.plugin_registry.after_render_all(after_context);

        // 计算输出路径
        let relative_path = Path::new(file).strip_prefix(input_dir)?;

        // 生成 HTML 文件
        let html_path = Path::new(output_dir).join(relative_path).with_extension("html");
        if let Some(parent) = html_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&html_path, &final_context.content)?;
        println!("Generated: {}", html_path.display());

        // 生成 PDF 文件
        let pdf_path = Path::new(output_dir).join(relative_path).with_extension("pdf");
        if let Some(parent) = pdf_path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.core.pdf_generator.generate(&final_context.content, &pdf_path)?;
        println!("Generated: {}", pdf_path.display());

        Ok(())
    }

    /// 从 Markdown 内容中提取标题
    ///
    /// # 参数
    /// * `content` - Markdown 内容
    ///
    /// # 返回值
    /// 返回提取的标题，如果没有标题则返回 None
    fn extract_title(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            if line.starts_with("# ") {
                return Some(line[2..].trim().to_string());
            }
        }
        None
    }

    /// 生成索引文件
    ///
    /// # 参数
    /// * `output_dir` - 输出目录
    ///
    /// # 返回值
    /// 返回 Result，成功时为 ()，失败时为错误信息
    fn generate_index(&self, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let index_content = r#"
# HXO Documentation

Welcome to HXO Documentation!

## Getting Started

- [Installation](installation.html)
- [Quick Start](quick-start.html)
- [Configuration](configuration.html)

## API Reference

- [Core Concepts](core-concepts.html)
- [CLI Commands](cli-commands.html)
- [API Documentation](api-documentation.html)
"#;

        let markdown_html = self.core.markdown_renderer.render(index_content)?;
        let full_html = self.core.html_generator.generate(&markdown_html, "HXO Documentation");

        // 生成 HTML 索引文件
        let index_path = Path::new(output_dir).join("index.html");
        fs::write(&index_path, &full_html)?;
        println!("Generated: {}", index_path.display());

        // 生成 PDF 索引文件
        let pdf_path = Path::new(output_dir).join("index.pdf");
        self.core.pdf_generator.generate(&full_html, &pdf_path)?;
        println!("Generated: {}", pdf_path.display());

        Ok(())
    }
}
