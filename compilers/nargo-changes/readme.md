# nargo-changes

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20change%20management%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Changes Logo" width="150" height="150">
  <h3>📋 Nargo 框架的版本变更管理工具</h3>
  <p>用于跟踪和管理代码变更，生成标准化的 changelog，提供变更分析和统计</p>
</div>

## 🎯 简介

`nargo-changes` 是 Nargo 框架的核心组件，负责版本变更管理，用于跟踪和管理代码变更，生成标准化的 changelog。它通过结构化的变更集管理，确保项目的版本历史清晰可追溯，为团队协作和版本发布提供有力支持。

## ✨ 核心特性

### 变更类型管理
- **多种变更类型**: 支持 breaking、feature、fix、docs、refactor、perf、test、build、chore 等多种变更类型
- **类型验证**: 自动验证变更类型的有效性
- **类型优先级**: 基于变更类型的优先级排序，确保重要变更优先显示

### 变更集管理
- **结构化存储**: 将变更集存储为 JSON 文件，便于版本控制和审查
- **变更集验证**: 自动验证变更集的完整性和有效性
- **变更集合并**: 支持合并多个变更集，便于批量处理
- **变更集历史**: 维护变更集的完整历史，便于追溯

### Changelog 生成
- **标准化格式**: 自动生成符合规范的 changelog
- **版本分组**: 按版本号分组变更，清晰展示每个版本的变更内容
- **变更分类**: 按变更类型分类展示，便于快速了解变更性质
- **自定义模板**: 支持自定义 changelog 模板，满足不同项目的需求

### 包影响跟踪
- **多包支持**: 跟踪变更影响的包，便于版本管理
- **依赖分析**: 分析变更对依赖包的影响
- **版本管理**: 基于变更影响自动建议版本号变更

### 变更分析和统计
- **变更统计**: 提供变更统计数据，包括变更类型分布、包影响分析等
- **趋势分析**: 分析变更趋势，包括变更率、趋势方向等
- **报告生成**: 生成包含统计数据和趋势分析的综合变更报告
- **可视化支持**: 提供数据可视化支持，便于直观理解变更情况

## 🚀 使用方法

### 命令行使用

```bash
# 创建变更集
nargo changes add --type feature --summary "Add new feature" --description "Add a new feature to the framework" --packages nargo-core,nargo-compiler

# 列出所有变更集
nargo changes list

# 生成 changelog
nargo changes generate --version 1.0.0 --date 2024-01-01

# 清理变更集
nargo changes clear

# 获取变更统计
nargo changes stats

# 分析变更趋势
nargo changes trend --period 30d

# 生成变更报告
nargo changes report --period 30d
```

### API 使用

#### 基本使用

```rust
use nargo_changes::{ChangeSet, ChangeSetManager, ChangeType};
use std::path::Path;

// 创建变更集管理器
let manager = ChangeSetManager::new(Path::new(".changes"));

// 创建变更集
let change_set = ChangeSet {
    id: "123456".to_string(),
    r#type: ChangeType::Feature,
    summary: "Add new feature".to_string(),
    description: Some("Add a new feature to the framework".to_string()),
    author: Some("John Doe".to_string()),
    packages: vec!["nargo-core".to_string(), "nargo-compiler".to_string()],
    prerelease: false,
};

// 保存变更集
manager.create_change_set(&change_set).unwrap();

// 生成 changelog
let changelog = manager.generate_changelog("1.0.0", "2024-01-01").unwrap();
println!("{}", changelog);

// 清理变更集
manager.clear_change_sets().unwrap();
```

#### 变更分析和统计

```rust
use nargo_changes::{ChangeSet, ChangeSetManager, ChangeType};
use std::path::Path;

// 创建变更集管理器
let manager = ChangeSetManager::new(Path::new(".changes"));

// 创建多个变更集
let change_sets = vec![
    ChangeSet {
        id: "1".to_string(),
        r#type: ChangeType::Feature,
        summary: "Add new feature".to_string(),
        description: None,
        author: None,
        packages: vec!["nargo-core".to_string()],
        prerelease: false,
    },
    ChangeSet {
        id: "2".to_string(),
        r#type: ChangeType::Fix,
        summary: "Fix bug".to_string(),
        description: None,
        author: None,
        packages: vec!["nargo-compiler".to_string()],
        prerelease: false,
    },
];

// 保存变更集
for change_set in &change_sets {
    manager.create_change_set(change_set).unwrap();
}

// 获取变更统计
let stats = manager.get_change_stats().unwrap();
println!("{}", stats.generate_summary());

// 分析变更趋势
let trend = manager.analyze_change_trend("30 days").unwrap();
println!("{}", trend.generate_summary());

// 生成变更报告
let report = manager.generate_change_report("30 days").unwrap();
println!("{}", report);
```

## 🔧 配置选项

`nargo-changes` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.changes]
# 变更集存储目录
changes_dir = ".changes"

# Changelog 配置
[tool.nargo.changes.changelog]
# 输出文件路径
output_path = "CHANGELOG.md"
# 模板路径
template_path = "changelog-template.md"
# 是否包含作者信息
include_authors = true
# 是否包含包信息
include_packages = true

# 变更分析配置
[tool.nargo.changes.analysis]
# 趋势分析周期
trend_period = "30d"
# 是否生成可视化报告
generate_visualization = true
# 可视化输出格式
visualization_format = "svg"

# 变更类型配置
[tool.nargo.changes.types]
# 自定义变更类型
custom_types = [
  { name = "security", description = "Security improvements", priority = 2 }
]
# 变更类型排序
type_order = ["breaking", "feature", "fix", "security", "perf", "refactor", "docs", "test", "build", "chore"]
```

## 📊 与其他变更管理工具对比

| 特性 | nargo-changes | conventional-changelog | changesets | git-cliff |
|------|---------------|-------------------------|------------|-----------|
| 变更类型管理 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (有限) |
| 变更集存储 | ✅ (JSON) | ❌ (不支持) | ✅ (JSON) | ❌ (不支持) |
| 包影响跟踪 | ✅ (完整) | ❌ (不支持) | ✅ (完整) | ❌ (不支持) |
| 变更统计分析 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 变更趋势分析 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 自定义模板 | ✅ (完整) | ✅ (完整) | ✅ (有限) | ✅ (完整) |
| 多包支持 | ✅ (完整) | ❌ (不支持) | ✅ (完整) | ❌ (不支持) |
| 命令行工具 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |

## 🎯 应用场景

### 版本发布管理
在版本发布过程中，`nargo-changes` 可以帮助团队跟踪和管理变更，生成标准化的 changelog，确保版本发布的透明度和可追溯性。

### 团队协作
在团队协作中，`nargo-changes` 可以帮助团队成员了解项目的变更历史，便于协作和代码审查。

### 开源项目管理
在开源项目中，`nargo-changes` 可以帮助维护者跟踪和管理贡献，生成标准化的 changelog，提高项目的专业性和可维护性。

### 企业项目管理
在企业项目中，`nargo-changes` 可以帮助企业跟踪和管理代码变更，生成标准化的 changelog，满足合规要求和审计需求。

## 🔧 核心 API

### ChangeSet
变更集的核心数据结构，包含变更的详细信息：
- **id**: 变更集的唯一标识符
- **type**: 变更类型（breaking、feature、fix 等）
- **summary**: 变更的简短描述
- **description**: 变更的详细描述（可选）
- **author**: 变更的作者（可选）
- **packages**: 变更影响的包列表
- **prerelease**: 是否为预发布变更

### ChangeSetManager
变更集管理器，负责变更集的创建、读取、生成等操作：
- **create_change_set**: 创建新的变更集
- **read_change_sets**: 读取所有变更集
- **generate_changelog**: 生成 changelog
- **clear_change_sets**: 清理变更集
- **get_change_stats**: 获取变更统计数据
- **analyze_change_trend**: 分析变更趋势
- **generate_change_report**: 生成变更报告

### ChangeStats
变更统计数据，包含变更的统计信息：
- **total_changes**: 总变更数
- **type_distribution**: 变更类型分布
- **package_impact**: 包影响分析
- **author_contribution**: 作者贡献分析
- **generate_summary**: 生成统计摘要

### ChangeTrend
变更趋势分析，包含变更的趋势信息：
- **change_rate**: 变更率
- **trend_direction**: 趋势方向
- **peak_periods**: 变更高峰期
- **generate_summary**: 生成趋势摘要

## 🏗️ 模块结构

### change_set
变更集的核心实现，包含变更集的定义和操作：
- **ChangeSet**: 变更集的数据结构
- **ChangeType**: 变更类型的定义
- **ChangeSetValidation**: 变更集的验证逻辑

### manager
变更集管理器的实现，负责变更集的管理和操作：
- **ChangeSetManager**: 变更集管理器
- **ChangeSetStorage**: 变更集的存储逻辑
- **ChangelogGenerator**: Changelog 的生成逻辑

### analysis
变更分析的实现，包含变更统计和趋势分析：
- **ChangeStats**: 变更统计数据
- **ChangeTrend**: 变更趋势分析
- **ReportGenerator**: 变更报告生成

## 🔗 相关项目

- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 基础类型定义
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): 命令行工具集成
- [nargo-release](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-release): 版本发布工具，与变更管理配合使用
- [nargo-registry](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-registry): 包注册表客户端

## 📚 文档

- **[官方文档](https://docs.nargo.dev/changes)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/changes/api)** - 完整的 API 参考
- **[最佳实践](https://docs.nargo.dev/changes/best-practices)** - 变更管理最佳实践指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
