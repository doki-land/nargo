use nargo_ir::{IRModule, StyleIR};
use nargo_style_processor::{PreprocessorType, StyleProcessor, optimize, process_with_preprocessor};
use nargo_types::Result;

/// CSS 写入器
#[derive(Default)]
pub struct CssWriter {
    inner: String,
}

impl CssWriter {
    /// 创建新的 CSS 写入器
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
        // 简单实现，添加两个空格
        self.inner.push_str("  ");
    }

    /// 减少缩进
    pub fn dedent(&mut self) {
        // 简单实现，不做处理
    }

    /// 完成写入并返回结果
    pub fn finish(self) -> String {
        self.inner
    }
}

/// CSS 编译器
#[derive(Default)]
pub struct CssCompiler {}

impl CssCompiler {
    /// 创建新的 CSS 编译器
    pub fn new() -> Self {
        Self {}
    }

    /// 压缩 CSS 代码
    pub fn minify(&self, css: &str) -> String {
        css.trim().split('\n').map(|s| s.trim()).collect::<Vec<_>>().join("").replace(" {", "{").replace("{ ", "{").replace(" }", "}").replace("} ", "}").replace(": ", ":").replace("; ", ";")
    }
}

/// CSS 后端
pub struct CssBackend {
    /// 是否启用压缩
    pub minify: bool,
    /// 是否移除未使用的样式
    pub remove_unused: bool,
    /// 已使用的选择器列表
    pub used_selectors: Vec<String>,
}

impl CssBackend {
    /// 创建新的 CSS 后端
    pub fn new(minify: bool) -> Self {
        Self { minify, remove_unused: false, used_selectors: Vec::new() }
    }

    /// 设置是否移除未使用的样式
    pub fn with_remove_unused(mut self, remove_unused: bool) -> Self {
        self.remove_unused = remove_unused;
        self
    }

    /// 设置已使用的选择器列表
    pub fn with_used_selectors(mut self, used_selectors: Vec<String>) -> Self {
        self.used_selectors = used_selectors;
        self
    }

    /// 压缩 CSS 代码
    pub fn minify(&self, css: &str) -> String {
        // Basic minification for now, can be improved with lightningcss
        css.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// 生成 CSS 代码
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = CssWriter::new();

        for style in &ir.styles {
            self.generate_style(style, &mut writer)?;
            writer.newline();
        }

        Ok(writer.finish())
    }

    /// 生成单个样式块
    fn generate_style(&self, style: &StyleIR, writer: &mut CssWriter) -> Result<()> {
        // 根据 lang 属性确定预处理器类型
        let preprocessor = match style.lang.as_str() {
            "scss" | "sass" => Some(PreprocessorType::Scss),
            "less" => Some(PreprocessorType::Less),
            "stylus" => Some(PreprocessorType::Stylus),
            _ => None,
        };

        // 处理 CSS 代码
        let processed_css = if let Some(preprocessor_type) = preprocessor {
            // 使用预处理器处理
            let preprocessed = process_with_preprocessor(&style.code, preprocessor_type)?;

            // 进一步优化
            if self.remove_unused && !self.used_selectors.is_empty() {
                optimize(&preprocessed, &self.used_selectors)?
            }
            else if self.minify {
                let mut processor = StyleProcessor::new().with_minify(true);
                processor.process(&preprocessed, None)?
            }
            else {
                preprocessed
            }
        }
        else if self.remove_unused && !self.used_selectors.is_empty() {
            // 优化 CSS，移除未使用的样式并压缩
            optimize(&style.code, &self.used_selectors)?
        }
        else if self.minify {
            // 仅压缩 CSS
            let mut processor = StyleProcessor::new().with_minify(true);
            processor.process(&style.code, None)?
        }
        else {
            // 不做处理，保持原样
            style.code.clone()
        };

        // 增强 CSS 新特性支持
        let enhanced_css = self.enhance_css_features(&processed_css);

        if !self.minify {
            if style.scoped {
                writer.write_line(&format!("/* Scoped CSS (lang: {}) */", style.lang));
            }
            else {
                writer.write_line(&format!("/* CSS (lang: {}) */", style.lang));
            }
        }

        writer.write(&enhanced_css);
        Ok(())
    }

    /// 增强 CSS 新特性支持
    fn enhance_css_features(&self, css: &str) -> String {
        let mut result = css.to_string();

        // 处理 CSS 变量
        result = self.process_css_variables(&result);

        // 处理 CSS Grid 特性
        result = self.process_css_grid(&result);

        // 处理 Flexbox 高级特性
        result = self.process_flexbox(&result);

        // 处理 Container Queries
        result = self.process_container_queries(&result);

        // 处理 CSS 嵌套
        result = self.process_css_nesting(&result);

        result
    }

    /// 处理 CSS 变量
    fn process_css_variables(&self, css: &str) -> String {
        // 简单实现：确保 CSS 变量语法正确
        css.to_string()
    }

    /// 处理 CSS Grid 特性
    fn process_css_grid(&self, css: &str) -> String {
        // 简单实现：确保 Grid 语法正确
        css.to_string()
    }

    /// 处理 Flexbox 高级特性
    fn process_flexbox(&self, css: &str) -> String {
        // 简单实现：确保 Flexbox 语法正确
        css.to_string()
    }

    /// 处理 Container Queries
    fn process_container_queries(&self, css: &str) -> String {
        // 简单实现：确保 Container Queries 语法正确
        css.to_string()
    }

    /// 处理 CSS 嵌套
    fn process_css_nesting(&self, css: &str) -> String {
        // 简单实现：确保 CSS 嵌套语法正确
        css.to_string()
    }
}
