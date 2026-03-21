# 暗色主题

现代、护眼的暗色主题，适合长时间阅读文档。

## 特性

- 🌙 优雅的暗色配色方案
- 💙 蓝色强调色
- ✨ 平滑的过渡动画
- 🌓 减少眼睛疲劳
- 📱 完全响应式设计

## 配色方案

| 元素 | 颜色 | 用途 |
|------|------|------|
| 背景 | #0f0f0f | 页面背景 |
| 侧边栏/卡片 | #1a1a1a | 侧边栏和内容区域 |
| 边框 | #333 | 分隔线和边框 |
| 文本 | #e5e5e5 | 主要文本 |
| 次要文本 | #ccc | 辅助文本 |
| 强调文本 | #64c8ff | 链接和强调 |
| 悬停背景 | #2a2a2a | 交互元素悬停 |

## 使用方法

### 配置文件

在 `nargodoc.config.toml` 中启用暗色主题：

```toml
[theme]
name = "dark"
```

### API 使用

```rust
use nargo_document::generator::html::HtmlGenerator;
use nargo_document::config::Config;

let config = Config::default();
let generator = HtmlGenerator::with_config_and_theme(config, "dark");

let html = generator.generate("<h1>暗色主题</h1>", "我的文档");
```

## 自定义样式

虽然暗色主题已经有精美的默认样式，但你仍然可以通过自定义 CSS 进一步调整：

```css
/* 在你的自定义 CSS 中覆盖变量 */
:root {
    --primary-color: #64c8ff;
    --text-color: #e5e5e5;
    --bg-color: #0f0f0f;
    --card-bg: #1a1a1a;
    --border-color: #333;
}
```

## 设计理念

### 对比度

暗色主题特别注重对比度，确保文本在深色背景上清晰可读：

- 主要文本使用浅灰色 (#e5e5e5) 而不是纯白，减少刺眼感
- 链接使用天蓝色 (#64c8ff)，在暗色背景上清晰可见
- 代码块使用深色背景 (#1e1e1e) 和浅色文本 (#d4d4d4)

### 视觉层次

通过阴影和边框创建清晰的视觉层次：

- 侧边栏和内容区域有轻微的阴影
- 悬停状态有柔和的发光效果
- 边框使用中等深度的灰色，不会过于突兀

## 示例效果

### 标题和段落

暗色主题中的标题使用白色文本，带有轻微的发光效果：

```markdown
# 一级标题

这是一段普通文本，在暗色背景上清晰可读。

## 二级标题

- 列表项 1
- 列表项 2
- 列表项 3
```

### 代码高亮

代码块使用暗色主题配色，与整体风格协调：

```rust
fn main() {
    println!("Hello, dark theme!");
}
```

### 链接和强调

链接使用天蓝色，悬停时有发光效果：

```markdown
访问 [nargo-document](/) 了解更多信息。
```

## 最佳实践

### 配合语法高亮

暗色主题与暗色代码高亮主题配合最佳：

```toml
[markdown]
theme.light = "github-light"
theme.dark = "github-dark"
```

### 添加自定义 CSS

如果你想进一步自定义暗色主题，可以在配置中添加自定义 CSS：

```toml
[build]
custom_css = "./styles/dark-custom.css"
```

## 兼容性

暗色主题与所有 nargo-document 功能完全兼容：

- ✅ Markdown 渲染
- ✅ 代码高亮
- ✅ KaTeX 数学公式
- ✅ Mermaid 图表
- ✅ 自定义容器
- ✅ 多语言支持

## 相关主题

- [默认主题](./default.md) - 经典的浅色主题
- [科技风主题](./tech.md) - 科技风格主题
- [主题开发指南](./development-guide.md) - 创建自己的主题
