# 常见问题 (FAQ)

## 目录

- [关于 Nargo](#关于-nargo)
- [安装与配置](#安装与配置)
- [使用问题](#使用问题)
- [技术细节](#技术细节)
- [故障排除](#故障排除)

## 关于 Nargo

### Q: Nargo 是什么？

A: Nargo 是一个基于 Rust 全栈哲学的下一代前端调试与构建器。它集成了编译器、运行时、Git 管理等多种功能于一体，提供极致快、极致轻、零配置的开发体验。

### Q: Nargo 与 Vite/Rolldown 有什么区别？

A: Nargo 不仅仅是一个构建工具，它是对整个前端工程化链路的重新思考：
- **CaaS (Compiler as a Service)**: 深度集成 `rustc_interface`，瞬时冷启动和智能热更新
- **RaaS (Runtime as a Service)**: 内置高性能运行时，无感环境体验
- **All in One Binary**: 原生 Git、Hooks、Monorepo 支持，无需外部依赖

### Q: Nargo 支持哪些框架？

A: Nargo 目前主要支持 Vue 生态，同时也在积极扩展对其他框架的支持。请查看我们的兼容性文档了解更多详情。

### Q: Nargo 是免费的吗？

A: 是的，Nargo 是完全开源的，采用 MIT 许可证。您可以自由使用、修改和分发。

## 安装与配置

### Q: 如何安装 Nargo？

A: 您可以通过以下方式安装 Nargo：

```bash
# 使用 Cargo 安装（推荐）
cargo install nargo

# 或从 GitHub Releases 下载预编译二进制文件
```

### Q: 安装 Nargo 需要什么前置条件？

A: 需要：
- Rust 1.70 或更高版本（如果从源码编译）
- Git（用于版本控制功能）

### Q: 如何卸载 Nargo？

A: 如果通过 Cargo 安装：

```bash
cargo uninstall nargo
```

如果是手动安装，直接删除二进制文件即可。

### Q: 如何配置 Nargo？

A: Nargo 使用 `Nargo.toml` 作为配置文件。您可以在项目根目录创建该文件进行配置。详细配置选项请参考文档。

## 使用问题

### Q: 如何创建一个新的 Nargo 项目？

A: 使用 `init` 命令：

```bash
nargo init my-project
cd my-project
```

### Q: 如何启动开发服务器？

A: 使用 `dev` 命令：

```bash
# 普通模式
nargo dev

# 混合模式（Rust + TS 热更新）
nargo dev --hybrid
```

### Q: 如何构建生产版本？

A: 使用 `build` 命令：

```bash
nargo build
```

### Q: 如何运行测试？

A: 使用 `test` 命令：

```bash
nargo test
```

### Q: 如何同步前后端类型？

A: 使用 `bridge` 命令：

```bash
nargo bridge
```

这将自动将 Rust 后端的结构体转换为 TypeScript 定义。

### Q: 如何使用原生 Git 功能？

A: Nargo 提供了原生 Git 命令：

```bash
nargo git status
nargo git add .
nargo git commit -m "message"
```

这些命令比原生 Git 更快，因为它们直接操作 `.git` 目录而不调用外部命令。

### Q: 如何安装 Git Hooks？

A: 使用 `hooks` 命令：

```bash
nargo hooks install
```

这将安装 Nargo 原生的 Git Hooks，替代 Husky。

### Q: 如何格式化代码？

A: 使用 `format` 命令：

```bash
# 格式化单个文件
nargo format ./src/App.nargo

# 格式化整个目录
nargo format ./src
```

### Q: 如何检查代码质量？

A: 使用 `lint` 命令：

```bash
# 检查单个文件
nargo lint ./src/App.nargo

# 检查整个目录
nargo lint ./src

# 自动修复问题
nargo lint --fix ./src
```

### Q: 如何检查类型错误？

A: 使用 `typecheck` 命令：

```bash
# 类型检查
nargo typecheck

# 严格模式类型检查
nargo typecheck --strict
```

## 技术细节

### Q: Nargo 为什么这么快？

A: Nargo 的性能优势来自多个方面：
- **Rust 实现**: 所有核心功能都用 Rust 编写，内存安全且性能卓越
- **增量编译**: 利用 `rustc_interface` 实现智能增量编译
- **常驻内存 Session**: 编译器 Session 常驻内存，避免重复启动开销
- **原生实现**: Git、Hooks 等功能都是原生实现，无需外部进程调用

### Q: 什么是 Runtime as a Service？

A: Runtime as a Service (RaaS) 是 Nargo 提出的新概念：
- 开发者无需关心 Node.js、Bun 或 Deno
- 内置高性能运行时服务
- 提供即时的类型桥接与 Mock 能力
- 运行时能力按需动态注入

### Q: Nargo 如何处理依赖管理？

A: Nargo 使用类似 Cargo 的依赖管理方式：
- 使用 `Nargo.toml` 声明依赖
- 使用 `Nargo.lock` 锁定版本
- 依赖存储在全局缓存中，避免重复下载
- 严格的版本解析，避免依赖冲突

### Q: 可以在 Nargo 中使用 npm 包吗？

A: 是的，Nargo 支持与 npm 生态系统的互操作。您可以通过配置来使用 npm 包。

## 故障排除

### Q: 安装失败怎么办？

A: 请检查：
1. Rust 版本是否为 1.70 或更高
2. 网络连接是否正常
3. 是否有足够的磁盘空间

如果问题持续，请在 GitHub Issues 中报告。

### Q: 构建时遇到错误怎么办？

A: 尝试以下步骤：
1. 清理构建缓存：`nargo clean`
2. 更新依赖：`nargo update`
3. 检查 `Nargo.toml` 配置是否正确
4. 查看错误日志获取详细信息

### Q: 热更新不工作怎么办？

A: 请检查：
1. 是否使用了 `--hybrid` 标志（如果需要 Rust 热更新）
2. 文件监听是否正常工作
3. 是否有防火墙阻止了 WebSocket 连接

### Q: 性能不如预期怎么办？

A: 可以尝试：
1. 使用发布模式构建：`cargo build --release`
2. 检查是否启用了增量编译
3. 查看系统资源使用情况
4. 在 GitHub Issues 中报告性能问题

### Q: 如何获取更多帮助？

A: 您可以：
1. 查看 [文档](documentation/)
2. 在 [Discussions](https://github.com/nargo-js/nargo/discussions) 提问
3. 在 [Issues](https://github.com/nargo-js/nargo/issues) 报告 Bug
4. 加入我们的社区交流（详见 [COMMUNITY.md](COMMUNITY.md)）

### Q: 如何报告 Bug？

A: 请参考 [CONTRIBUTING.md](CONTRIBUTING.md) 中的 Bug 报告指南，提供清晰的复现步骤和环境信息。
