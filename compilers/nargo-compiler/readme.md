# nargo-compiler

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20code%20compiler%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Compiler Logo" width="150" height="150">
  <h3>⚡ Nargo 框架的核心编译引擎</h3>
  <p>负责将 .nargo 组件转换为高度优化的运行时代码，集成模板解析、脚本分析、样式提取和代码生成</p>
</div>

## 🎯 简介

`nargo-compiler` 是 Nargo 编译链的核心调度中心，它集成了模板解析、脚本分析、样式提取、优化和代码生成，支持多种 DSL（如 Pug, Markdown, Scss 等）的混合编译。作为 Nargo 生态系统的核心组件，它实现了 Nargo 核心的 **Reactive Transform** 和 **Atomic CSS** 自动生成，为前端开发提供高效、优化的编译体验。

## ✨ 核心特性

### Pipeline 编排
- **智能工作流**: 自动协调 `nargo-parser`, `nargo-optimizer` 和各类 Target 生成器的工作流
- **并行处理**: 支持多线程并行编译，提高编译速度
- **增量编译**: 只重新编译变更的文件，大幅提升开发体验
- **错误处理**: 提供详细的错误信息，帮助开发者快速定位问题

### 多语言支持
- **插件化架构**: 通过插件化注册机制，支持多种语言和格式
- **原生支持**: 原生支持 Pug, Markdown, Scss, Less, Tailwind, YAML, TOML 等多种格式
- **自定义插件**: 支持自定义语言插件，扩展编译能力
- **语言集成**: 支持在同一组件中混合使用多种语言

### SSR & Hydration
- **服务端渲染**: 支持一键生成服务端渲染函数
- **客户端激活**: 生成客户端激活代码，实现 hydration
- **同构应用**: 支持构建同构应用，兼顾 SEO 和用户体验
- **性能优化**: 优化 SSR 性能，减少服务器负载

### Reactive Transform
- **编译时优化**: 在编译时识别响应式变量，自动处理变量访问
- **消除运行时开销**: 消除运行时的 `.value` 开销，提升性能
- **智能转换**: 智能处理响应式变量的读取和写入
- **类型安全**: 保持类型安全，确保代码的可靠性

### 原子化样式引擎
- **深度集成**: 深度集成 Tailwind CSS 支持
- **自动提取**: 自动从组件中提取并生成最小化的 CSS
- **优化输出**: 生成高效、最小化的原子化 CSS
- **运行时无依赖**: 生成的 CSS 不依赖任何运行时库

### 高级优化
- **静态提升**: 静态提升常量和不变表达式
- **常量折叠**: 在编译时计算常量表达式
- **死代码消除**: 移除未使用的代码
- **树摇优化**: 智能移除未使用的模块

## 🚀 使用方法

### 命令行使用

```bash
# 构建项目
nargo build

# 启动开发服务器
nargo run dev

# 初始化新项目
nargo init my-project

# 运行类型检查
nargo check

# 运行测试
nargo test

# 生成类型文档
nargo docs
```

### API 使用

```rust
use nargo_compiler::{Compiler, CompilerOptions};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // 创建编译器实例
    let mut compiler = Compiler::new();

    // 配置编译选项
    let options = CompilerOptions {
        entry_point: Path::new("src/main.nargo").to_path_buf(),
        output_dir: Path::new("dist").to_path_buf(),
        enable_ssr: true,
        enable_hydration: true,
        optimization_level: 2,
        ..Default::default()
    };

    // 执行编译
    compiler.compile(&options)?;

    println!("编译成功！");
    Ok(())
}
```

## 🔧 配置选项

`nargo-compiler` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.compiler]
# 编译配置
entry_point = "src/main.nargo"
output_dir = "dist"
optimization_level = 2  # 0-3，3 为最高优化级别

# 语言支持
[tool.nargo.compiler.languages]
enable_pug = true
enable_markdown = true
enable_scss = true
enable_less = true
enable_tailwind = true

# SSR 配置
[tool.nargo.compiler.ssr]
enable = true
enable_hydration = true
ssr_output_dir = "dist/ssr"

# 样式配置
[tool.nargo.compiler.style]
enable_atomic_css = true
css_output_dir = "dist/css"
minify_css = true

# 开发服务器配置
[tool.nargo.compiler.dev]
enable_hmr = true
port = 3000
host = "localhost"
```

## 📊 与其他编译器对比

| 特性 | nargo-compiler | Vite | Rollup | Webpack |
|------|---------------|------|--------|---------|
| 启动速度 | ⚡⚡⚡ (瞬时) | ⚡⚡⚡ (瞬时) | ⚡⚡ (快速) | ⚡ (一般) |
| 构建速度 | ⚡⚡⚡ (快速) | ⚡⚡ (快速) | ⚡ (一般) | ⚡ (一般) |
| 增量编译 | ✅ (完整) | ✅ (完整) | ❌ (有限) | ❌ (有限) |
| 多语言支持 | ✅ (完整) | ✅ (通过插件) | ✅ (通过插件) | ✅ (通过插件) |
| SSR 支持 | ✅ (原生) | ✅ (通过插件) | ❌ (有限) | ❌ (有限) |
| 响应式优化 | ✅ (原生) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 原子化样式 | ✅ (原生) | ❌ (通过插件) | ❌ (通过插件) | ❌ (通过插件) |
| 类型安全 | ✅ (完整) | ✅ (通过 TypeScript) | ✅ (通过 TypeScript) | ✅ (通过 TypeScript) |

## 🎯 应用场景

### 单页应用开发
在单页应用开发中，`nargo-compiler` 提供快速的开发体验和高效的构建产物，支持多种前端框架和库。

### 服务端渲染应用
在服务端渲染应用中，`nargo-compiler` 提供一键式 SSR 支持，生成优化的服务端渲染函数和客户端激活代码。

### 静态网站生成
在静态网站生成中，`nargo-compiler` 可以生成静态 HTML 文件，支持 Markdown 等多种内容格式。

### 组件库开发
在组件库开发中，`nargo-compiler` 提供高效的组件编译和优化，支持多种样式方案和组件格式。

## 🔧 核心 API

### Compiler
编译器的核心类，负责整个编译过程的协调和执行：
- **new**: 创建新的编译器实例
- **compile**: 执行编译过程
- **compile_file**: 编译单个文件
- **compile_dir**: 编译整个目录
- **set_options**: 设置编译选项

### CompilerOptions
编译选项，控制编译过程的各个方面：
- **entry_point**: 入口文件路径
- **output_dir**: 输出目录路径
- **optimization_level**: 优化级别
- **enable_ssr**: 是否启用 SSR
- **enable_hydration**: 是否启用 hydration
- **languages**: 启用的语言支持

### Session
编译会话，管理编译过程中的状态和资源：
- **new**: 创建新的会话
- **add_file**: 添加文件到会话
- **compile**: 编译会话中的文件
- **get_output**: 获取编译输出

### IRModule
中间表示模块，存储编译过程中的中间数据：
- **new**: 创建新的中间表示模块
- **add_node**: 添加节点到模块
- **optimize**: 优化模块
- **generate_code**: 生成代码

## 🏗️ 核心逻辑

1. **Registry Init**: 初始化解析器注册表，加载所有支持的语言插件
2. **Parsing**: 调用 `nargo-parser` 将 SFC 拆分为块，并使用匹配的子解析器生成 `IRModule`
3. **Optimization**: 使用 `nargo-optimizer` 进行静态提升、常量折叠和样式提取
4. **Code Generation**: 
   - 若开启 SSR，调用 `nargo-ssr` 生成渲染函数
   - 若开启 Hydrate，调用 `nargo-hydrate` 生成激活逻辑
   - 默认调用 `nargo-bundler` 生成客户端运行时代码
5. **Asset Bundling**: 汇总生成的 JS、CSS 和 Source Map
6. **Output**: 将编译产物输出到指定目录

## 🔗 相关项目

- [nargo-parser](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-parser): 负责第一阶段的语法拆解
- [nargo-optimizer](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-optimizer): 负责中间表示的优化
- [nargo-bundler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-bundler): 负责代码打包和运行时生成
- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 基础类型定义
- [nargo-ssr](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-ssr): 服务端渲染支持
- [nargo-hydrate](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-hydrate): 客户端激活支持

## 📚 文档

- **[官方文档](https://docs.nargo.dev/compiler)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/compiler/api)** - 完整的 API 参考
- **[优化指南](https://docs.nargo.dev/compiler/optimization)** - 编译优化最佳实践指南
- **[插件开发](https://docs.nargo.dev/compiler/plugins)** - 编译器插件开发指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
