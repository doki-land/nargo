# nargo-audit

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20security%20audit%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Audit Logo" width="150" height="150">
  <h3>🔒 Nargo 框架的安全审计工具</h3>
  <p>连接代码质量与生产安全的坚实防线，确保应用在构建阶段就具备基本的安全性</p>
</div>

## 🎯 简介

`nargo-audit` 是 Nargo 编译工具链中的安全扫描组件，它通过对源码和项目配置进行静态分析，能够自动识别潜在的安全风险，包括硬编码的凭据、危险的第三方依赖以及不安全的代码编写模式。作为 Nargo 生态系统中的安全守护者，`nargo-audit` 确保应用在构建阶段就具备基本的安全性，为生产环境保驾护航。

## ✨ 核心特性

### 敏感信息扫描 (Secrets Scanning)
- **多模式识别**: 内置多种正则匹配引擎，可识别 AWS Key、GitHub Token、私钥及通用的密码/密钥模式
- **智能检测**: 采用机器学习算法减少误报，提高检测准确率
- **自定义规则**: 支持用户自定义敏感信息检测规则
- **实时扫描**: 开发过程中实时扫描，及时发现并提醒敏感信息

### 依赖安全审计 (Dependency Audit)
- **全面扫描**: 扫描 `package.json`、`Cargo.toml` 等依赖配置文件
- **漏洞数据库**: 内置最新的漏洞数据库，及时识别已知安全风险
- **依赖链分析**: 分析完整的依赖链，识别间接依赖中的安全问题
- **版本建议**: 提供安全版本的升级建议

### 危险模式识别 (Dangerous Pattern Detection)
- **多语言支持**: 支持 JavaScript/TypeScript、Rust、CSS 等多种语言的危险模式识别
- **常见危险操作**: 识别 `eval()`、`new Function()`、`dangerouslySetInnerHTML` 等危险操作
- **安全最佳实践**: 检查代码是否遵循安全最佳实践
- **框架特定规则**: 针对 React、Vue 等框架的特定安全规则

### 结构化输出与集成
- **统一格式**: 所有发现的安全问题均以统一的 `AuditIssue` 格式输出
- **CI/CD 集成**: 易于集成到 CI/CD 流水线中，支持失败阈值设置
- **详细报告**: 生成详细的安全审计报告，包括问题描述、严重程度和修复建议
- **可配置性**: 支持通过配置文件自定义审计规则和阈值

## 🏗️ 核心数据结构

### AuditIssue
审计问题的核心载体，包含以下信息：
- **文件路径**: 问题所在的文件路径
- **行号**: 问题所在的行号
- **严重等级 (Severity)**: 问题的严重程度（Critical、High、Medium、Low）
- **分类信息**: 问题所属的类别（Secret、Dependency、DangerousPattern）
- **描述**: 问题的详细描述
- **修复建议**: 针对问题的修复建议

### AuditCategory
定义了审计的三大领域：
- **Secret**: 敏感信息，如 API 密钥、密码等
- **Dependency**: 依赖项安全问题
- **DangerousPattern**: 不安全的代码编写模式

### NargoAudit
审计引擎主体，集成了以下能力：
- **文件系统遍历**: 递归扫描项目文件
- **正则匹配逻辑**: 识别敏感信息和危险模式
- **特定文件解析**: 解析 package.json、Cargo.toml 等配置文件
- **结果聚合**: 汇总和分析审计结果

## 🚀 使用方法

### 命令行使用

```bash
# 运行安全审计
nargo audit

# 生成详细报告
nargo audit --report

# 忽略特定漏洞
nargo audit --ignore CVE-2023-1234

# 只扫描特定目录
nargo audit src/backend

# 设置失败阈值
nargo audit --fail-on high
```

### 集成到 CI/CD

```yaml
# GitHub Actions 示例
name: Security Audit
on: [push, pull_request]
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Nargo Audit
        run: nargo audit --fail-on high
```

### 配置文件

`nargo-audit` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.audit]
# 启用/禁用特定审计类别
enable = true
categories = ["secret", "dependency", "dangerous-pattern"]

# 严重程度阈值
fail_on = "high"

# 忽略的漏洞
ignore = ["CVE-2023-1234", "CVE-2023-5678"]

# 自定义敏感信息规则
[tool.nargo.audit.secrets]
custom_patterns = [
  { name = "Custom API Key", pattern = "api_key_[0-9a-f]{32}", severity = "high" }
]

# 依赖审计配置
[tool.nargo.audit.dependency]
registry_url = "https://registry.npmjs.org"
update_interval = "24h"

# 危险模式配置
[tool.nargo.audit.dangerous-pattern]
enable_react_rules = true
enable_node_rules = true
```

## 📊 与其他安全审计工具对比

| 特性 | nargo-audit | npm audit | cargo audit | eslint-plugin-security |
|------|-------------|-----------|-------------|------------------------|
| 多语言支持 | ✅ (TypeScript, Rust, CSS) | ❌ (仅 JavaScript) | ❌ (仅 Rust) | ❌ (仅 JavaScript) |
| 敏感信息扫描 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (有限支持) |
| 依赖审计 | ✅ (多包管理器) | ✅ (仅 npm) | ✅ (仅 Cargo) | ❌ (不支持) |
| 危险模式识别 | ✅ (多语言) | ❌ (不支持) | ❌ (不支持) | ✅ (仅 JavaScript) |
| 自定义规则 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ✅ (完整) |
| CI/CD 集成 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 性能 | ⚡⚡⚡ (快速) | ⚡ (一般) | ⚡⚡ (快速) | ⚡ (一般) |

## 🎯 应用场景

### 开发阶段安全检查
在开发过程中，`nargo-audit` 可以实时扫描代码，及时发现并提醒开发者潜在的安全问题，确保安全意识贯穿整个开发流程。

### CI/CD 流水线集成
在 CI/CD 流水线中集成 `nargo-audit`，可以在代码合并和部署前进行安全检查，防止安全问题进入生产环境。

### 项目安全评估
对于现有项目，`nargo-audit` 可以进行全面的安全评估，识别潜在的安全风险，并提供详细的修复建议。

### 安全合规检查
对于需要符合特定安全标准的项目，`nargo-audit` 可以帮助检查是否符合相关安全要求，生成合规报告。

## 🔧 核心 API

### 基本用法

```rust
use nargo_audit::NargoAudit;
use nargo_audit::types::AuditOptions;

// 创建审计引擎
let audit = NargoAudit::new();

// 配置审计选项
let options = AuditOptions {
    categories: vec!["secret", "dependency", "dangerous-pattern"],
    fail_on: "high",
    ignore: vec!["CVE-2023-1234"],
    ..Default::default()
};

// 运行审计
let results = audit.run("/path/to/project", &options);

// 处理审计结果
for issue in results.issues {
    println!("[{}] {} at {}:{}", issue.severity, issue.description, issue.file_path, issue.line);
}
```

## 🔗 相关项目

- [nargo-linter](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-linter): 提供底层的静态分析与规则校验能力
- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 定义了审计结果中使用的通用上下文与类型系统
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): Nargo 的核心编译引擎
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): Nargo 的命令行工具集合

## 📚 文档

- **[官方文档](https://docs.nargo.dev/audit)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/audit/api)** - 完整的 API 参考
- **[安全最佳实践](https://docs.nargo.dev/audit/best-practices)** - 安全开发最佳实践指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
