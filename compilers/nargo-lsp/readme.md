# nargo-lsp

> HXO 框架的语言服务器协议 (LSP) 实现。

## 📖 简介

`nargo-lsp` 为编辑器提供 HXO 语言的深度支持。通过实现 LSP 协议，它使得各类编辑器（如 VS Code, NeoVim）能够提供语法高亮、自动补全、实时诊断、跳转定义等高级功能。

## ✨ 核心特性

- **实时诊断**: 在编写代码时即时显示语法错误和潜在问题。
- **智能补全**: 针对模板标签、指令（`@`, `:`）、脚本变量提供精准补全，支持配置 `auto_imports`。
- **跳转定义**: 支持从模板标签跳转到组件定义，从变量引用跳转到声明。
- **文档悬浮 (Hover)**: 悬浮显示标签属性、函数签名等详细信息。

## 🏗️ 核心逻辑

- **Backend**: 基于 `oak-lsp` 的服务端实现，管理文档状态与工作区配置。
- **多语言解析**: 协同调用 `nargo-parser-template` 和 `nargo-parser-expression` 实时解析文档内容。
- **配置感知**: 自动读取 `nargo.config.toml`，支持 `auto_imports` 等编译器设置。

## 🔗 相关项目

- [nargo-parser](file:///e:/模板引擎/nargo/compilers/nargo-parser): 用于文档分块解析。
- [nargo-parser-toml](file:///e:/模板引擎/nargo/compilers/nargo-parser-toml): 用于解析项目配置文件。
- [nargo-types](file:///e:/模板引擎/nargo/compilers/nargo-types): 提供位置判断等基础工具。
