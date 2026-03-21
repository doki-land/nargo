# nargo-hydrate

> HXO 框架的客户端激活 (Hydration) 逻辑。

## 📖 简介

`nargo-hydrate` 负责在客户端接管服务端渲染的静态 HTML。它通过复用已有的 DOM 节点并建立响应式绑定，实现页面的平滑激活，避免闪烁和性能浪费。

## ✨ 核心特性

- **非破坏性激活**: 智能匹配 DOM 结构，利用 `data-nargo-id` 仅在必要时更新节点。
- **事件绑定**: 自动为 SSR 生成的静态 HTML 节点挂载事件监听器。
- **响应式重建**: 在客户端重新建立 Signal 绑定，使静态页面恢复交互能力。
- **异步 Hydration**: 支持分块激活页面，优先响应用户交互。

## 🏗️ 核心逻辑

- **HydrateBackend**: 生成激活脚本的核心后端，利用 `JsWriter` 构建 `hydrate` 函数。
- **节点检索**: 使用 `querySelector` 和 `data-nargo-id` 快速定位需要激活的 DOM 元素。
- **副作用建立**: 对插值和动态属性调用 `createEffect`，将其与客户端状态同步。

## 🔗 相关项目

- [nargo-ir](file:///e:/模板引擎/nargo/compilers/nargo-ir): 提供组件结构的中间表示。
- [nargo-ssr](file:///e:/模板引擎/nargo/compilers/nargo-ssr): 生成此包所需的 HTML 和 ID。
- [nargo-target-js](file:///e:/模板引擎/nargo/compilers/nargo-target-js): 提供 `JsWriter` 用于生成激活代码。
