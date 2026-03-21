# HXO 全栈开发文档项目

该项目是 **HXO (TypeScript On Codex)** 的官方文档库，基于 [VitePress](https://vitepress.dev/) 构建。

## 🌟 什么是 HXO?

HXO 是 **TypeScript On Codex 思想的实现**。它是一款革命性的 Web 框架，吸收了现代前端的开发直觉，并结合了现代编译器技术与 Signal 响应式模型。我们不只是在做另一个框架，而是在通过底层重构，彻底消除传统框架中虚拟 DOM (VDOM) 的开销和 `.value` 的心智负担。

## 文档结构

- **[架构与职责](./architecture.md)**: 了解 HXO 的定位、核心职责、与 VMZ 框架的关系。
- **[入门指南](./frontend/index.md)**: 快速上手 HXO，了解单页开发和基本语法。
- **[全栈整合](./integrated/index.md)**: 深入了解控制器、服务、数据库等整合型应用开发。
- **[分布式架构](./distributed/index.md)**: 学习逻辑漂移、契约驱动等分布式开发特性。
- **[插件与基础设施](./plugins.md)**: 了解如何扩展 HXO 的核心能力。
- **[常见问题](./faq.md)**: 汇总开发过程中可能遇到的各类问题。

## 开发与预览

本项目使用 VitePress 驱动，支持实时预览和静态站点生成。

### 安装依赖

在文档根目录下，使用 pnpm 安装：

```bash
pnpm install
```

### 启动开发服务器

```bash
pnpm dev
```

启动后可在浏览器访问 `http://localhost:5173` 进行实时预览。

### 构建静态站点

```bash
pnpm build
```

构建产物将保存在 `.vitepress/dist` 目录中。

## 参与贡献

我们欢迎并感谢任何形式的贡献，包括修复错别字、完善文档内容或翻译。

1. Fork 本仓库
2. 创建特性分支
3. 提交更改
4. 发起 Pull Request

---

**HXO Team** - 致力于构建下一代全栈开发体验。
