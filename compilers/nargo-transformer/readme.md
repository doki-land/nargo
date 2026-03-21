# nargo-transformer

> HXO 框架的变换追踪与 IR 操纵模块。

## 📖 简介

`nargo-transformer` 负责在编译过程中对中间表示 (IR) 进行变换，并提供全链路的变换追踪能力。它允许开发者记录每一步变换操作，为调试、性能分析和 SourceMap 的精准映射提供数据支持。

## ✨ 核心特性

- **变换追踪**: 自动或手动记录每一次对 IR 的修改，包括操作名称、描述、时间戳及受影响的代码范围。
- **插件化接口**: 通过 `TransformPass` trait，支持以插件形式扩展编译器的变换能力。
- **透明性**: 为复杂的编译流程提供透明的审计日志，方便排查变换过程中引入的问题。

## 🏗️ 核心逻辑

- **Transformer**: 变换管理器，负责持有变换日志并应用变换 Pass。
- **Transformation**: 单次变换的记录结构，包含元数据和位置信息。
- **TransformPass**: 变换插件接口，定义了统一的 `transform` 逻辑。

## 🔗 相关项目

- [nargo-ir](file:///e:/模板引擎/nargo/compilers/nargo-ir): 此模块操纵的核心数据结构。
- [nargo-optimizer](file:///e:/模板引擎/nargo/compilers/nargo-optimizer): 使用此模块进行更高级的 IR 优化与追踪。
