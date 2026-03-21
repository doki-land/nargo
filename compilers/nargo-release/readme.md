# nargo-release

> Nargo 框架的发布管理工具，自动化多包工作区的版本更迭与发布流程。

## 📖 简介

`nargo-release` 专为管理 Nargo Monorepo 中的多包发布而设计。它能够自动识别 Cargo 和 npm 包的版本状态，分析依赖图谱以确定发布的先后顺序，并自动化生成变更日志（Changelog）与 Git 标签，确保在复杂的全栈项目中，版本发布过程既严谨又高效。

## ✨ 核心特性

- **多包依赖编排**: 基于 `petgraph` 构建依赖图，利用拓扑排序算法自动计算多包发布的最佳顺序。
- **混合栈支持**: 同时支持 Rust (Cargo.toml) 和 TypeScript (package.json) 项目的版本解析与更新。
- **自动化变更日志**: 智能提取提交信息，自动生成符合语义化版本的变更记录。
- **Git 深度集成**: 自动完成代码提交、标签创建以及分支合并等发布后续操作。

## 🏗️ 核心数据结构

- **ReleaseManager**: 发布流程的总控引擎，持有工作区上下文与包信息集合。
- **PackageInfo**: 包的统一抽象模型，封装了名称、版本、路径及跨语言依赖关系。
- **Topology Sort Engine**: 核心算法逻辑，确保依赖包优先于被依赖包进行发布。

## 🔗 相关项目

- [nargo-mono](file:///e:/模板引擎/nargo/compilers/nargo-mono): 提供工作区发现与成员识别的基础能力。
- [nargo-config](file:///e:/模板引擎/nargo/compilers/nargo-config): 为发布策略提供配置支持。
