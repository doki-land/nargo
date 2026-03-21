# nargo-scaffolder

> Nargo 框架的脚手架工具，开启高效 Nargo 开发的第一步。

## 📖 简介

`nargo-scaffolder` 提供了快速初始化 Nargo 项目的能力。它支持从本地或远程模板库拉取标准化的 Nargo 项目结构，自动配置目录架构、基础源码以及必要的依赖文件，帮助开发者在秒级时间内搭建起开箱即用的全栈开发环境。

## ✨ 核心特性

- **极速初始化**: 一键生成标准的 `src`, `public` 目录及 `index.nargo` 示例文件。
- **智能模板注入**: 支持根据项目名称动态渲染 `package.json` 等配置文件。
- **规范引导**: 自动配置符合 Nargo 最佳实践的目录结构与初始代码风格。
- **跨平台一致性**: 确保在不同操作系统下生成的项目结构与权限设置完全一致。

## 🏗️ 核心逻辑

- **init 方法**: 脚手架的核心入口，负责文件系统的创建与初始模板的写入。
- **Template Rendering**: 基于字符串格式化或高级模板引擎实现的配置文件动态生成。
- **Directory Structure**: 定义了 Nargo 项目的官方标准布局规范。

## 🔗 相关项目

- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 为脚手架生成的项目提供核心编译能力。
- [nargo-server](file:///e:/模板引擎/nargo/compilers/nargo-server): 配合脚手架，实现“即开即用”的开发体验。
