# 主题系统

nargo-document 提供了灵活的主题系统，支持自定义文档的外观和布局。

## 内置主题

nargo-document 目前提供三个精心设计的内置主题：

- [默认主题](./default.md) - 经典的浅色主题，适合大多数文档场景
- [暗色主题](./dark.md) - 现代的暗色主题，护眼舒适
- [科技风主题](./tech.md) - 科技风格主题，带有终端和黑客美学

## 快速开始

### 选择主题

在 `nargodoc.config.toml` 中选择主题：

```toml
[theme]
name = "dark"
```

### 使用 API 切换主题

```rust
use nargo_document::generator::html::HtmlGenerator;
use nargo_document::config::Config;

let config = Config::default();
let mut generator = HtmlGenerator::with_config_and_theme(config, "default");

// 切换到暗色主题
generator.switch_theme("dark")?;

// 获取可用主题列表
let themes = HtmlGenerator::available_themes();
println!("Available themes: {:?}", themes);
```

## 主题配置

每个主题都可以有自己的配置项，请参考具体主题的文档：

- [默认主题配置](./default.md)
- [暗色主题配置](./dark.md)
- [科技风主题配置](./tech.md)

## 开发自定义主题

如果你想开发自己的主题，请参考：

- [主题开发指南](./development-guide.md) - 详细的主题开发文档
- [创建自定义主题](./custom.md) - 如何创建自己的主题

## 主题特性

### 所有主题都支持：

✅ 响应式布局  
✅ 导航栏  
✅ 侧边栏  
✅ 页脚  
✅ 多语言支持  
✅ 语法高亮  
✅ Markdown 全特性支持  

### 主题切换

使用 ThemeFactory 可以轻松创建和切换主题：

```rust
use nargo_document::theme::{ThemeFactory, Theme};
use nargo_document::config::Config;

let config = Config::default();

// 创建不同的主题
let default_theme: Box<dyn Theme> = ThemeFactory::create("default", config.clone())?;
let dark_theme: Box<dyn Theme> = ThemeFactory::create("dark", config.clone())?;
let tech_theme: Box<dyn Theme> = ThemeFactory::create("tech", config)?;

// 使用主题
println!("Theme name: {}", default_theme.name());
```

## 示例配置

这里是一个完整的配置示例，展示如何使用主题系统：

```toml
[general]
title = "我的文档"
description = "使用 nargo-document 构建的文档"

[theme]
name = "dark"

[theme_config]
site_title = "我的文档"

[theme_config.nav]
[[theme_config.nav.items]]
text = "首页"
link = "/"

[[theme_config.nav.items]]
text = "指南"
link = "/guide/"

[theme_config.footer]
message = "© 2024 My Project"
copyright = "All rights reserved"
```

## 下一步

- 查看 [主题开发指南](./development-guide.md) 学习如何创建自定义主题
- 尝试不同的 [内置主题](./default.md)
- 参考 [API 文档](../readme.md) 了解更多功能
