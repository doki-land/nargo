# 科技风主题

充满未来感和科技美学的主题，带有终端和黑客风格。

## 特性

- 🚀 赛博朋克风格配色
- 💚 绿色主色调（矩阵绿）
- 💜 洋红色强调
- ⌨️ 等宽字体
- 🖥️ 网格背景
- ✨ 发光效果
- 🔲 终端风格装饰

## 配色方案

| 元素 | 颜色 | 用途 |
|------|------|------|
| 背景 | 渐变 #0a0a1a → #1a1a2e | 页面背景 |
| 卡片背景 | rgba(10, 10, 30, 0.85) | 内容区域 |
| 主色 | #00ff88 | 标题、链接、强调 |
| 次色 | #ff00ff | 装饰、标记 |
| 第三色 | #00cccc | 次要文本 |
| 文本 | #00ff88 | 主要文本 |
| 次要文本 | #aaffcc | 辅助文本 |
| 边框 | rgba(0, 255, 136, 0.3) | 分隔线 |

## 使用方法

### 配置文件

在 `nargodoc.config.toml` 中启用科技风主题：

```toml
[theme]
name = "tech"
```

### API 使用

```rust
use nargo_document::generator::html::HtmlGenerator;
use nargo_document::config::Config;

let config = Config::default();
let generator = HtmlGenerator::with_config_and_theme(config, "tech");

let html = generator.generate("<h1>科技风主题</h1>", "我的文档");
```

## 设计元素

### 网格背景

科技风主题带有微妙的网格背景，营造数字空间的感觉：

```css
body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-image: 
        linear-gradient(rgba(0, 255, 136, 0.03) 1px, transparent 1px),
        linear-gradient(90deg, rgba(0, 255, 136, 0.03) 1px, transparent 1px);
    background-size: 50px 50px;
}
```

### 终端风格装饰

主题包含多种终端风格的装饰元素：

- 侧边栏顶部显示 `[SYSTEM]`
- 标题前有 `>` 提示符
- 链接前有 `$` 提示符
- 分组标题前有 `//` 注释标记
- 内容区域顶部有 `DOCUMENT://` 标签
- 页脚前有 `END_OF_TRANSMISSION`

### 发光效果

交互元素带有柔和的发光效果：

- 悬停链接有绿色发光
- 激活状态有洋红色发光
- 标题有轻微的绿色光晕

## 字体

科技风主题使用等宽字体，营造代码/终端的感觉：

```css
body {
    font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}
```

推荐的字体：
- **SF Mono** - macOS 内置
- **Fira Code** - 免费开源，支持连字
- **Consolas** - Windows 内置
- **JetBrains Mono** - 免费开源

## 示例效果

### 标题

标题带有 Markdown 风格的前缀：

```markdown
# 一级标题 → # 一级标题
## 二级标题 → ## 二级标题
### 三级标题 → ### 三级标题
```

### 代码块

代码块带有终端风格：

```rust
fn main() {
    println!("Welcome to the matrix!");
}
```

### 列表

列表使用洋红色的项目符号：

```markdown
- 项目 1
- 项目 2
- 项目 3
```

### 链接

链接带有箭头指示器：

```markdown
访问 [nargo-document](/) → 访问 nargo-document →
```

## 自定义样式

虽然科技风主题已经有很独特的风格，但你仍然可以自定义：

```css
/* 修改主色调 */
:root {
    --tech-primary: #00ff88;
    --tech-secondary: #ff00ff;
    --tech-tertiary: #00cccc;
}

/* 修改网格大小 */
body::before {
    background-size: 30px 30px;
}

/* 调整发光强度 */
a:hover {
    text-shadow: 0 0 20px rgba(0, 255, 136, 0.8);
}
```

## 最佳实践

### 配合语法高亮

科技风主题与暗色代码高亮主题配合最佳：

```toml
[markdown]
theme.light = "github-light"
theme.dark = "one-dark-pro"
```

### 内容建议

科技风主题特别适合：

- 📚 技术文档
- 💻 API 参考
- 🔧 开发指南
- 🎮 游戏/电子项目
- 🤖 人工智能/机器学习文档

### 注意事项

- 确保在明亮环境下也有足够的对比度
- 考虑提供主题切换选项
- 测试在不同设备上的可读性

## 兼容性

科技风主题与所有 nargo-document 功能完全兼容：

- ✅ Markdown 渲染
- ✅ 代码高亮
- ✅ KaTeX 数学公式
- ✅ Mermaid 图表
- ✅ 自定义容器
- ✅ 多语言支持

## 相关主题

- [默认主题](./default.md) - 经典的浅色主题
- [暗色主题](./dark.md) - 现代的暗色主题
- [主题开发指南](./development-guide.md) - 创建自己的主题

## 灵感来源

科技风主题的设计灵感来自：

- 🎬 《黑客帝国》电影
- 💾 经典终端界面
- 🎮 赛博朋克游戏
- 🔬 科学幻想小说
- ⚡ 80/90 年代计算机界面
