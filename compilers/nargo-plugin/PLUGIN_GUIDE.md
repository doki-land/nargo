# Nargo 插件开发指南

本文档将指导您如何开发 Nargo 插件系统的插件。

## 目录

1. [简介](#简介)
2. [快速开始](#快速开始)
3. [插件生命周期](#插件生命周期)
4. [核心概念](#核心概念)
5. [示例插件](#示例插件)
6. [最佳实践](#最佳实践)

## 简介

Nargo 插件系统允许开发者扩展 Nargo 编译器的功能。通过实现 `Plugin` trait，您可以在编译的各个关键阶段介入并修改编译器行为。

## 快速开始

### 1. 创建插件项目

首先，创建一个新的 Rust 库项目：

```bash
cargo new --lib my-nargo-plugin
cd my-nargo-plugin
```

### 2. 添加依赖

在 `Cargo.toml` 中添加必要的依赖：

```toml
[package]
name = "my-nargo-plugin"
version = "0.1.0"
edition = "2024"

[dependencies]
nargo-plugin = { path = "../nargo/compilers/nargo-plugin" }
nargo-types = { path = "../nargo/compilers/nargo-types" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 3. 实现插件

在 `src/lib.rs` 中实现您的插件：

```rust
use nargo_plugin::Plugin;
use nargo_types::{NargoContext, PluginConfig, Result};
use std::sync::Arc;

pub struct MyPlugin {
    name: String,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "my-plugin".to_string(),
        }
    }
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "My awesome Nargo plugin"
    }

    fn on_init(&self, _ctx: Arc<NargoContext>, _config: &PluginConfig) -> Result<()> {
        println!("MyPlugin initialized!");
        Ok(())
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!(
            "// Transformed by MyPlugin\n{}",
            code
        );
        Ok(Some(transformed))
    }
}
```

### 4. 使用插件

在您的 Nargo 项目中使用这个插件：

```rust
use nargo_plugin::PluginManager;
use nargo_types::NargoContext;
use std::sync::Arc;
use my_nargo_plugin::MyPlugin;

#[tokio::main]
async fn main() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = MyPlugin::new();
    manager.register(Box::new(plugin));

    manager.init_all().unwrap();

    let code = "console.log('Hello, World!');";
    let result = manager.transform(code.to_string()).await.unwrap();

    println!("Transformed code:\n{}", result);
}
```

## 插件生命周期

Nargo 插件系统提供了多个生命周期钩子，让您可以在编译的不同阶段介入：

### 初始化阶段

- `on_init`: 插件初始化时调用，用于设置插件的初始状态

### 解析阶段

- `on_pre_parse`: 解析前调用
- `on_parse`: 解析中调用
- `on_post_parse`: 解析后调用

### 变换阶段

- `on_pre_transform`: 变换前调用
- `on_transform`: 变换中调用
- `on_post_transform`: 变换后调用

### 打包阶段

- `on_pre_bundle`: 打包前调用
- `on_bundle`: 打包中调用
- `on_post_bundle`: 打包后调用

### 清理阶段

- `on_cleanup`: 编译完成后调用，用于清理资源

所有这些钩子方法都有默认实现，您只需要重写您需要的方法即可。

## 核心概念

### Plugin Trait

`Plugin` trait 是所有 Nargo 插件的核心。它定义了插件的基本行为和生命周期钩子。

### PluginManager

`PluginManager` 负责管理所有注册的插件。它提供了以下功能：

- 注册插件
- 初始化插件
- 按优先级执行插件
- 启用/禁用插件
- 获取插件信息

### PluginConfig

`PluginConfig` 包含插件的配置信息：

- `name`: 插件名称
- `version`: 插件版本
- `description`: 插件描述
- `author`: 插件作者（可选）
- `homepage`: 插件主页（可选）
- `priority`: 插件优先级（数值越小，优先级越高）
- `config`: 插件自定义配置参数
- `enabled`: 插件启用状态

### 优先级

插件优先级决定了插件的执行顺序。数值越小，优先级越高，执行越晚（因为高优先级的插件会在低优先级插件之后应用）。

例如：
- 优先级 1 的插件会在优先级 10 的插件之后执行
- 这意味着优先级 1 的插件的变换结果会覆盖优先级 10 的插件的变换结果

## 示例插件

### 1. 日志插件

```rust
use nargo_plugin::Plugin;
use nargo_types::{NargoContext, PluginConfig, Result};
use std::sync::Arc;

pub struct LoggingPlugin {
    name: String,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            name: "logging-plugin".to_string(),
        }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Logs all plugin lifecycle events"
    }

    fn on_init(&self, _ctx: Arc<NargoContext>, _config: &PluginConfig) -> Result<()> {
        tracing::info!("LoggingPlugin initialized");
        Ok(())
    }

    fn on_pre_parse(&self, source: &str) -> Result<Option<String>> {
        tracing::debug!("Pre-parsing: {} bytes", source.len());
        Ok(None)
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        tracing::debug!("Transforming: {} bytes", code.len());
        Ok(None)
    }

    fn on_cleanup(&self) -> Result<()> {
        tracing::info!("LoggingPlugin cleaned up");
        Ok(())
    }
}
```

### 2. 代码变换插件

```rust
use nargo_plugin::Plugin;
use nargo_types::Result;

pub struct TransformPlugin {
    name: String,
}

impl TransformPlugin {
    pub fn new() -> Self {
        Self {
            name: "transform-plugin".to_string(),
        }
    }
}

impl Plugin for TransformPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Adds a header comment to transformed code"
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!(
            "// Transformed by TransformPlugin\n{}",
            code
        );
        Ok(Some(transformed))
    }
}
```

### 3. 代码压缩插件

```rust
use nargo_plugin::Plugin;
use nargo_types::Result;

pub struct MinifyPlugin {
    name: String,
}

impl MinifyPlugin {
    pub fn new() -> Self {
        Self {
            name: "minify-plugin".to_string(),
        }
    }

    fn minify_code(&self, code: &str) -> String {
        code
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
            .replace("  ", " ")
            .replace("( ", "(")
            .replace(" )", ")")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("; ", ";")
            .replace(" ,", ",")
            .replace(" + ", "+")
            .replace(" = ", "=")
    }
}

impl Plugin for MinifyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Minifies JavaScript code"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let minified = self.minify_code(code);
        Ok(Some(minified))
    }
}
```

## 最佳实践

### 1. 错误处理

始终正确处理错误，并提供有意义的错误信息：

```rust
fn on_transform(&self, code: &str) -> Result<Option<String>> {
    if code.is_empty() {
        return Err(nargo_types::Error::external_error(
            "MyPlugin".to_string(),
            "Empty code provided".to_string(),
            nargo_types::Span::unknown(),
        ));
    }
    // ... 处理代码
    Ok(Some(transformed))
}
```

### 2. 性能考虑

- 避免在插件钩子中执行耗时操作
- 对于复杂的变换，考虑使用异步操作
- 缓存中间结果以提高性能

### 3. 配置管理

使用 `PluginConfig` 来提供插件的可配置性：

```rust
fn on_init(&self, _ctx: Arc<NargoContext>, config: &PluginConfig) -> Result<()> {
    if let Some(value) = config.config.get("debug") {
        if let Some(debug) = value.as_bool() {
            // 设置调试模式
        }
    }
    Ok(())
}
```

### 4. 优先级设置

合理设置插件优先级：
- 基础变换插件使用较低的优先级（较高的数值）
- 最终处理插件使用较高的优先级（较低的数值）

### 5. 文档

为您的插件提供完整的文档：
- 清晰的插件描述
- 配置选项说明
- 使用示例
- 注意事项和限制

### 6. 测试

为您的插件编写完整的测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.name(), "my-plugin");
    }

    #[tokio::test]
    async fn test_transform() {
        let plugin = MyPlugin::new();
        let input = "test code";
        let result = plugin.on_transform(input);
        assert!(result.is_ok());
        // ... 验证结果
    }
}
```

## 总结

Nargo 插件系统提供了一个强大而灵活的方式来扩展 Nargo 编译器的功能。通过遵循本指南，您可以创建高质量的插件，为 Nargo 生态系统做出贡献。

如果您有任何问题或建议，请查看 [Nargo 文档](../README.md) 或提交 Issue。
