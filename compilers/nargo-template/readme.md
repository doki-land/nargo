# nargo-template

Nargo 模板引擎管理库，提供统一的模板引擎抽象层和 SSG 构建工具。

## 功能特性

- **统一模板引擎抽象层**：支持多种模板引擎（HTML、Jinja2、Liquid、DejaVu）
- **基于 ECS 的数据处理系统**：提供灵活的实体组件系统用于数据处理
- **统一配置管理接口**：支持 JSON、TOML、YAML 格式的配置文件
- **统一文件系统操作抽象**：提供一致的文件系统操作接口
- **统一插件系统接口**：支持插件的加载、注册和执行
- **统一构建流程管理**：提供完整的构建流程管理和执行机制
- **SSG 引擎集成**：支持与现有 SSG 引擎的集成

## 安装

在 `Cargo.toml` 文件中添加以下依赖：

```toml
[dependencies]
nargo-template = { path = "path/to/nargo-template" }
```

## 基本使用

### 1. 模板引擎使用

```rust
use nargo_template::{TemplateEngine, TemplateContext};

// 创建模板引擎
let mut engine = TemplateEngine::new();

// 创建模板上下文
let mut context = TemplateContext::new();
context.set("name", "World");

// 渲染模板
let template = "Hello, {{ name }}!";
let result = engine.render(template, &context).unwrap();
println!("{}", result); // 输出: Hello, World!
```

### 2. 配置管理

```rust
use nargo_template::{ConfigManager, JsonConfigLoader};

// 创建配置管理器
let mut manager = ConfigManager::new();

// 添加 JSON 配置加载器
manager.add_loader(Box::new(JsonConfigLoader::new()));

// 加载配置
let config = manager.load("config.json").unwrap();

// 获取配置值
let site_name = config.get("site_name").unwrap();
```

### 3. 文件系统操作

```rust
use nargo_template::{FileSystemManager, LocalFileSystem};

// 创建文件系统管理器
let mut fs_manager = FileSystemManager::with_local();

// 读取文件
let content = fs_manager.fs().read_file("src/index.md").unwrap();

// 写入文件
fs_manager.fs().write_file("dist/index.html", "<html><body>Hello World</body></html>").unwrap();
```

### 4. 插件系统

```rust
use nargo_template::{PluginManager, ExamplePlugin, PluginContext, PluginStage};

// 创建插件管理器
let mut plugin_manager = PluginManager::new();

// 注册插件
plugin_manager.register_plugin(Box::new(ExamplePlugin::new()));

// 创建插件上下文
let context = PluginContext::new(config, fs);

// 初始化插件
plugin_manager.init_all(&mut context).await.unwrap();

// 执行插件阶段
plugin_manager.execute_stage(PluginStage::PreBuild, &mut context).await.unwrap();
```

### 5. 构建流程管理

```rust
use nargo_template::{BuildManager, BuildContext, ExampleBuildHandler, BuildStage};

// 创建构建上下文
let context = BuildContext::new(config, fs);

// 创建构建管理器
let mut build_manager = BuildManager::new();

// 添加构建阶段处理器
build_manager.add_handler(Box::new(ExampleBuildHandler::new()));

// 设置构建上下文
build_manager.set_context(context);

// 执行构建
build_manager.build().await.unwrap();
```

### 6. SSG 引擎集成

```rust
use nargo_template::{SsgIntegrationManager, ExampleSsgAdapter, BuildContext};

// 创建 SSG 集成管理器
let mut integration_manager = SsgIntegrationManager::new();

// 注册 SSG 适配器
integration_manager.register_adapter("example", Box::new(ExampleSsgAdapter::new()));

// 设置活动适配器
integration_manager.set_active_adapter("example").unwrap();

// 创建构建上下文
let context = BuildContext::new(config, fs);

// 初始化适配器
integration_manager.init(&mut context).await.unwrap();

// 构建站点
integration_manager.build(&mut context).await.unwrap();
```

## 模块结构

- **build**：构建流程管理
- **config**：配置管理
- **context**：模板上下文
- **ecs**：基于 ECS 的数据处理系统
- **engine**：模板引擎
- **fs**：文件系统操作抽象
- **integration**：SSG 引擎集成
- **manager**：模板管理器
- **plugins**：插件系统
- **renderers**：模板渲染器
- **template_engine**：统一模板引擎抽象层

## 贡献

欢迎贡献代码、报告问题或提出建议。请在 GitHub 上创建 issue 或提交 pull request。

## 许可证

MIT
