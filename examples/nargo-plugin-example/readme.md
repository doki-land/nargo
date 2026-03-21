# Nargo Plugin 插件开发示例

这个示例展示了如何开发和使用 Nargo 插件系统。

## 功能演示

1. **插件基础** - 如何实现和注册基本插件
2. **插件链** - 如何使用多个插件组成处理链
3. **多插件协作** - 多个插件如何协同工作

## 项目结构

```
nargo-plugin-example/
├── Cargo.toml          # 项目配置文件
├── README.md           # 说明文档
├── src/
│   ├── main.rs         # 主程序代码
│   └── plugins.rs      # 插件实现
└── plugins/            # 插件目录（预留）
```

## 运行示例

### 构建项目

```bash
cargo build
```

### 运行示例

```bash
cargo run
```

### 运行测试

```bash
cargo test
```

## 插件系统架构

### Plugin Trait

所有 Nargo 插件都必须实现 `Plugin` trait：

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    
    fn on_init(&self, _ctx: Arc<NargoContext>) -> Result<()> {
        Ok(())
    }
    
    fn on_parse(&self, _source: &str) -> Result<Option<String>> {
        Ok(None)
    }
    
    fn on_transform(&self, _code: &str) -> Result<Option<String>> {
        Ok(None)
    }
    
    fn on_bundle(&self, _bundle: &str) -> Result<Option<String>> {
        Ok(None)
    }
}
```

### 插件生命周期

1. **on_init** - 插件初始化阶段
2. **on_parse** - 源码解析阶段
3. **on_transform** - 代码变换阶段
4. **on_bundle** - 打包阶段

## 内置示例插件

### 1. LoggingPlugin

记录编译过程中的日志信息：

```rust
pub struct LoggingPlugin {
    name: String,
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_init(&self, _ctx: Arc<NargoContext>) -> Result<()> {
        tracing::info!("LoggingPlugin initialized");
        Ok(())
    }
}
```

### 2. TransformPlugin

在代码顶部添加变换注释：

```rust
pub struct TransformPlugin {
    name: String,
}

impl Plugin for TransformPlugin {
    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = format!(
            "// Transformed by TransformPlugin\n{}",
            code
        );
        Ok(Some(transformed))
    }
}
```

### 3. MinifyPlugin

简单的代码压缩插件：

```rust
pub struct MinifyPlugin {
    name: String,
}

impl Plugin for MinifyPlugin {
    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let minified = self.minify_code(code);
        Ok(Some(minified))
    }
}
```

### 4. CustomPlugin

可配置的自定义插件：

```rust
pub struct CustomPlugin {
    name: String,
    prefix: String,
}
```

## 使用插件管理器

### 注册插件

```rust
let ctx = Arc::new(NargoContext::default());
let mut manager = PluginManager::new(ctx);

manager.register(Box::new(LoggingPlugin::new()));
manager.register(Box::new(TransformPlugin::new()));
manager.register(Box::new(MinifyPlugin::new()));
```

### 初始化插件

```rust
manager.init_all()?;
```

### 应用插件链

```rust
let result = manager.transform(input_code.to_string()).await?;
```

## 开发自己的插件

### 步骤 1: 定义插件结构

```rust
pub struct MyPlugin {
    name: String,
    config: MyConfig,
}
```

### 步骤 2: 实现 Plugin Trait

```rust
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_init(&self, ctx: Arc<NargoContext>) -> Result<()> {
        println!("MyPlugin initialized");
        Ok(())
    }

    fn on_transform(&self, code: &str) -> Result<Option<String>> {
        let transformed = self.process(code);
        Ok(Some(transformed))
    }
}
```

### 步骤 3: 注册和使用

```rust
let mut manager = PluginManager::new(ctx);
manager.register(Box::new(MyPlugin::new()));
```

## 代码说明

### 插件基础使用

```rust
let ctx = Arc::new(nargo_types::NargoContext::default());
let mut manager = PluginManager::new(ctx);

let logging_plugin = LoggingPlugin::new();
manager.register(Box::new(logging_plugin));

manager.init_all()?;
```

### 插件链处理

```rust
let result = manager.transform(input_code.to_string()).await?;
```

### 多插件协作

```rust
manager.register(Box::new(LoggingPlugin::new()));
manager.register(Box::new(TransformPlugin::new()));
manager.register(Box::new(MinifyPlugin::new()));
```

## 依赖项

- `nargo-plugin` - Nargo 插件系统核心
- `nargo-types` - Nargo 类型系统
- `tokio` - 异步运行时
- `anyhow` - 错误处理
- `async-trait` - 异步 trait 支持

## 学习要点

1. 如何实现 `Plugin` trait
2. 插件生命周期的各个阶段
3. 如何使用 `PluginManager` 管理插件
4. 如何创建插件处理链
5. 如何测试插件功能
6. 插件间如何协作
