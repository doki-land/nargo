# 自定义主题

你可以创建自己的自定义主题来完全控制文档的外观。

## 主题结构

一个主题通常包含以下文件：

```
my-theme/
├── templates/
│   ├── page.html
│   └── layout.html
├── styles/
│   └── main.css
├── scripts/
│   └── main.js
└── theme.rs
```

## 创建自定义主题

### 1. 实现 Theme Trait

```rust
use nargo_document::theme::{Theme, PageContext};

pub struct MyTheme;

impl Theme for MyTheme {
    fn name(&self) -> &str {
        "my-theme"
    }
    
    fn render(&self, context: PageContext) -> Result<String, anyhow::Error> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <link rel="stylesheet" href="/styles/main.css">
</head>
<body>
    <nav>
        <!-- 导航栏 -->
    </nav>
    <main>
        {}
    </main>
    <footer>
        <!-- 页脚 -->
    </footer>
    <script src="/scripts/main.js"></script>
</body>
</html>
        "#, context.title, context.content);
        
        Ok(html)
    }
}
```

### 2. 配置主题

在 `nargodoc.config.toml` 中使用自定义主题：

```toml
[theme]
name = "my-theme"
path = "./themes/my-theme"
```

### 3. 提供静态资源

确保你的主题的静态资源（CSS、JS、图片等）被正确复制到输出目录。

## 主题 API

### PageContext

`PageContext` 提供了渲染页面所需的所有数据：

- `title` - 页面标题
- `content` - 页面 HTML 内容
- `nav` - 导航栏配置
- `sidebar` - 侧边栏配置
- `footer` - 页脚配置
- `locales` - 多语言信息
- `current_locale` - 当前语言

## 最佳实践

1. **响应式设计**：确保主题在不同设备上都能良好显示
2. **可访问性**：遵循 WCAG 可访问性标准
3. **性能优化**：压缩 CSS 和 JS，使用适当的缓存策略
4. **深色模式**：提供深色和浅色两种主题模式
5. **多语言**：支持多语言切换功能
