//! 主题系统测试
//! 测试所有内置主题的功能和正确性

use nargo_document::{
    config::Config,
    generator::html::HtmlGenerator,
    theme::{DarkTheme, DefaultTheme, TechTheme, Theme, ThemeFactory},
};

#[test]
fn test_theme_factory_create() {
    let config = Config::default();

    // 测试创建默认主题
    let default_theme = ThemeFactory::create("default", config.clone());
    assert!(default_theme.is_ok());
    assert_eq!(default_theme.unwrap().name(), "default");

    // 测试创建暗色主题
    let dark_theme = ThemeFactory::create("dark", config.clone());
    assert!(dark_theme.is_ok());
    assert_eq!(dark_theme.unwrap().name(), "dark");

    // 测试创建科技风主题
    let tech_theme = ThemeFactory::create("tech", config.clone());
    assert!(tech_theme.is_ok());
    assert_eq!(tech_theme.unwrap().name(), "tech");

    // 测试不区分大小写
    let theme = ThemeFactory::create("DEFAULT", config.clone());
    assert!(theme.is_ok());
    assert_eq!(theme.unwrap().name(), "default");

    // 测试未知主题
    let unknown_theme = ThemeFactory::create("unknown", config);
    assert!(unknown_theme.is_err());
}

#[test]
fn test_available_themes() {
    let themes = ThemeFactory::available_themes();
    assert_eq!(themes.len(), 3);
    assert!(themes.contains(&"default"));
    assert!(themes.contains(&"dark"));
    assert!(themes.contains(&"tech"));
}

#[test]
fn test_default_theme_creation() {
    let config = Config::default();
    let theme = DefaultTheme::new(config.clone());
    assert!(theme.is_ok());

    let theme = theme.unwrap();
    assert_eq!(theme.name(), "default");
    assert_eq!(theme.site_title(), "");
    assert_eq!(theme.config().title, None);
}

#[test]
fn test_dark_theme_creation() {
    let config = Config::default();
    let theme = DarkTheme::new(config.clone());
    assert!(theme.is_ok());

    let theme = theme.unwrap();
    assert_eq!(theme.name(), "dark");
    assert_eq!(theme.site_title(), "");
    assert_eq!(theme.config().title, None);
}

#[test]
fn test_tech_theme_creation() {
    let config = Config::default();
    let theme = TechTheme::new(config.clone());
    assert!(theme.is_ok());

    let theme = theme.unwrap();
    assert_eq!(theme.name(), "tech");
    assert_eq!(theme.site_title(), "");
    assert_eq!(theme.config().title, None);
}

#[test]
fn test_html_generator_with_theme() {
    let config = Config::default();

    // 测试使用不同主题创建 HtmlGenerator
    let mut generator = HtmlGenerator::with_config_and_theme(config.clone(), "default");
    assert_eq!(generator.current_theme_name(), "default");

    // 测试生成 HTML
    let html = generator.generate("<h1>Test</h1>", "Test Page");
    assert!(html.contains("<h1>Test</h1>"));
    assert!(html.contains("Test Page"));

    // 测试切换主题
    generator.switch_theme("dark").unwrap();
    assert_eq!(generator.current_theme_name(), "dark");

    let html_dark = generator.generate("<h1>Dark Test</h1>", "Dark Page");
    assert!(html_dark.contains("<h1>Dark Test</h1>"));
    assert!(html_dark.contains("Dark Page"));

    // 切换到科技风主题
    generator.switch_theme("tech").unwrap();
    assert_eq!(generator.current_theme_name(), "tech");

    let html_tech = generator.generate("<h1>Tech Test</h1>", "Tech Page");
    assert!(html_tech.contains("<h1>Tech Test</h1>"));
    assert!(html_tech.contains("Tech Page"));
}

#[test]
fn test_html_generator_available_themes() {
    let themes = HtmlGenerator::available_themes();
    assert_eq!(themes.len(), 3);
    assert!(themes.contains(&"default"));
    assert!(themes.contains(&"dark"));
    assert!(themes.contains(&"tech"));
}

#[test]
fn test_html_generator_default() {
    let generator = HtmlGenerator::default();
    assert_eq!(generator.current_theme_name(), "default");
}

#[test]
fn test_all_themes_generate_html() {
    let themes = HtmlGenerator::available_themes();

    for theme_name in themes {
        let config = Config::default();
        let generator = HtmlGenerator::with_config_and_theme(config, theme_name);

        // 测试基本 HTML 生成
        let html = generator.generate("<h1>Theme Test</h1>", &format!("{} Theme", theme_name));
        assert!(html.contains("<h1>Theme Test</h1>"));
        assert!(html.contains(&format!("{} Theme", theme_name)));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
    }
}

#[test]
fn test_theme_config_access() {
    let mut config = Config::default();
    config.title = Some("Custom Title".to_string());
    config.description = Some("Custom Description".to_string());

    // 测试所有主题都能正确访问配置
    let generator = HtmlGenerator::with_config_and_theme(config.clone(), "default");
    assert_eq!(generator.current_theme_name(), "default");

    let html = generator.generate("<h1>Test</h1>", "Test");
    assert!(html.contains("Custom Title"));
}

#[test]
fn test_theme_static_resources() {
    let config = Config::default();

    // 测试默认主题的静态资源
    let default_theme = DefaultTheme::new(config.clone()).unwrap();
    let resources = default_theme.static_resources();
    assert!(resources.is_empty()); // 默认主题没有静态资源

    // 测试暗色主题的静态资源
    let dark_theme = DarkTheme::new(config.clone()).unwrap();
    let resources = dark_theme.static_resources();
    assert!(resources.is_empty());

    // 测试科技风主题的静态资源
    let tech_theme = TechTheme::new(config).unwrap();
    let resources = tech_theme.static_resources();
    assert!(resources.is_empty());
}

#[test]
fn test_theme_switch_fallback() {
    let config = Config::default();

    // 测试切换到未知主题时会回退到默认主题
    let mut generator = HtmlGenerator::with_config_and_theme(config, "unknown");
    assert_eq!(generator.current_theme_name(), "default");
}

#[test]
fn test_complex_content_rendering() {
    let config = Config::default();
    let generator = HtmlGenerator::with_config_and_theme(config, "default");

    let complex_content = r#"
        <h1>Main Title</h1>
        <p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
        <h2>Subtitle</h2>
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>Item 3</li>
        </ul>
        <pre><code>fn main() {
    println!("Hello");
}</code></pre>
    "#;

    let html = generator.generate(complex_content, "Complex Page");

    assert!(html.contains("Main Title"));
    assert!(html.contains("bold text"));
    assert!(html.contains("italic text"));
    assert!(html.contains("Subtitle"));
    assert!(html.contains("Item 1"));
    assert!(html.contains("Item 2"));
    assert!(html.contains("Item 3"));
    assert!(html.contains("fn main()"));
    assert!(html.contains("println!"));
}
