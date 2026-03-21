# nargo-mono

> Nargo 框架的 Monorepo 管理引擎，混合开发模式下的高效编排专家。

## 📖 简介

`nargo-mono` 专为管理大型复杂项目而设计。它能够自动识别 Cargo 工作区（Workspace）结构，并提供针对 Rust 后端与 TS 前端的混合热更新（Hybrid HMR）能力，通过智能的任务编排与并行执行，显著提升了多包项目的开发体验。

## ✨ 核心特性

- **工作区自动发现**: 自动解析根目录 `Cargo.toml`，智能识别所有的 workspace 成员及其层级关系。
- **混合热更新 (Hybrid HMR)**: 同时监听 Rust 源码与 TS 源码的变更，并根据不同技术栈自动触发重编译或前端更新。
- **任务并行执行**: 利用 `rayon` 和 `tokio` 的并发能力，实现多包构建与检查任务的高效调度。
- **高性能监听**: 基于 `notify` 机制实现低开销的文件系统监控，确保在大规模代码库下依然反应灵敏。

## 🏗️ 核心逻辑

- **NargoWorkspace**: 工作区模型，封装了成员发现、列表展示及混合开发模式的启动逻辑。
- **Hybrid HMR Pipeline**: 编排 Rust 后端服务与前端编译器的生命周期，实现联动更新。
- **Cargo Integration**: 深度集成 Cargo 指令，自动化处理跨语言的构建依赖。

## 🔗 相关项目

- [nargo-config](file:///e:/模板引擎/nargo/compilers/nargo-config): 为工作区管理提供全局配置参数。
- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 作为 Monorepo 中各子项目的核心编译引擎。
