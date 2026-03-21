# 默认主题

nargo-document 提供了一个功能丰富的默认主题。

## 特性

- 响应式布局
- 导航栏
- 侧边栏
- 页脚
- 多语言支持
- 搜索功能
- 深色/浅色模式切换

## 配置

在 `nargodoc.config.toml` 中配置主题：

```toml
[theme_config]
site_title = "我的文档"
site_description = "这是一个使用 nargo-document 构建的文档网站"

[theme_config.nav]
[[theme_config.nav.items]]
text = "首页"
link = "/"

[[theme_config.nav.items]]
text = "指南"
link = "/guide/"

[[theme_config.nav.items]]
text = "API"
link = "/api/"

[theme_config.sidebar]
[[theme_config.sidebar.groups]]
title = "快速开始"
items = [
    { text = "安装", link = "/guide/install.md" },
    { text = "配置", link = "/guide/config.md" }
]

[[theme_config.sidebar.groups]]
title = "高级主题"
items = [
    { text = "插件", link = "/guide/plugins.md" },
    { text = "主题", link = "/guide/themes.md" }
]

[theme_config.footer]
message = "© 2024 My Project"
copyright = "All rights reserved"

[[theme_config.footer.links]]
text = "GitHub"
link = "https://github.com"

[[theme_config.footer.links]]
text = "Twitter"
link = "https://twitter.com"

[theme_config.locales]
default = "zh-CN"

[[theme_config.locales.available]]
code = "zh-CN"
name = "简体中文"

[[theme_config.locales.available]]
code = "en-US"
name = "English"
```

## 页面上下文

默认主题使用 `PageContext` 来传递页面数据：

```rust
pub struct PageContext {
    pub title: String,
    pub content: String,
    pub nav: Vec<NavItem>,
    pub sidebar: Vec<SidebarGroup>,
    pub footer: Footer,
    pub locales: LocaleInfo,
    pub current_locale: String,
}
```

## 自定义样式

可以通过自定义 CSS 来修改主题样式：

```css
:root {
    --primary-color: #3b82f6;
    --secondary-color: #64748b;
    --text-color: #1e293b;
    --bg-color: #ffffff;
}

.dark {
    --text-color: #f1f5f9;
    --bg-color: #1e293b;
}
```
