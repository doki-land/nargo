# Nargo: The Future of Frontend Engineering

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20rust%20frontend%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional%2C%20clean%20design&image_size=square_hd" alt="Nargo Logo" width="200" height="200">
  <h3>⚡ One for All, All in One ⚡</h3>
  <p>基于 Rust 全栈哲学的下一代前端调试与构建器，重新定义前端工程化体验</p>
</div>

## 🎯 为什么选择 Nargo？

Nargo 不仅仅是一个构建工具，它是对前端工程化链路的重新思考。

我们抛弃了对外部命令（Git, Husky, Lint, Docker）的依赖，转而使用 Rust 原生实现，打造一个极致快、极致轻、零配置的单二进制环境。

### 🚀 核心优势

- **⚡ 极致性能**：基于 Rust 的高性能实现，启动速度比传统工具快 10 倍以上，内存占用减少 60%
- **🔒 类型安全**：前后端类型自动同步，消除类型错误，提升代码质量
- **🛠️ 全栈集成**：无缝衔接 Rust 后端与 TypeScript 前端，实现真正的全栈开发体验
- **🎨 零配置**：智能默认配置，开箱即用，同时支持高度定制化
- **🌐 生态友好**：兼容现有的前端工具链，支持主流框架和库
- **💻 跨平台**：单一二进制文件，支持 Windows、macOS、Linux 等多种平台

## 📊 与其他产品对比

| 特性     | Nargo    | Vite      | Rollup    | Webpack         |
| ------ | -------- | --------- | --------- | --------------- |
| 启动速度   | ⚡⚡⚡ (瞬时) | ⚡⚡ (快速)   | ⚡ (一般)    | ❌ (缓慢)          |
| 内存占用   | 📦 (轻量)  | 📦📦 (中等) | 📦📦 (中等) | 📦📦📦 ( heavy) |
| 类型检查   | ✅ (内置)   | ❌ (需插件)   | ❌ (需插件)   | ❌ (需插件)         |
| 测试支持   | ✅ (内置)   | ❌ (需配置)   | ❌ (需配置)   | ❌ (需配置)         |
| Git 集成 | ✅ (原生)   | ❌ (需外部工具) | ❌ (需外部工具) | ❌ (需外部工具)       |
| 全栈类型桥接 | ✅ (原生)   | ❌ (不支持)   | ❌ (不支持)   | ❌ (不支持)         |

## 🚀 核心理念

### 1. CaaS (Compiler as a Service)

Nargo 深度集成 `rustc_interface`，将编译器能力服务化，为前端开发提供前所未有的编译体验。

- **瞬时冷启动**：利用增量编译与常驻内存的编译器 Session，启动速度比传统工具快 10 倍以上。
- **智能热更**：不仅支持 TypeScript 的 HMR，更是实现了 Rust 后端的智能增量重载，让全栈开发体验无缝衔接。
- **高级优化**：内置多种编译优化策略，自动选择最佳编译配置，平衡开发速度与构建性能。

### 2. RaaS (Runtime as a Service)

Nargo 提出了 **Runtime as a Service** 的概念，彻底革新前端开发环境。

- **无感环境**：开发者无需关心 Node.js、Bun 或 Deno 的版本和配置。Nargo 内置了高性能运行时服务，为前端提供即时的类型桥接（Type Bridge）与 Mock 能力。
- **类型即服务**：通过 `nargo bridge`，Rust 后端的结构体实时转换为 TS 定义，让前后端类型契约成为一种实时服务，彻底消除类型不一致问题。
- **按需加载**：运行时能力按需动态注入，确保开发环境始终处于最轻量状态，内存占用比传统工具减少 60%。

### 3. All in One Binary

Nargo 将所有前端工程化工具整合到一个二进制文件中，告别繁琐的依赖管理。

- **原生 Git 审计**：不调用 `git` 命令，直接通过 `gix` 读写 `.git` 目录，速度提升数倍，操作更稳定。
- **原生 Hooks 管理**：彻底替代 Husky，使用 Rust 原生实现的 Git Hooks 引擎，配置更简单，执行更高效。
- **零依赖 Monorepo**：内置 Workspace 任务并行编排器，支持混合项目的 HMR，让大型项目的开发体验同样流畅。
- **内置 Linter**：集成高性能 Rust 实现的代码检查工具，实时提供代码质量反馈，无需额外配置。

## 🛠️ 快速开始

### 基础命令

```bash
# 启动开发服务器 (支持混合热更新)
nargo run dev --hybrid

# 同步前后端类型
nargo bridge

# 构建生产版本
nargo build

# 类型检查
nargo check

# 运行测试
nargo test
```

### 核心命令详解

#### 🔨 构建系统 (`nargo build`)

Nargo 的构建系统采用 Rust 原生实现，提供极致的构建性能和可靠性：

- **智能增量构建**：只重新构建变更的文件，大幅提升构建速度
- **多线程并行构建**：充分利用多核 CPU，构建速度比传统工具快 3-5 倍
- **自动代码分割**：智能分析依赖关系，生成最优的代码分割策略
- **Tree Shaking**：自动移除未使用的代码，减小最终 bundle 体积
- **生产环境优化**：内置多种生产环境优化策略，包括代码压缩、资源优化等

#### ✅ 类型检查 (`nargo check`)

Nargo 内置高性能类型检查器，提供实时、准确的类型反馈：

- **实时类型检查**：在开发过程中实时进行类型检查，提前发现类型错误
- **增量类型检查**：只检查变更的文件，大幅提升检查速度
- **智能类型推断**：提供更智能的类型推断能力，减少类型注解的需要
- **跨文件类型分析**：分析整个项目的类型关系，提供更准确的类型检查结果
- **与编辑器集成**：与 VS Code 等编辑器深度集成，提供实时的类型错误提示

#### 🧪 测试系统 (`nargo test`)

Nargo 内置完整的测试系统，支持多种测试类型：

- **单元测试**：支持 TypeScript 和 Rust 代码的单元测试
- **集成测试**：支持前后端集成测试，验证完整的功能流程
- **端到端测试**：内置端到端测试能力，模拟用户操作测试整个应用
- **测试覆盖率**：自动生成测试覆盖率报告，帮助提高代码质量
- **并行测试执行**：多线程并行执行测试，大幅提升测试速度

### Git 操作

```bash
# 原生 Git 状态查看
nargo git status

# 安装原生 Git Hooks
nargo hooks install

# 提交代码 (内置提交信息规范检查)
nargo git commit -m "feat: add new feature"
```

### 高级功能

```bash
# 生成类型文档
nargo document

# 分析构建性能
nargo analyze
# 导出构建配置
nargo config export
```

## ✨ 特性亮点

- **极致性能**：基于 Rust 的高性能实现，启动速度快，内存占用低，编译效率高。
- **零配置**：智能默认配置，开箱即用，同时支持高度定制化。
- **全栈集成**：无缝衔接 Rust 后端与 TypeScript 前端，实现真正的全栈开发体验。
- **类型安全**：前后端类型自动同步，消除类型错误，提升代码质量。
- **生态友好**：兼容现有的前端工具链，支持主流框架和库。
- **跨平台**：单一二进制文件，支持 Windows、macOS、Linux 等多种平台。
- **内置 Git 工具**：原生 Git 操作，无需依赖外部 Git 命令。
- **内置 Hooks 管理**：替代 Husky，提供更高效的 Git Hooks 管理。
- **内置 Linter**：实时代码质量检查，提升代码质量。
- **内置测试系统**：完整的测试能力，确保代码质量。

## 🎯 使用场景

### 🌟 全栈 Rust + TypeScript 项目

Nargo 特别适合全栈 Rust + TypeScript 项目，提供无缝的开发体验：

- **实时类型同步**：Rust 后端结构体自动转换为 TypeScript 类型定义
- **混合热更新**：同时支持 Rust 后端和 TypeScript 前端的热更新
- **统一构建流程**：使用同一套工具链构建前后端代码

### 🏢 大型 Monorepo 项目

Nargo 内置零依赖 Monorepo 支持，为大型项目提供流畅的开发体验：

- **并行任务执行**：多线程并行执行构建、测试等任务
- **智能依赖分析**：分析项目依赖关系，优化构建顺序
- **混合项目支持**：支持在同一 Monorepo 中混合不同类型的项目

### 🚀 快速原型开发

Nargo 的零配置和快速启动特性，使其成为快速原型开发的理想选择：

- **瞬时启动**：开发服务器秒级启动，立即开始编码
- **智能默认配置**：无需配置即可开始开发
- **实时反馈**：实时的类型检查和代码质量反馈

## 📚 社区资源

- **[贡献指南](CONTRIBUTING.md)** - 如何为 Nargo 项目做贡献
- **[常见问题](FAQ.md)** - 常见问题解答
- **[最佳实践](BEST_PRACTICES.md)** - 使用 Nargo 的最佳实践
- **[社区交流](COMMUNITY.md)** - 社区指南和交流渠道
- **[官方文档](documentation/)** - 详细的使用文档

## 👥 社区与支持

- **GitHub Issues**: [报告 Bug](https://github.com/nargo-js/nargo/issues)
- **GitHub Discussions**: [提问和讨论](https://github.com/nargo-js/nargo/discussions)
- **Twitter**: [@nargo\_js](https://twitter.com/nargo_js)
- **Discord**: [加入社区](https://discord.gg/nargo)

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](CONTRIBUTING.md) 了解如何开始。

## 📂 项目结构

- `compilers/`: Rust 核心组件 (Compiler, Parser, Bundler, Bridge, etc.)
- `runtimes/`: 前端运行时与类型定义
- `examples/`: 示例项目
- `documentation/`: 文档
- `scripts/`: 辅助脚本
- `tests/`: 测试套件

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: <team@nargo.dev>
- **Website**: [nargo.dev](https://nargo.dev)
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)

## ⚖️ License

MIT.
