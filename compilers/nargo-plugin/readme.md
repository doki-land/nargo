# nargo-plugin

> Nargo 框架的插件系统内核，构建可扩展编译器生态的基石。

## 📖 简介

`nargo-plugin` 定义了 Nargo 编译器的扩展规范。它提供了一套完整的异步 Hook 机制，允许开发者在编译的初始化、解析、变换及打包等各个关键阶段介入并修改编译器行为，从而实现高度定制化的编译管线。

## ✨ 核心特性

- **完整生命周期 Hook**: 涵盖了 `on_init`, `on_pre_parse`, `on_parse`, `on_post_parse`, `on_pre_transform`, `on_transform`, `on_post_transform`, `on_pre_bundle`, `on_bundle`, `on_post_bundle`, `on_cleanup` 等核心阶段。
- **优先级管理**: 支持插件优先级设置，确保插件执行顺序的可控性。
- **配置系统**: 提供灵活的插件配置机制，支持自定义参数。
- **混合插件支持**: 设计上支持 Rust 原生插件，并为后续集成脚本语言插件（如 JS/Deno）预留了接口。
- **串行变换链**: 插件可以对代码进行链式处理，确保变换逻辑的顺序性与可预测性。
- **完整的元数据支持**: 插件可以提供名称、版本、描述、作者等元信息。

## 🏗️ 核心数据结构

### Plugin Trait

插件的核心契约，定义了插件名称及各个生命周期阶段的 Hook 接口。

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> Option<&str>;
    fn homepage(&self) -> Option<&str>;
    fn priority(&self) -> i32;
    
    fn on_init(&self, ctx: Arc<NargoContext>, config: &PluginConfig) -> Result<()>;
    fn on_pre_parse(&self, source: &str) -> Result<Option<String>>;
    fn on_parse(&self, source: &str) -> Result<Option<String>>;
    fn on_post_parse(&self, source: &str) -> Result<Option<String>>;
    fn on_pre_transform(&self, code: &str) -> Result<Option<String>>;
    fn on_transform(&self, code: &str) -> Result<Option<String>>;
    fn on_post_transform(&self, code: &str) -> Result<Option<String>>;
    fn on_pre_bundle(&self, bundle: &str) -> Result<Option<String>>;
    fn on_bundle(&self, bundle: &str) -> Result<Option<String>>;
    fn on_post_bundle(&self, bundle: &str) -> Result<Option<String>>;
    fn on_cleanup(&self) -> Result<()>;
}
```

### PluginManager

插件管理器，负责插件的注册、初始化以及在编译过程中调度 Hook 执行。

```rust
pub struct PluginManager {
    ctx: Arc<NargoContext>,
    plugins: Vec<Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
}
```

### PluginConfig

插件配置项，包含插件的基本信息和自定义配置。

```rust
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub priority: i32,
    pub config: HashMap<String, NargoValue>,
    pub enabled: bool,
}
```

### JsPlugin

专为脚本化扩展设计的插件模型（正在完善中）。

## 🚀 快速开始

### 安装

在您的 `Cargo.toml` 中添加：

```toml
[dependencies]
nargo-plugin = { workspace = true }
nargo-types = { workspace = true }
```

### 创建一个简单的插件

```rust
use nargo_plugin::Plugin;
use nargo_types::{NargoContext, PluginConfig, Result};
use std::sync::Arc;

pub struct HelloPlugin {
    name: String,
}

impl HelloPlugin {
    pub fn new() -> Self {
        Self {
            name: "hello-plugin".to_string(),
        }
    }
}

impl Plugin for HelloPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "A simple hello world plugin"
    }

    fn on_init(&self, _ctx: Arc<NargoContext>, _config: &PluginConfig) -> Result<()> {
        println!("Hello from HelloPlugin!");
        Ok(())
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!(
            "// Hello from HelloPlugin!\n{}",
            code
        );
        Ok(Some(transformed))
    }
}
```

### 使用插件

```rust
use nargo_plugin::PluginManager;
use nargo_types::NargoContext;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let plugin = HelloPlugin::new();
    manager.register(Box::new(plugin));

    manager.init_all().unwrap();

    let code = "console.log('Hello, World!');";
    let result = manager.transform(code.to_string()).await.unwrap();

    println!("Result:\n{}", result);
}
```

## 📚 文档

- [插件开发指南](./PLUGIN_GUIDE.md) - 详细的插件开发指南
- [API 文档](https://docs.rs/nargo-plugin) - 完整的 API 文档（待发布）

## 🔗 相关项目

- [nargo-compiler](../nargo-compiler) - 插件系统的核心消费者，在其编译流程中调用 Hook。
- [nargo-types](../nargo-types) - 为插件 Hook 提供统一的执行上下文（Context）。
- [nargo-transformer](../nargo-transformer) - 内置变换管线，包含示例插件实现。

## 🧪 测试

运行测试：

```bash
cargo test
```

## 📝 改进记录

### 2024-03-09 版本更新

1. **完善了核心接口**
   - 统一了 `Plugin` trait，添加了完整的元数据支持（版本、描述、作者、主页）
   - 扩展了生命周期钩子，支持更细粒度的控制
   - 添加了优先级系统，确保插件执行顺序可控

2. **增强了 PluginManager**
   - 添加了 `register_with_config` 方法，支持自定义配置
   - 实现了插件的启用/禁用功能
   - 添加了插件信息查询方法
   - 实现了自动按优先级排序

3. **添加了完整的测试覆盖**
   - 核心功能测试
   - 插件链测试
   - 优先级测试
   - 配置管理测试
   - 生命周期测试

4. **完善了文档**
   - 编写了详细的插件开发指南
   - 更新了 README 文档
   - 添加了多个示例插件

5. **保持了向后兼容性**
   - 所有方法都有默认实现
   - 可以逐步迁移现有插件

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](../../CONTRIBUTING.md) 了解更多信息。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](../../LICENSE) 文件了解详情。
