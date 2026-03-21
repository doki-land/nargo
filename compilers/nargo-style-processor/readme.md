# nargo-style-processor

> HXO 框架的样式处理器，支持多种样式预处理器和实时预览功能。

## 📖 简介

`nargo-style-processor` 负责对 CSS 代码进行处理，支持多种样式预处理器，并提供实时预览和热更新功能。

## ✨ 核心特性

- **多预处理器支持**: 支持 Sass/SCSS、Less、Stylus 等主流样式预处理器。
- **实时预览**: 监听文件变化并自动重新处理样式，实现热更新。
- **样式优化**: 支持压缩 CSS 和移除未使用的样式。
- **扩展性设计**: 结构上预留了 PostCSS 风格的转换管线接口。

## 🏗️ 核心逻辑

- **StyleProcessor**: 提供 `process` 方法，处理 CSS 转换流程（如预处理器转换、压缩等）。
- **预处理器集成**: 集成了 Sass/SCSS 预处理器，并为 Less 和 Stylus 预处理器预留了接口。
- **实时监听**: 提供 `watch` 方法，监听文件变化并自动重新处理样式。

## 🔗 相关项目

- [nargo-target-css](file:///e://nargo/compilers/nargo-bundler/src/targets/css): 负责样式的写入与简单压缩。
- [nargo-compiler](file:///e://nargo/compilers/nargo-compiler): 编译管线的末端调用者。
