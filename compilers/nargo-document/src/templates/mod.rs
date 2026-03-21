//! 模板系统模块
//! 提供模板管理和渲染功能
//!
//! 本模块兼容旧的 Template 系统，同时支持新的主题系统

use nargo_types::NargoValue;
use oak_toml;
use std::{fs, path::Path};

/// 模板配置结构
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// 主题名称
    pub name: String,
    /// 主题颜色
    pub colors: TemplateColors,
    /// 字体配置
    pub fonts: TemplateFonts,
    /// 功能开关
    pub features: TemplateFeatures,
    /// 语言设置
    pub i18n: TemplateI18n,
}

/// 模板颜色配置
#[derive(Debug, Clone)]
pub struct TemplateColors {
    /// 亮色模式主色
    pub primary_color: String,
    /// 亮色模式次要色
    pub secondary_color: String,
    /// 亮色模式背景色
    pub background_color: String,
    /// 亮色模式文本色
    pub text_color: String,
    /// 亮色模式代码背景色
    pub code_background: String,
    /// 亮色模式边框色
    pub border_color: String,
    /// 暗色模式主色
    pub primary_color_dark: String,
    /// 暗色模式次要色
    pub secondary_color_dark: String,
    /// 暗色模式背景色
    pub background_color_dark: String,
    /// 暗色模式文本色
    pub text_color_dark: String,
    /// 暗色模式代码背景色
    pub code_background_dark: String,
    /// 暗色模式边框色
    pub border_color_dark: String,
}

/// 模板字体配置
#[derive(Debug, Clone)]
pub struct TemplateFonts {
    /// 字体系列
    pub font_family: String,
    /// 代码字体系列
    pub code_font_family: String,
}

/// 模板功能开关
#[derive(Debug, Clone)]
pub struct TemplateFeatures {
    /// 是否启用 Mermaid 图表
    pub enable_mermaid: bool,
    /// 是否启用语法高亮
    pub enable_highlight: bool,
    /// 语法高亮主题
    pub highlight_theme: String,
}

/// 模板语言设置
#[derive(Debug, Clone)]
pub struct TemplateI18n {
    /// 默认语言
    pub default_lang: String,
}

/// 模板结构（旧版本兼容性）
#[derive(Debug, Clone)]
pub struct Template {
    /// 模板名称
    pub name: String,
    /// 模板配置
    pub config: TemplateConfig,
    /// 模板内容
    pub content: String,
}

impl Template {
    /// 从目录加载模板
    ///
    /// # 参数
    /// * `template_dir` - 模板目录路径
    ///
    /// # 返回值
    /// 返回 Result，成功时为 Template，失败时为错误信息
    pub fn load_from_dir(template_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let template_path = Path::new(template_dir).join("template.html");
        let content = fs::read_to_string(template_path)?;

        let config_path = Path::new(template_dir).join("config.toml");
        let config_content = fs::read_to_string(config_path)?;
        let config_value: NargoValue = oak_toml::from_str(&config_content)?;

        let config = Self::parse_config(&config_value)?;

        Ok(Self { name: config.name.clone(), config, content })
    }

    /// 解析模板配置
    ///
    /// # 参数
    /// * `config_value` - 配置值
    ///
    /// # 返回值
    /// 返回 Result，成功时为 TemplateConfig，失败时为错误信息
    fn parse_config(config_value: &NargoValue) -> Result<TemplateConfig, Box<dyn std::error::Error>> {
        let theme = config_value.get("theme").ok_or("Missing theme config")?;

        let name = theme.get("name").and_then(|v| v.as_str()).unwrap_or("default").to_string();

        let colors = theme.get("colors").ok_or("Missing colors config")?;
        let colors = TemplateColors {
            primary_color: colors.get("primary_color").and_then(|v| v.as_str()).unwrap_or("#3498db").to_string(),
            secondary_color: colors.get("secondary_color").and_then(|v| v.as_str()).unwrap_or("#2c3e50").to_string(),
            background_color: colors.get("background_color").and_then(|v| v.as_str()).unwrap_or("#ffffff").to_string(),
            text_color: colors.get("text_color").and_then(|v| v.as_str()).unwrap_or("#333333").to_string(),
            code_background: colors.get("code_background").and_then(|v| v.as_str()).unwrap_or("#f4f4f4").to_string(),
            border_color: colors.get("border_color").and_then(|v| v.as_str()).unwrap_or("#e0e0e0").to_string(),
            primary_color_dark: colors.get("primary_color_dark").and_then(|v| v.as_str()).unwrap_or("#3498db").to_string(),
            secondary_color_dark: colors.get("secondary_color_dark").and_then(|v| v.as_str()).unwrap_or("#ecf0f1").to_string(),
            background_color_dark: colors.get("background_color_dark").and_then(|v| v.as_str()).unwrap_or("#1a1a1a").to_string(),
            text_color_dark: colors.get("text_color_dark").and_then(|v| v.as_str()).unwrap_or("#e0e0e0").to_string(),
            code_background_dark: colors.get("code_background_dark").and_then(|v| v.as_str()).unwrap_or("#2d2d2d").to_string(),
            border_color_dark: colors.get("border_color_dark").and_then(|v| v.as_str()).unwrap_or("#3d3d3d").to_string(),
        };

        let fonts = theme.get("fonts").ok_or("Missing fonts config")?;
        let fonts = TemplateFonts { font_family: fonts.get("font_family").and_then(|v| v.as_str()).unwrap_or("-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif").to_string(), code_font_family: fonts.get("code_font_family").and_then(|v| v.as_str()).unwrap_or("'Courier New', Courier, monospace").to_string() };

        let features = theme.get("features").ok_or("Missing features config")?;
        let features = TemplateFeatures { enable_mermaid: features.get("enable_mermaid").and_then(|v| v.as_bool()).unwrap_or(true), enable_highlight: features.get("enable_highlight").and_then(|v| v.as_bool()).unwrap_or(true), highlight_theme: features.get("highlight_theme").and_then(|v| v.as_str()).unwrap_or("github").to_string() };

        let i18n = theme.get("i18n").ok_or("Missing i18n config")?;
        let i18n = TemplateI18n { default_lang: i18n.get("default_lang").and_then(|v| v.as_str()).unwrap_or("zh-CN").to_string() };

        Ok(TemplateConfig { name, colors, fonts, features, i18n })
    }

    /// 渲染模板
    ///
    /// # 参数
    /// * `title` - 页面标题
    /// * `content` - 页面内容
    /// * `sidebar` - 侧边栏内容
    ///
    /// # 返回值
    /// 返回渲染后的 HTML 字符串
    pub fn render(&self, title: &str, content: &str, sidebar: &str) -> String {
        let mut rendered = self.content.clone();

        rendered = rendered.replace("{{ title }}", title);
        rendered = rendered.replace("{{ content }}", content);
        rendered = rendered.replace("{{ sidebar }}", sidebar);
        rendered = rendered.replace("{{ lang }}", &self.config.i18n.default_lang);

        rendered = rendered.replace("{{ primary_color }}", &self.config.colors.primary_color);
        rendered = rendered.replace("{{ secondary_color }}", &self.config.colors.secondary_color);
        rendered = rendered.replace("{{ background_color }}", &self.config.colors.background_color);
        rendered = rendered.replace("{{ text_color }}", &self.config.colors.text_color);
        rendered = rendered.replace("{{ code_background }}", &self.config.colors.code_background);
        rendered = rendered.replace("{{ border_color }}", &self.config.colors.border_color);
        rendered = rendered.replace("{{ primary_color_dark }}", &self.config.colors.primary_color_dark);
        rendered = rendered.replace("{{ secondary_color_dark }}", &self.config.colors.secondary_color_dark);
        rendered = rendered.replace("{{ background_color_dark }}", &self.config.colors.background_color_dark);
        rendered = rendered.replace("{{ text_color_dark }}", &self.config.colors.text_color_dark);
        rendered = rendered.replace("{{ code_background_dark }}", &self.config.colors.code_background_dark);
        rendered = rendered.replace("{{ border_color_dark }}", &self.config.colors.border_color_dark);

        rendered = rendered.replace("{{ font_family }}", &self.config.fonts.font_family);
        rendered = rendered.replace("{{ code_font_family }}", &self.config.fonts.code_font_family);

        rendered = rendered.replace("{{ enable_mermaid }}", &self.config.features.enable_mermaid.to_string());
        rendered = rendered.replace("{{ enable_highlight }}", &self.config.features.enable_highlight.to_string());
        rendered = rendered.replace("{{ highlight_theme }}", &self.config.features.highlight_theme);

        if !self.config.features.enable_mermaid {
            rendered = rendered.replace("{% if enable_mermaid %}", "");
            rendered = rendered.replace("{% endif %}", "");
        }
        else {
            rendered = rendered.replace("{% if enable_mermaid %}", "");
            rendered = rendered.replace("{% endif %}", "");
        }

        if !self.config.features.enable_highlight {
            rendered = rendered.replace("{% if enable_highlight %}", "");
            rendered = rendered.replace("{% endif %}", "");
        }
        else {
            rendered = rendered.replace("{% if enable_highlight %}", "");
            rendered = rendered.replace("{% endif %}", "");
        }

        rendered
    }
}
