# 主题开发指南

本指南将帮助你为 nargo-document 创建自定义主题。

## 目录

- [主题概述](#主题概述)
- [核心概念](#核心概念)
- [创建新主题](#创建新主题)
- [主题 API](#主题-api)
- [模板系统](#模板系统)
- [样式设计](#样式设计)
- [测试主题](#测试主题)
- [最佳实践](#最佳实践)

## 主题概述

nargo-document 的主题系统采用了 trait 抽象设计，允许开发者轻松创建、切换和自定义文档的外观。

### 内置主题

nargo-document 目前提供三个内置主题：

1. **default** - 经典的浅色主题，适合大多数文档场景
2. **dark** - 现代的暗色主题，护眼舒适
3. **tech** - 科技风格主题，带有终端和黑客美学

## 核心概念

### Theme Trait

所有主题都必须实现 `Theme` trait：

```rust
pub trait Theme: Clone + Send + Sync {
    /// 获取主题名称
    fn name(&self) -> &str;
    
    /// 渲染页面
    fn render_page(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>>;
    
    /// 获取主题配置
    fn config(&self) -> &Config;
    
    /// 获取主题的静态资源（可选）
    fn static_resources(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
```

### ThemeFactory

使用 `ThemeFactory` 来创建主题实例：

```rust
use nargo_document::theme::ThemeFactory;
use nargo_document::config::Config;

let config = Config::default();
let theme = ThemeFactory::create("dark", config)?;
```

### PageContext

`PageContext` 包含了渲染页面所需的所有数据：

```rust
pub struct PageContext {
    pub page_title: String,
    pub site_title: String,
    pub content: String,
    pub nav_items: Vec<NavItem>,
    pub sidebar_groups: Vec<SidebarGroup>,
    pub current_path: String,
    pub has_footer: bool,
    pub has_footer_message: bool,
    pub footer_message: String,
    pub has_footer_copyright: bool,
    pub footer_copyright: String,
    pub current_lang: String,
    pub available_locales: Vec<LocaleInfo>,
    pub root_path: String,
}
```

## 创建新主题

### 步骤 1: 创建主题文件

在 `src/theme/` 目录下创建新的主题文件，例如 `my_theme.rs`：

```rust
//! 我的自定义主题

use askama::Template;
use crate::config::Config;
use super::Theme;
use super::default_theme::{LocaleInfo, SidebarGroup, SidebarLink, NavItem, PageContext};

/// 自定义主题页面模板上下文
#[derive(Debug, Clone, Template)]
#[template(path = "my_page.html")]
pub struct MyPageContext {
    pub page_title: String,
    pub site_title: String,
    pub content: String,
    pub nav_items: Vec<NavItem>,
    pub sidebar_groups: Vec<SidebarGroup>,
    pub current_path: String,
    pub has_footer: bool,
    pub has_footer_message: bool,
    pub footer_message: String,
    pub has_footer_copyright: bool,
    pub footer_copyright: String,
    pub current_lang: String,
    pub available_locales: Vec<LocaleInfo>,
    pub root_path: String,
}

impl From<PageContext> for MyPageContext {
    fn from(context: PageContext) -> Self {
        Self {
            page_title: context.page_title,
            site_title: context.site_title,
            content: context.content,
            nav_items: context.nav_items,
            sidebar_groups: context.sidebar_groups,
            current_path: context.current_path,
            has_footer: context.has_footer,
            has_footer_message: context.has_footer_message,
            footer_message: context.footer_message,
            has_footer_copyright: context.has_footer_copyright,
            footer_copyright: context.footer_copyright,
            current_lang: context.current_lang,
            available_locales: context.available_locales,
            root_path: context.root_path,
        }
    }
}

/// 自定义主题
#[derive(Clone)]
pub struct MyTheme {
    pub config: Config,
}

impl Theme for MyTheme {
    fn name(&self) -> &str {
        "my-theme"
    }
    
    fn render_page(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        let my_context: MyPageContext = context.clone().into();
        <MyPageContext as Template>::render(&my_context)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()).into())
    }
    
    fn config(&self) -> &Config {
        &self.config
    }
}

impl MyTheme {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
        })
    }
    
    pub fn site_title(&self) -> &str {
        &self.config.title
    }
}
```

### 步骤 2: 注册主题

在 `src/theme/mod.rs` 中注册新主题：

```rust
pub mod my_theme;
pub use my_theme::MyTheme;

impl ThemeFactory {
    pub fn create(theme_name: &str, config: Config) -> Result<Box<dyn Theme>, Box<dyn std::error::Error>> {
        match theme_name.to_lowercase().as_str() {
            "default" => Ok(Box::new(DefaultTheme::new(config)?)),
            "dark" => Ok(Box::new(DarkTheme::new(config)?)),
            "tech" => Ok(Box::new(TechTheme::new(config)?)),
            "my-theme" => Ok(Box::new(MyTheme::new(config)?)),
            _ => Err(format!("Unknown theme: {}", theme_name).into()),
        }
    }
    
    pub fn available_themes() -> Vec<&'static str> {
        vec!["default", "dark", "tech", "my-theme"]
    }
}
```

### 步骤 3: 创建 HTML 模板

在 `src/templates/` 目录下创建对应的 HTML 模板文件 `my_page.html`：

```html
<!DOCTYPE html>
<html lang="{{ current_lang }}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page_title }}</title>
    <style>
        /* 你的样式 */
        body {
            font-family: sans-serif;
            margin: 0;
            padding: 0;
        }
    </style>
</head>
<body>
    <header>
        <h1>{{ site_title }}</h1>
        <nav>
            {% for item in &nav_items %}
            <a href="{{ item.link }}">{{ item.text }}</a>
            {% endfor %}
        </nav>
    </header>
    
    <main>
        {{ content|safe }}
    </main>
    
    {% if has_footer %}
    <footer>
        {% if has_footer_message %}
        <p>{{ footer_message }}</p>
        {% endif %}
        {% if has_footer_copyright %}
        <p>{{ footer_copyright }}</p>
        {% endif %}
    </footer>
    {% endif %}
</body>
</html>
```

## 主题 API

### 使用 HtmlGenerator

```rust
use nargo_document::generator::html::HtmlGenerator;
use nargo_document::config::Config;

// 使用默认主题创建生成器
let mut generator = HtmlGenerator::new();

// 使用指定主题创建生成器
let config = Config::default();
let mut generator = HtmlGenerator::with_config_and_theme(config, "dark");

// 切换主题
generator.switch_theme("tech")?;

// 获取当前主题名称
let theme_name = generator.current_theme_name();

// 获取可用主题列表
let themes = HtmlGenerator::available_themes();

// 生成 HTML
let html = generator.generate("<h1>Hello</h1>", "My Page");
```

## 模板系统

nargo-document 使用 [Askama](https://github.com/djc/askama) 模板引擎，它的语法类似于 Jinja2 或 Django 模板。

### 模板语法

#### 变量输出

```html
<p>{{ page_title }}</p>
```

#### 循环

```html
{% for item in &nav_items %}
<li><a href="{{ item.link }}">{{ item.text }}</a></li>
{% endfor %}
```

#### 条件判断

```html
{% if has_footer %}
<footer>{{ footer_message }}</footer>
{% endif %}
```

#### 安全输出（不转义 HTML）

```html
{{ content|safe }}
```

### 模板上下文变量

| 变量 | 类型 | 描述 |
|------|------|------|
| page_title | String | 页面标题 |
| site_title | String | 站点标题 |
| content | String | 页面 HTML 内容 |
| nav_items | Vec<NavItem> | 导航栏项目 |
| sidebar_groups | Vec<SidebarGroup> | 侧边栏分组 |
| current_path | String | 当前页面路径 |
| has_footer | bool | 是否显示页脚 |
| has_footer_message | bool | 是否有页脚消息 |
| footer_message | String | 页脚消息 |
| has_footer_copyright | bool | 是否有版权信息 |
| footer_copyright | String | 版权信息 |
| current_lang | String | 当前语言 |
| available_locales | Vec<LocaleInfo> | 可用语言列表 |
| root_path | String | 根路径 |

## 样式设计

### CSS 变量

建议使用 CSS 变量来提高主题的可定制性：

```css
:root {
    --primary-color: #3b82f6;
    --secondary-color: #64748b;
    --text-color: #1e293b;
    --bg-color: #ffffff;
    --code-bg: #f1f5f9;
    --border-color: #e2e8f0;
}

body {
    color: var(--text-color);
    background: var(--bg-color);
}

a {
    color: var(--primary-color);
}
```

### 响应式设计

确保你的主题在不同设备上都能良好显示：

```css
@media (max-width: 768px) {
    .sidebar {
        display: none;
    }
    
    .main-content {
        margin-left: 0;
        padding: 20px;
    }
}
```

### 动画和过渡

添加平滑的过渡效果：

```css
a {
    transition: color 0.3s ease;
}

a:hover {
    color: #2563eb;
}
```

## 测试主题

### 运行现有测试

```bash
cd compilers/nargo-document
cargo test
```

### 创建主题测试

在 `tests/` 目录下创建主题测试文件：

```rust
#[cfg(test)]
mod tests {
    use nargo_document::generator::html::HtmlGenerator;
    use nargo_document::config::Config;
    
    #[test]
    fn test_all_themes() {
        let themes = HtmlGenerator::available_themes();
        
        for theme_name in themes {
            let config = Config::default();
            let generator = HtmlGenerator::with_config_and_theme(config, theme_name);
            
            let html = generator.generate("<h1>Test</h1>", "Test Page");
            assert!(html.contains("<h1>Test</h1>"));
            assert!(html.contains("Test Page"));
        }
    }
    
    #[test]
    fn test_theme_switch() {
        let mut generator = HtmlGenerator::new();
        assert_eq!(generator.current_theme_name(), "default");
        
        generator.switch_theme("dark").unwrap();
        assert_eq!(generator.current_theme_name(), "dark");
    }
}
```

## 最佳实践

### 1. 遵循设计原则

- **一致性**：保持整个主题的视觉风格一致
- **可读性**：确保文本易于阅读
- **可访问性**：遵循 WCAG 可访问性标准
- **响应式**：适配不同屏幕尺寸

### 2. 性能优化

- 压缩 CSS 和 JavaScript
- 使用适当的缓存策略
- 优化图片资源
- 减少重绘和重排

### 3. 代码组织

- 将样式分为语义化的部分
- 使用 CSS 预处理器（如 Sass）如果需要
- 保持模板简洁，逻辑放在 Rust 代码中

### 4. 文档化

- 为你的主题编写文档
- 提供使用示例
- 记录自定义配置选项

### 5. 测试

- 在不同浏览器中测试
- 在不同设备上测试
- 测试深色/浅色模式（如果支持）
- 测试不同语言

## 示例主题

查看内置主题的实现作为参考：

- [默认主题](../../src/theme/default_theme.rs)
- [暗色主题](../../src/theme/dark_theme.rs)
- [科技风主题](../../src/theme/tech_theme.rs)

## 贡献主题

如果你创建了一个很棒的主题，欢迎贡献到 nargo-document 项目！

1. Fork 项目
2. 创建你的主题分支
3. 提交你的更改
4. 推送到分支
5. 创建 Pull Request

## 获得帮助

如果你在开发主题时有任何问题：

- 查看 [GitHub Issues](https://github.com/nargo/nargo/issues)
- 加入我们的社区讨论
- 阅读源代码了解更多细节

---

祝你创建出优秀的主题！🎨
