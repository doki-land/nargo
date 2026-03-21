# nargo-server

> Nargo 高性能开发与预览服务器，基于异步 I/O 构建的极速开发环境。

## 📖 简介

`nargo-server` 是 Nargo 框架的专用 Web 服务器组件。它不仅为开发者提供了带有即时编译（JIT）和文件监听功能的开发环境，还支持对生产产物进行高性能的静态资源预览。

## ✨ 核心特性

- **双模式切换**:
    - **Dev 模式**: 集成文件系统监听（`notify`），自动触发重编译，支持热更新。
    - **Serve 模式**: 高性能静态文件服务器，用于生产环境模拟与预览。
- **异步驱动**: 基于 `wae-server` 和 `tokio` 构建，具备极高的并发处理能力。
- **集成监听机制**: 内置 `watcher` 线程，实时感知项目根目录下的文件变更。
- **SPA 回退支持**: 内置 `fallback` 机制，完美支持单页应用（SPA）的路由跳转。

## 🏗️ 核心数据结构

- **ServerOptions**: 服务器启动参数，包括运行模式（Dev/Serve）、监听地址及端口。
- **ServerState**: 全局共享状态，管理编译器实例、配置上下文及运行模式。
- **ServerMode**: 明确区分开发环境与预览环境的行为差异。

## 🔗 相关项目

- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 为开发模式提供即时编译能力支撑。
- [nargo-config](file:///e:/模板引擎/nargo/compilers/nargo-config): 提供服务器所需的网络配置与根目录信息。
