# nargo-parser

> Nargo 框架的 SFC 分块解析器与解析器调度中心。

## 📖 简介

`nargo-parser` 是 Nargo 组件解析的第一站。它负责将 `.nargo` 单文件组件（SFC）拆分为不同的功能块（如 `<script>`, `<template>`, `<style>`），并根据块的属性（如 `lang="scss"`）智能地调度相应的子解析器。

## ✨ 核心特性

- **SFC 分块**: 能够稳健地解析带有任意嵌套层次的 SFC 文件。
- **解析器注册机制**: 允许动态注册不同语言的子解析器，支持 `nargo`, `pug`, `ts`, `scss`, `yaml` 等多种格式。
- **智能分发**: 自动读取块上的 `lang` 或 `type` 属性，并调用匹配的解析器插件。
- **位置感知**: 在分块过程中全程保持源码位置追踪，确保后续生成的 Source Map 准确无误。

## 🏗️ 核心逻辑

1. **扫描**: 识别顶层标签及其属性。
2. **提取**: 提取每个块的内容，并计算其其在原始文件中的偏移。
3. **分发**: 将提取出的内容传递给注册的子解析器（如 `nargo-parser-template`）。
4. **聚合**: 汇总所有子解析器的结果，生成 `ParsedNargoFile`。

## 🔗 相关项目

- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 调用此包作为解析阶段的入口。
- [nargo-parser-*](file:///e:/模板引擎/nargo/compilers/): 各类语言专用的子解析器插件。
