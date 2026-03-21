# Nargo Document - 文档生成库

提供 Markdown 文档的渲染和多种输出格式的生成功能，兼容 VuTeX 配置格式

## 示例

```rust
use nargo_document::{config::Config, generator::Generator};

// 创建默认配置
let config = Config::default();

// 创建文档生成器
let mut generator = Generator::new(config);

// 生成文档
generator.generate(".", "dist").expect("Failed to generate documentation");
```

## 功能特性

- 支持 Markdown 渲染
- 多种输出格式：HTML、PDF、静态文件
- 主题系统支持
- 插件扩展机制
- 兼容 VuTeX 配置格式
- 开发服务器支持

## 主题

- default - 默认主题
- dark - 暗色主题
- tech - 科技风主题

## 插件

- KaTeX - 数学公式渲染
- Mermaid - 图表渲染
- Prism - 代码高亮
- Shiki - 代码高亮
- Container - 容器组件
