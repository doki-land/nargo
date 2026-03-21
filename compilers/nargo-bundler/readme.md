# nargo-bundler

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20code%20bundler%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Bundler Logo" width="150" height="150">
  <h3>📦 Nargo 框架的打包与运行时优化工具</h3>
  <p>实现 "按需打包" 和 "极致体积" 愿景的关键组件，为前端应用提供高效的打包解决方案</p>
</div>

## 🎯 简介

`nargo-bundler` 是 Nargo 编译工具链中的核心打包组件，负责将多个模块聚合在一起，并根据实际使用的功能特性生成定制化的、极简的运行时代码。它是实现 Nargo "按需打包" 和 "极致体积" 愿景的关键，通过智能分析和优化，确保最终的打包产物既小又高效。

## ✨ 核心特性

### 智能特性分析
- **深度扫描**: 深度扫描组件树，识别是否使用了 Signals、Effects、VDOM 或 SSR 等特性
- **精确依赖分析**: 准确分析模块间的依赖关系，避免冗余打包
- **特性集识别**: 自动识别项目使用的特性集，为定制化打包提供依据
- **增量分析**: 支持增量分析，只重新分析变更的模块

### 定制化运行时生成
- **按需打包**: 根据分析结果，仅打包必要的运行时代码
- **死代码消除**: 彻底消除死代码（Dead Code），减小打包体积
- **运行时优化**: 针对不同特性集生成优化的运行时代码
- **多目标支持**: 支持生成不同环境的运行时代码（浏览器、Node.js、SSR 等）

### 高效模块处理
- **组件聚合**: 支持将多个模块合并为单一的可分发文件
- **依赖优化**: 智能处理组件间的循环依赖和公共模块提取
- **模块内联**: 将多个组件的 IR 及其依赖项内联到最终的输出文件中
- **代码分割**: 支持智能代码分割，优化加载性能

### 高级优化
- **Tree Shaking**: 智能移除未使用的代码
- **作用域提升**: 优化变量作用域，减少运行时开销
- **常量折叠**: 在编译时计算常量表达式
- **代码压缩**: 内置代码压缩功能，减小最终产物体积

## 🏗️ 核心逻辑

### FeatureSet
核心数据结构，记录了组件树对响应式、VDOM、SSR 等特性的依赖情况：
- **响应式特性**: 记录是否使用了 Signals、Effects 等响应式特性
- **渲染特性**: 记录是否使用了 VDOM、SSR 等渲染特性
- **运行时特性**: 记录是否需要特定的运行时支持
- **依赖关系**: 记录模块间的依赖关系

### Bundler
执行分析与打包的主体类，负责遍历 `IRModule` 并生成定制化的 JS 运行时：
- **模块分析**: 分析模块的特性使用情况
- **依赖解析**: 解析模块间的依赖关系
- **运行时生成**: 根据特性分析结果生成定制化的运行时代码
- **代码优化**: 应用各种代码优化策略

### Module Inlining
将多个组件的 IR 及其依赖项内联到最终的输出文件中：
- **内联策略**: 智能决定哪些模块需要内联
- **依赖处理**: 处理内联模块的依赖关系
- **代码生成**: 生成内联后的代码
- **优化输出**: 确保内联后的代码高效运行

## 🚀 使用方法

### 命令行使用

```bash
# 打包项目
nargo build

# 打包特定模块
nargo build --module src/main.ts

# 启用详细输出
nargo build --verbose

# 指定输出目录
nargo build --output dist

# 启用代码分割
nargo build --split-chunks
```

### API 使用

```rust
use nargo_bundler::{Bundler, BundlerOptions};

// 创建打包器实例
let mut bundler = Bundler::new();

// 配置打包选项
let options = BundlerOptions {
    output_path: "dist/bundle.js".into(),
    enable_tree_shaking: true,
    enable_code_splitting: true,
    optimization_level: 2,
    ..Default::default()
};

// 执行打包
bundler.build("src/main.ts", &options).unwrap();
```

## 🔧 配置选项

`nargo-bundler` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.bundler]
# 输出配置
output_path = "dist/bundle.js"
public_path = "/"

# 优化配置
optimization_level = 2  # 0-3，3 为最高优化级别
enable_tree_shaking = true
enable_code_splitting = true
enable_scope_hoisting = true
enable_constant_folding = true

# 代码分割配置
[tool.nargo.bundler.code_splitting]
split_chunks = true
max_chunks = 10
min_chunk_size = 3000

# 目标环境配置
[tool.nargo.bundler.target]
environment = "browser"  # browser, node, ssr
minify = true
source_map = true
```

## 📊 与其他打包工具对比

| 特性 | nargo-bundler | Webpack | Rollup | Vite |
|------|---------------|---------|--------|------|
| 启动速度 | ⚡⚡⚡ (瞬时) | ⚡ (一般) | ⚡⚡ (快速) | ⚡⚡⚡ (瞬时) |
| 构建速度 | ⚡⚡⚡ (快速) | ⚡ (一般) | ⚡⚡ (快速) | ⚡⚡ (快速) |
| 打包体积 | 📦 (极小) | 📦📦📦 (较大) | 📦📦 (中等) | 📦📦 (中等) |
| 按需打包 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 特性分析 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 定制化运行时 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 代码分割 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| Tree Shaking | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |

## 🎯 应用场景

### 前端应用打包
在前端应用开发中，`nargo-bundler` 提供高效的打包解决方案，确保应用体积小、加载快。

### 库开发
在库开发中，`nargo-bundler` 可以生成最小化的打包产物，减少库的体积，提高使用体验。

### SSR 应用
在 SSR 应用中，`nargo-bundler` 可以生成针对服务器环境优化的运行时代码，提高 SSR 性能。

### 微前端应用
在微前端应用中，`nargo-bundler` 可以生成独立的、最小化的微应用包，提高微前端的加载性能。

## 🔧 核心 API

### 基本用法

```rust
use nargo_bundler::{Bundler, BundlerOptions};

// 创建打包器实例
let mut bundler = Bundler::new();

// 配置打包选项
let options = BundlerOptions {
    output_path: "dist/bundle.js".into(),
    enable_tree_shaking: true,
    enable_code_splitting: false,
    optimization_level: 2,
    ..Default::default()
};

// 执行打包
let result = bundler.build("src/main.ts", &options);

match result {
    Ok(output) => println!("打包成功，输出到: {}", output),
    Err(error) => println!("打包失败: {}", error),
}
```

### 高级用法

```rust
use nargo_bundler::{Bundler, BundlerOptions, FeatureSet};

// 创建打包器实例
let mut bundler = Bundler::new();

// 自定义特性集
let feature_set = FeatureSet {
    signals: true,
    effects: true,
    vdom: true,
    ssr: false,
    ..Default::default()
};

// 配置打包选项
let options = BundlerOptions {
    output_path: "dist/bundle.js".into(),
    enable_tree_shaking: true,
    enable_code_splitting: true,
    optimization_level: 3,
    custom_feature_set: Some(feature_set),
    ..Default::default()
};

// 执行打包
bundler.build("src/main.ts", &options).unwrap();
```

## 🔗 相关项目

- [nargo-ir](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-ir): 分析的基础数据结构
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): 在构建链路中集成此工具
- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 提供类型定义和工具函数
- [nargo-server](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-server): 开发服务器，集成打包功能

## 📚 文档

- **[官方文档](https://docs.nargo.dev/bundler)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/bundler/api)** - 完整的 API 参考
- **[优化指南](https://docs.nargo.dev/bundler/optimization)** - 打包优化最佳实践指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
