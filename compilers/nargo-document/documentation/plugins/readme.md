# 插件系统

nargo-document 提供了强大的插件系统，可以扩展文档的功能。

## 内置插件

- [KaTeX](./katex.md) - 数学公式渲染
- [Mermaid](./mermaid.md) - 图表渲染
- [Shiki](./code-highlight.md) - 代码高亮（基于 TextMate）
- [Prism](./code-highlight.md) - 代码高亮（轻量级）
- [Container](./container.md) - 自定义容器

## 配置插件

在 `nargodoc.config.toml` 中配置插件：

```toml
[plugins]
katex = true
mermaid = true
shiki = true
container = true
```

## 插件架构

### DocumentPlugin Trait

所有插件都需要实现 `DocumentPlugin` trait：

```rust
pub trait DocumentPlugin: Send + Sync {
    fn meta(&self) -> &PluginMeta;
    
    fn setup(&mut self, config: Option<HashMap<String, NargoValue>>) {
        let _ = config;
    }
    
    fn before_render(&self, context: PluginContext) -> PluginContext {
        context
    }
    
    fn after_render(&self, context: PluginContext) -> PluginContext {
        context
    }
}
```

### 插件钩子

- `setup` - 插件初始化时调用
- `before_render` - Markdown 解析后、HTML 渲染前调用
- `after_render` - HTML 渲染后调用

### 插件注册

使用 `PluginRegistry` 注册插件：

```rust
let mut registry = PluginRegistry::new();
registry.register(KaTeXPlugin::new());
registry.register(MermaidPlugin::new());
```
