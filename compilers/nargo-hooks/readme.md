# nargo-hooks

> Nargo 框架的 Git Hooks 管理工具，守护代码提交的第一道质量防线。

## 📖 简介

`nargo-hooks` 提供了高性能、易配置的 Git Hooks 自动化管理方案。它允许开发者通过简单的配置文件（`nargo.config.json`）来定义提交前的检查逻辑（如 Lint, Test, Audit），并以毫秒级的响应速度拦截不符合规范的代码提交，确保代码库的健壮性。

## ✨ 核心特性

- **原生 Hook 安装**: 自动接管 `.git/hooks` 目录，通过轻量级的 shell 脚本将 Git 事件转发至 Nargo 原生引擎。
- **跨平台支持**: 针对 Unix 和 Windows 环境进行了优化，确保在不同操作系统下都能稳定运行。
- **配置驱动**: 无需编写复杂的 shell 脚本，通过 JSON/TOML 配置即可编排复杂的钩子流水线。
- **高性能拦截**: 直接基于 Rust 构建，相比传统的 Node.js 钩子工具（如 husky），启动速度提升了一个数量级。

## 🏗️ 核心逻辑

- **NargoHooks**: 提供 `install`, `uninstall` 和 `run` 等核心静态方法，管理钩子的生命周期。
- **Hook 脚本生成**: 自动检测当前执行路径，生成指向 Nargo 二进制文件的重定向脚本。
- **参数透传**: 支持将 Git 原生参数（如提交信息、文件列表）完整透传给下游的检查任务。

## 🔗 相关项目

- [nargo-config](file:///e:/模板引擎/nargo/compilers/nargo-config): 为钩子执行提供配置支持。
- [nargo-linter](file:///e:/模板引擎/nargo/compilers/nargo-linter): 常作为 `pre-commit` 钩子的核心检查项。
