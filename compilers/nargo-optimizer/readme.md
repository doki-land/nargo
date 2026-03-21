# nargo-optimizer

> HXO 框架的 AST 优化器，提升运行时性能的核心组件。

## 📖 简介

`nargo-optimizer` 负责在编译阶段对 `nargo-ir` 进行静态分析和转换。通过识别模板中的静态内容、常量提升以及预计算，它能显著减少运行时的计算量和 DOM 操作。

## ✨ 核心特性

- **静态提升 (Static Hoisting)**: 识别模板中的完全静态子树，并将其提升为常量，避免在渲染函数中重复创建。
- **样式提取 (Style Collection)**: 深度扫描模板和脚本，提取 Tailwind 类名并交由 `StyleEngine` 生成原子 CSS。
- **国际化优化 (i18n Optimization)**: 在编译时预填多语言文本，减少客户端的翻译开销。
- **调用追踪**: 追踪脚本中函数和变量的调用频次，为后续的代码内联提供决策依据。

## 🏗️ 核心逻辑

- **Optimizer**: 核心优化引擎，持有 `StyleEngine` 实例并管理整个优化管线。
- **Static Analysis**: 遍历 `TemplateNodeIR`，通过 `is_static` 标记可被提升的节点。
- **Style Processing**: 收集 `JsExpr` 和 `AttributeIR` 中的类名，统一分发至样式引擎。

## 🔗 相关项目

- [nargo-ir](file:///e:/模板引擎/nargo/compilers/nargo-ir): 优化的核心对象。
- [nargo-parser-tailwind](file:///e:/模板引擎/nargo/compilers/nargo-parser-tailwind): 消费提取出的样式类。
- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 在编译管线中调度此优化器。
