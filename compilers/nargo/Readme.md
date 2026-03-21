# nargo

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20rust%20command%20line%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo CLI Logo" width="150" height="150">
  <h3>⚡ Nargo 框架主命令行工具</h3>
  <p>集成所有子模块功能，提供统一、高效的开发者体验</p>
</div>

## 🎯 简介

`nargo` 是 Nargo 编译工具链的核心入口点，它不仅是一个命令行工具，更是一个高度集成的开发工作站。通过封装底层的编译器、打包器、测试运行器、Linter 以及开发服务器，`nargo` 为开发者提供了一套完整、一致且极速的开发流程。无论是在单体应用还是复杂的 Monorepo 项目中，`nargo` 都能提供卓越的性能支撑。

## ✨ 核心特性

- **一站式工作流**: 集成了从项目初始化、开发调试、自动化测试、静态分析到生产构建的全生命周期指令。
- **Monorepo 原生支持**: 深度集成 `nargo-workspace`，能够自动识别工作区结构，支持跨包任务编排与混合热更新。
- **极致性能**: 核心链路均采用 Rust 编写，利用多线程并发与增量编译技术，显著提升开发反馈速度。
- **全方位的质量保障**: 内置 Linter、安全审计 (Audit)、代码格式化 (Format) 以及高精度的基准测试 (Bench)。
- **原生能力扩展**: 提供高性能的 Git 操作封装及 Git Hooks 自动化管理。
- **智能类型桥接**: 通过 `nargo bridge` 命令，实现 Rust 后端与 TypeScript 前端的类型自动同步。

## 🏗️ 核心指令详解

### 开发与调试

#### `nargo run dev`
启动带有 HMR 和 Mock 功能的高性能开发服务器：
- **混合热更新**: 同时支持 TypeScript 前端和 Rust 后端的热更新
- **智能 Mock**: 内置 Mock 服务，无需后端即可进行前端开发
- **实时类型检查**: 开发过程中实时进行类型检查，提前发现错误
- **多线程编译**: 利用多核 CPU 加速编译过程

```bash
# 启动开发服务器
nargo run dev

# 启动支持混合热更新的开发服务器
nargo run dev --hybrid

# 指定端口
nargo run dev --port 3000
```

### 构建与部署

#### `nargo build`
执行生产环境构建，生成极致优化的代码产物：
- **智能增量构建**: 只重新构建变更的文件
- **多线程并行构建**: 充分利用多核 CPU
- **自动代码分割**: 智能分析依赖关系，生成最优的代码分割策略
- **Tree Shaking**: 自动移除未使用的代码
- **生产环境优化**: 内置多种生产环境优化策略

```bash
# 构建生产版本
nargo build

# 构建特定环境
nargo build --env staging

# 启用详细输出
nargo build --verbose
```

### 测试与质量

#### `nargo test`
运行单元测试、集成测试，并支持基于插桩的代码覆盖率收集：
- **多测试类型支持**: 单元测试、集成测试、端到端测试
- **并行测试执行**: 多线程并行执行测试
- **测试覆盖率**: 自动生成测试覆盖率报告
- **智能测试选择**: 支持按模式选择测试

```bash
# 运行所有测试
nargo test

# 运行特定测试
nargo test --pattern "user*"

# 生成测试覆盖率报告
nargo test --coverage
```

#### `nargo lint`
对项目进行全方位的静态代码检查，识别潜在风险：
- **多语言支持**: 支持 TypeScript、Rust、CSS 等多种语言
- **实时检查**: 开发过程中实时进行代码检查
- **可配置规则**: 支持自定义检查规则
- **自动修复**: 支持自动修复某些类型的问题

```bash
# 运行代码检查
nargo lint

# 自动修复问题
nargo lint --fix

# 检查特定目录
nargo lint src/components
```

#### `nargo format`
自动化格式化 Nargo 组件（Template, Script, Style）：
- **统一代码风格**: 确保项目代码风格一致
- **多语言支持**: 支持多种语言的格式化
- **可配置规则**: 支持自定义格式化规则
- **批量处理**: 支持批量格式化多个文件

```bash
# 格式化所有文件
nargo format

# 检查格式但不修改
nargo format --check

# 格式化特定文件
nargo format src/components/Button.tsx
```

### 性能与安全

#### `nargo bench`
执行高精度的性能基准测试，辅助开发者进行性能调优：
- **高精度测量**: 提供精确的性能测量数据
- **趋势分析**: 跟踪性能变化趋势
- **多维度测试**: 支持多种性能指标测试
- **比较模式**: 支持与历史版本比较

```bash
# 运行基准测试
nargo bench

# 与历史版本比较
nargo bench --compare

# 测试特定功能
nargo bench --pattern "render*"
```

#### `nargo audit`
进行项目安全扫描，识别硬编码凭据与危险依赖：
- **依赖扫描**: 检查依赖项的安全漏洞
- **代码审计**: 识别硬编码的凭据和敏感信息
- **配置检查**: 检查项目配置中的安全问题
- **报告生成**: 生成详细的安全审计报告

```bash
# 运行安全审计
nargo audit

# 生成详细报告
nargo audit --report

# 忽略特定漏洞
nargo audit --ignore CVE-2023-1234
```

### Monorepo 管理

#### `nargo run`
在 Monorepo 环境下跨包运行任务，支持复杂的依赖关系处理：
- **智能依赖分析**: 自动分析包之间的依赖关系
- **并行任务执行**: 多线程并行执行任务
- **任务编排**: 支持复杂的任务编排
- **环境一致性**: 确保所有包使用一致的环境

```bash
# 运行特定包的任务
nargo run build --package my-package

# 并行运行多个任务
nargo run build --parallel

# 运行工作区所有包的任务
nargo run test --workspace
```

### 类型桥接

#### `nargo bridge`
同步 Rust 后端与 TypeScript 前端的类型定义：
- **实时同步**: 自动同步 Rust 结构体到 TypeScript 类型
- **类型安全**: 确保前后端类型一致
- **减少手动工作**: 无需手动编写类型定义
- **支持复杂类型**: 支持复杂的嵌套类型和泛型

```bash
# 同步类型
nargo bridge

# 生成类型文档
nargo bridge --docs

# 自定义输出路径
nargo bridge --output src/types/rust.d.ts
```

## 📊 与其他工具对比

| 特性 | Nargo CLI | npm/yarn | pnpm | Cargo |
|------|-----------|----------|------|-------|
| 启动速度 | ⚡⚡⚡ (瞬时) | ⚡ (一般) | ⚡⚡ (快速) | ⚡⚡ (快速) |
| 内存占用 | 📦 (轻量) | 📦📦📦 (heavy) | 📦📦 (中等) | 📦📦 (中等) |
| 内置测试 | ✅ (完整) | ❌ (需配置) | ❌ (需配置) | ✅ (完整) |
| 内置 lint | ✅ (完整) | ❌ (需插件) | ❌ (需插件) | ❌ (需插件) |
| 内置格式化 | ✅ (完整) | ❌ (需插件) | ❌ (需插件) | ❌ (需插件) |
| Monorepo 支持 | ✅ (原生) | ❌ (需工具) | ✅ (原生) | ❌ (需工具) |
| 类型桥接 | ✅ (原生) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |

## 🔧 配置选项

`nargo` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo]
# 开发服务器配置
[tool.nargo.dev]
port = 3000
hmr = true

# 构建配置
[tool.nargo.build]
env = "production"
optimization = "aggressive"

# 测试配置
[tool.nargo.test]
coverage = true
parallel = true

# Linter 配置
[tool.nargo.lint]
enable = true
rules = ["no-unused-vars", "no-console"]
```

## 🚀 快速开始

### 安装

```bash
# 从源码构建
cargo install --path compilers/nargo

# 或使用预编译二进制文件
curl -fsSL https://nargo.dev/install.sh | bash
```

### 初始化项目

```bash
# 初始化新项目
nargo init my-project

# 初始化 Monorepo 项目
nargo init my-monorepo --workspace

# 使用特定模板
nargo init my-project --template react
```

## 🔗 相关项目

- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): 提供核心转译能力
- [nargo-server](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-server): 驱动开发环境的 Web 服务
- [nargo-workspace](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-workspace): 处理多包工作区的管理与调度
- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 定义了跨模块共享的通用数据模型
- [nargo-bundler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-bundler): 提供高性能的代码打包能力
- [nargo-linter](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-linter): 提供代码静态分析能力

## 📚 文档

- **[官方文档](https://docs.nargo.dev)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/api)** - 完整的 API 参考
- **[贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md)** - 如何为 Nargo 项目做贡献

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
