# vmz-audit

> Vmz 框架的安全审计工具，连接代码质量与生产安全的坚实防线。

## 📖 简介

`vmz-audit` 是 Vmz 编译工具链中的安全扫描组件。它通过对源码和项目配置进行静态分析，能够自动识别潜在的安全风险，包括硬编码的凭据、危险的第三方依赖以及不安全的代码编写模式，确保应用在构建阶段就具备基本的安全性。

## ✨ 核心特性

- **敏感信息扫描 (Secrets Scanning)**: 内置多种正则匹配引擎，可识别 AWS Key、GitHub Token、私钥及通用的密码/密钥模式。
- **依赖安全审计 (Dependency Audit)**: 扫描 `package.json` 中的依赖项，识别已知已废弃或存在安全风险的 npm 包（如 `request`）。
- **危险模式识别 (Dangerous Pattern Detection)**: 静态识别 JavaScript/TypeScript 中的危险操作，如 `eval()`、`new Function()` 以及 React/Vmz 中的 `dangerouslySetInnerHTML`。
- **结构化输出**: 所有发现的安全问题均以统一的 `AuditIssue` 格式输出，便于集成到 CI/CD 流水线中。

## 🏗️ 核心数据结构

- **AuditIssue**: 审计问题的核心载体，包含文件路径、行号、严重等级（Severity）以及详细的分类信息。
- **AuditCategory**: 定义了审计的三大领域：`Secret` (凭据)、`Dependency` (依赖) 和 `DangerousPattern` (代码模式)。
- **VmzAudit**: 审计引擎主体，集成了文件系统遍历、正则匹配逻辑以及特定文件的解析能力。

## 🔗 相关项目

- [vmz-linter](file:///e:/模板引擎/vmz/compilers/vmz-linter): 提供底层的静态分析与规则校验能力。
- [vmz-types](file:///e:/模板引擎/vmz/compilers/vmz-types): 定义了审计结果中使用的通用上下文与类型系统。
