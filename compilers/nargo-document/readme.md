# nargo-document

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20documentation%20generator%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Document Logo" width="150" height="150">
  <h3>📄 Nargo 框架的文档生成器</h3>
  <p>用于生成 Nargo 项目的文档，支持多种格式和丰富的插件系统</p>
</div>

## 🎯 简介

`nargo-document` 是 Nargo 框架的文档生成器，用于生成 Nargo 项目的文档。它支持多种输出格式，提供完整的插件系统和灵活的主题系统，为 Nargo 项目提供专业、美观的文档解决方案。作为 Nargo 生态系统的重要组成部分，`nargo-document` 帮助开发者快速生成高质量的项目文档。

## ✨ 核心特性

### 多格式支持
- **HTML 文档**: 生成美观、响应式的 HTML 文档站点
- **Markdown 文档**: 生成标准的 Markdown 文档
- **PDF 文档**: 生成高质量的 PDF 文档
- **静态资源管理**: 自动处理和优化静态资源

### 插件系统
- **完整的插件架构**: 支持在文档生成过程中扩展和增强功能
- **内置插件**: KaTeX（数学公式）、Mermaid（图表）、Shiki（代码高亮）、Prism（代码高亮）、Container（自定义容器）
- **插件钩子**: 插件通过钩子方法在文档渲染的不同阶段进行介入
- **自定义插件**: 支持开发和集成自定义插件

### 主题系统
- **默认主题**: 提供完整的文档站点主题和样式
- **响应式设计**: 适配不同屏幕尺寸
- **多语言支持**: 支持多语言文档
- **自定义主题**: 支持创建和使用自定义主题

### 文档服务器
- **本地预览**: 提供本地文档服务器，便于预览和测试
- **实时更新**: 支持文档的实时更新
- **静态文件服务**: 提供静态文件服务功能

### 高级功能
- **前置元数据**: 支持 Markdown 前置元数据（Frontmatter）
- **导航生成**: 自动生成文档导航
- **搜索功能**: 内置文档搜索功能
- **代码高亮**: 支持多种编程语言的代码高亮

## 🚀 使用方法

### 命令行使用

```bash
# 生成 HTML 文档
nargo docs generate --format html --output ./docs

# 生成 Markdown 文档
nargo docs generate --format markdown --output ./docs

# 生成 PDF 文档
nargo docs generate --format pdf --output ./docs

# 启动文档服务器
nargo docs serve --port 8080

# 启动文档服务器并监听变化
nargo docs serve --port 8080 --watch
```

### API 使用

```rust
use nargo_document::{DocumentGenerator, DocumentOptions};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // 创建文档生成器
    let generator = DocumentGenerator::new();

    // 配置生成选项
    let options = DocumentOptions {
        input_dir: Path::new("./docs-source").to_path_buf(),
        output_dir: Path::new("./docs").to_path_buf(),
        format: "html".into(),
        theme: "default".into(),
        plugins: vec!["katex", "mermaid", "shiki", "container"],
        ..Default::default()
    };

    // 生成文档
    generator.generate(&options)?;

    println!("文档生成成功！");
    Ok(())
}
```

## 🔧 插件系统

### 插件架构

插件系统包含以下核心组件：

- **DocumentPlugin Trait**: 定义插件需要实现的钩子方法
- **PluginMeta**: 插件元数据，包含名称、版本、描述
- **PluginContext**: 插件上下文，提供文档内容、前置元数据、文档路径
- **PluginRegistry**: 插件注册表，用于管理和注册文档插件

### 内置插件

#### KaTeX 插件

KaTeX 数学公式渲染插件，支持行内公式和块级公式。

**功能：**
- 行内公式：`$E=mc^2$`
- 块级公式：`$$\sum_{i=1}^n i = \frac{n(n+1)}{2}$$`

**使用示例：**

```markdown
Einstein's equation $E=mc^2$ is famous.

The sum of the first n integers is:

$$\sum_{i=1}^n i = \frac{n(n+1)}{2}$$
```

#### Mermaid 插件

Mermaid 图表渲染插件，支持流程图、时序图、甘特图等。

**功能：**
- 流程图
- 时序图
- 类图
- 状态图
- 甘特图

**使用示例：**

```markdown
```mermaid
graph TD
    A[Start] --> B{Is it working?}
    B -->|Yes| C[Great!]
    B -->|No| D[Debug]
```
```

#### Shiki 插件

Shiki 代码高亮插件，提供代码块语法高亮功能。

**功能：**
- 支持多种编程语言语法高亮
- 高质量的语法着色

**使用示例：**

```markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
```

#### Prism 插件

Prism 代码高亮插件，提供代码块语法高亮功能。

**功能：**
- 支持多种编程语言语法高亮
- 轻量级的语法着色库

**使用示例：**

```markdown
```javascript
function hello() {
    console.log("Hello, world!");
}
```
```

#### Container 插件

自定义容器插件，提供信息提示、警告提示等容器组件。

**内置容器类型：**
- `tip`：提示信息
- `warning`：警告信息
- `danger`：危险信息
- `info`：信息提示

**使用示例：**

```markdown
:::tip
这是一个提示信息
:::

:::warning
这是一个警告信息
:::

:::danger
这是一个危险信息
:::

:::info
这是一个信息提示
:::
```

**自定义容器配置：**

可以通过配置自定义容器，包括标题、图标和 CSS 类。

## 🎨 主题系统

### 默认主题

默认主题提供完整的文档站点主题和样式，包括：

- 响应式布局
- 导航栏
- 侧边栏导航
- 页脚
- 多语言支持

### 主题配置

默认主题使用 `PageContext` 进行页面渲染，包含以下配置项：

- `page_title`：页面标题
- `site_title`：站点标题
- `content`：页面内容
- `nav_items`：导航栏项目
- `sidebar_groups`：侧边栏组
- `current_path`：当前页面路径
- `has_footer`：是否有页脚
- `footer_message`：页脚消息
- `footer_copyright`：页脚版权
- `current_lang`：当前语言
- `available_locales`：可用语言列表
- `root_path`：相对于根目录的路径前缀

## 🔧 配置选项

配置文件：`nargodoc.config.toml`

```toml
[general]
title = "Nargo Documentation"
author = "Nargo Team"
description = "Nargo framework documentation"

[output]
directory = "./docs"
format = "html"

[server]
port = 8080
host = "127.0.0.1"
watch = true

[generator.html]
template = "default"
theme = "default"

[plugins]
enabled = ["katex", "mermaid", "shiki", "container"]

[plugins.container]
default_icons = true

[generator.pdf]
margin = [20, 20, 20, 20]
font = "Helvetica"
font_size = 12

[theme]
name = "default"

[theme.nav]
items = [
    { text = "首页", link = "/" },
    { text = "文档", link = "/docs/" },
    { text = "API", link = "/api/" }
]

[theme.sidebar]
groups = [
    { 
        text = "入门指南",
        items = [
            { text = "快速开始", link = "/guide/quick-start" },
            { text = "安装", link = "/guide/installation" }
        ]
    },
    {
        text = "核心概念",
        items = [
            { text = "架构", link = "/concepts/architecture" },
            { text = "配置", link = "/concepts/configuration" }
        ]
    }
]

[theme.footer]
enabled = true
message = "Powered by Nargo Document"
copyright = "© 2024 Nargo Team"

[theme.locales]
default = "zh-CN"
available = [
    { code = "zh-CN", label = "简体中文" },
    { code = "en-US", label = "English" }
]
```

## 📊 与其他文档生成工具对比

| 特性 | nargo-document | MkDocs | Docusaurus | Sphinx |
|------|---------------|--------|------------|--------|
| 多格式支持 | ✅ (HTML, Markdown, PDF) | ✅ (HTML) | ✅ (HTML) | ✅ (HTML, PDF) |
| 插件系统 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 主题系统 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 文档服务器 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ❌ (有限) |
| 数学公式支持 | ✅ (KaTeX) | ✅ (通过插件) | ✅ (通过插件) | ✅ (通过插件) |
| 图表支持 | ✅ (Mermaid) | ✅ (通过插件) | ✅ (通过插件) | ✅ (通过插件) |
| 代码高亮 | ✅ (Shiki, Prism) | ✅ (通过插件) | ✅ (通过插件) | ✅ (通过插件) |
| 多语言支持 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 性能 | ⚡⚡⚡ (快速) | ⚡⚡ (中等) | ⚡ (一般) | ⚡ (一般) |

## 🎯 应用场景

### 项目文档
在 Nargo 项目中，`nargo-document` 可以生成专业、美观的项目文档，帮助开发者和用户了解项目的功能和使用方法。

### API 文档
在 API 开发中，`nargo-document` 可以生成详细的 API 文档，包括接口说明、参数描述和示例代码。

### 技术文档
在技术团队中，`nargo-document` 可以生成技术文档，包括架构设计、技术方案和最佳实践。

### 教程和指南
在教育和培训中，`nargo-document` 可以生成教程和指南，帮助用户快速上手和掌握相关技术。

## 🔧 核心 API

### DocumentGenerator
文档生成器，负责生成文档：
- **new**: 创建新的文档生成器实例
- **generate**: 生成文档
- **serve**: 启动文档服务器
- **watch**: 监听文件变化并自动更新文档

### DocumentOptions
文档生成选项，控制文档生成的行为：
- **input_dir**: 输入目录
- **output_dir**: 输出目录
- **format**: 输出格式（html, markdown, pdf）
- **theme**: 主题名称
- **plugins**: 启用的插件列表
- **watch**: 是否监听文件变化

### PluginRegistry
插件注册表，管理和注册文档插件：
- **new**: 创建新的插件注册表
- **register**: 注册插件
- **get_plugin**: 获取插件
- **get_enabled_plugins**: 获取启用的插件

### ThemeManager
主题管理器，管理和应用文档主题：
- **new**: 创建新的主题管理器
- **load_theme**: 加载主题
- **apply_theme**: 应用主题到文档

## 🏗️ 目录结构

- `src/` - 源代码
  - `config/` - 配置管理
  - `generator/` - 文档生成器
    - `html/` - HTML 生成器
    - `markdown/` - Markdown 生成器
    - `pdf/` - PDF 生成器
    - `static_/` - 静态资源生成器
  - `plugin/` - 插件系统
    - `katex.rs` - KaTeX 数学公式插件
    - `mermaid.rs` - Mermaid 图表插件
    - `shiki.rs` - Shiki 代码高亮插件
    - `prism.rs` - Prism 代码高亮插件
    - `container.rs` - 自定义容器插件
    - `mod.rs` - 插件系统核心
  - `theme/` - 主题系统
    - `default_theme.rs` - 默认主题
    - `dark_theme.rs` - 暗黑主题
    - `tech_theme.rs` - 技术主题
    - `mod.rs` - 主题系统核心
  - `server/` - 文档服务器
  - `lib.rs` - 库入口
- `tests/` - 测试代码
- `nargodoc.config.toml` - 配置文件

## 🔗 相关项目

- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 基础类型定义
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): 命令行工具集成
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): 核心编译引擎
- [nargo-bundler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-bundler): 代码打包器

## 📚 文档

- **[官方文档](https://docs.nargo.dev/document)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/document/api)** - 完整的 API 参考
- **[插件开发](https://docs.nargo.dev/document/plugins)** - 插件开发指南
- **[主题开发](https://docs.nargo.dev/document/themes)** - 主题开发指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
