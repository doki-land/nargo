# nargo-ssr

> HXO 框架的服务端渲染 (SSR) 核心逻辑。

## 📖 简介

`nargo-ssr` 负责将 `nargo-ir` 转换为可在服务端运行的渲染函数。它不仅生成 HTML 字符串，还负责注入 Hydration 所需的元数据（如 `data-nargo-id`），确保客户端能够平滑接管。

## ✨ 核心特性

- **高效字符串拼接**: 使用 `JsWriter` 生成基于字符串拼接的渲染函数，最大限度减少运行时的内存分配。
- **Hydration 友好**: 自动为动态节点注入 `data-nargo-id`，实现服务端与客户端的精准匹配。
- **指令支持**: 处理模板中的 SSR 特有指令，如条件渲染和列表循环。
- **状态注入**: 支持将初始化状态序列化并嵌入 HTML 页面中。

## 🏗️ 核心逻辑

- **SsrBackend**: 核心生成引擎，利用 `JsWriter` 构建 `render(ctx)` 函数。
- **标识符注入**: 为所有非静态元素分配唯一的 ID，以便 `nargo-hydrate` 在客户端识别。
- **静态提升**: 模板中的静态部分在编译时直接转化为 HTML 字符串常量。

## 🔗 相关项目

- [nargo-ir](file:///e:/模板引擎/nargo/compilers/nargo-ir): 输入的中间表示。
- [nargo-target-js](file:///e:/模板引擎/nargo/compilers/nargo-target-js): 提供代码写入工具 `JsWriter`。
- [nargo-hydrate](file:///e:/模板引擎/nargo/compilers/nargo-hydrate): 负责在客户端消费此包生成的 HTML。
