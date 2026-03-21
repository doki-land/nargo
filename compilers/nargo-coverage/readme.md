# nargo-coverage

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20code%20coverage%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Coverage Logo" width="150" height="150">
  <h3>📊 Nargo 框架的代码覆盖率插桩工具</h3>
  <p>连接编译管线与精准测试度量的桥梁，为测试提供详实的覆盖率数据支撑</p>
</div>

## 🎯 简介

`nargo-coverage` 是 Nargo 框架的核心代码覆盖率工具，负责在编译过程中对 `IRModule` 进行自动化插桩。它通过在语句、分支和函数入口处注入计数器代码，实现了对代码执行路径的精确追踪，为后续的覆盖率报告提供详实的数据支撑。作为 Nargo 测试生态系统的重要组成部分，`nargo-coverage` 为开发者提供了全面、准确的代码覆盖率分析能力。

## ✨ 核心特性

### 编译器级插桩
- **中间表示操作**: 直接操作 `nargo-ir` 的中间表示，相比源码级转换更加稳健且易于维护
- **精确插桩**: 在编译过程中精确注入覆盖率代码，确保数据的准确性
- **低开销**: 采用高效的插桩算法，对编译速度的影响微乎其微
- **可扩展性**: 支持自定义插桩策略，满足不同场景的需求

### 全方位覆盖
- **语句覆盖**: 统计每个语句的执行情况
- **分支覆盖**: 统计每个分支的执行情况，包括条件表达式的真假分支
- **函数覆盖**: 统计每个函数的执行情况
- **模板覆盖**: 支持对模板中的动态表达式进行覆盖统计
- **脚本覆盖**: 支持对脚本中的逻辑代码进行覆盖统计

### 统一覆盖分析
- **模板与脚本统一**: 不仅支持 `<script>` 块中的 JS/TS 逻辑，还能对 `<template>` 中的动态表达式进行插桩
- **跨文件分析**: 支持跨文件的覆盖率分析，提供整体项目的覆盖率视图
- **增量覆盖**: 支持增量覆盖率分析，只分析变更的代码

### 高效数据结构
- **零开销数据结构**: 采用紧凑的 `CoverageMetadata` 结构，确保插桩过程对编译速度的影响微乎其微
- **内存优化**: 高效的内存使用，支持处理大型项目的覆盖率数据
- **序列化支持**: 支持覆盖率数据的序列化和反序列化，便于存储和传输

## 🚀 使用方法

### 命令行使用

```bash
# 运行测试并收集覆盖率数据
nargo test --coverage

# 生成覆盖率报告
nargo coverage report

# 生成 HTML 覆盖率报告
nargo coverage report --format html

# 生成 LCOV 覆盖率报告
nargo coverage report --format lcov

# 查看覆盖率统计
nargo coverage stats
```

### API 使用

```rust
use nargo_coverage::{CoverageInstrumenter, CoverageOptions};
use nargo_ir::IRModule;

fn main() -> anyhow::Result<()> {
    // 加载 IR 模块
    let mut module = IRModule::load("src/main.nargo")?;

    // 创建覆盖率插桩器
    let instrumenter = CoverageInstrumenter::new();

    // 配置插桩选项
    let options = CoverageOptions {
        enable_statement_coverage: true,
        enable_branch_coverage: true,
        enable_function_coverage: true,
        enable_template_coverage: true,
        ..Default::default()
    };

    // 执行插桩
    let instrumented_module = instrumenter.instrument(module, &options)?;

    // 保存插桩后的模块
    instrumented_module.save("dist/main.instrumented.nargo")?;

    println!("插桩成功！");
    Ok(())
}
```

## 🔧 配置选项

`nargo-coverage` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.coverage]
# 启用的覆盖类型
enable_statement_coverage = true
enable_branch_coverage = true
enable_function_coverage = true
enable_template_coverage = true

# 报告配置
[tool.nargo.coverage.report]
# 输出格式 (html, lcov, json)
format = "html"
# 输出目录
output_dir = "coverage"
# 是否生成详细报告
detailed = true
# 是否包含未覆盖的代码
include_uncovered = true

# 阈值配置
[tool.nargo.coverage.threshold]
# 语句覆盖率阈值
statement = 80
# 分支覆盖率阈值
branch = 70
# 函数覆盖率阈值
function = 90
# 是否在阈值不达标时失败
fail_on_threshold = true
```

## 📊 与其他覆盖率工具对比

| 特性 | nargo-coverage | Istanbul | nyc | cargo-tarpaulin |
|------|----------------|----------|-----|------------------|
| 编译器级插桩 | ✅ (完整) | ❌ (源码级) | ❌ (源码级) | ✅ (完整) |
| 模板覆盖 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 分支覆盖 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 函数覆盖 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 语句覆盖 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 增量覆盖 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 跨文件分析 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 性能开销 | ⚡⚡⚡ (低) | ⚡⚡ (中等) | ⚡⚡ (中等) | ⚡ (高) |

## 🎯 应用场景

### 单元测试覆盖率
在单元测试中，`nargo-coverage` 可以提供详细的覆盖率数据，帮助开发者了解测试的覆盖情况，识别未测试的代码区域。

### 集成测试覆盖率
在集成测试中，`nargo-coverage` 可以提供整体系统的覆盖率视图，帮助开发者了解集成测试的覆盖情况。

### 持续集成
在 CI/CD 流水线中，`nargo-coverage` 可以作为质量门禁，确保代码覆盖率达到预定阈值，提高代码质量。

### 代码质量评估
在代码质量评估中，`nargo-coverage` 可以提供客观的覆盖率数据，作为代码质量评估的重要指标。

## 🔧 核心 API

### CoverageInstrumenter
覆盖率插桩器，负责对 IR 模块进行插桩：
- **new**: 创建新的插桩器实例
- **instrument**: 对 IR 模块进行插桩
- **instrument_file**: 对单个文件进行插桩
- **instrument_function**: 对单个函数进行插桩

### CoverageOptions
插桩选项，控制插桩的行为：
- **enable_statement_coverage**: 是否启用语句覆盖
- **enable_branch_coverage**: 是否启用分支覆盖
- **enable_function_coverage**: 是否启用函数覆盖
- **enable_template_coverage**: 是否启用模板覆盖

### CoverageMetadata
覆盖率元数据，存储覆盖率相关的信息：
- **file_path**: 文件路径
- **statements**: 语句映射
- **branches**: 分支元数据
- **functions**: 函数元数据
- **counters**: 命中计数器

### BranchMetadata
分支元数据，记录分支的信息：
- **branch_type**: 分支类型（如 `cond-expr`）
- **location**: 分支在源码中的物理位置
- **true_counter**: 真分支的命中计数器
- **false_counter**: 假分支的命中计数器

## 🏗️ 核心数据结构

### CoverageMetadata
顶层容器，存储了文件路径以及所有的语句映射、分支元数据和对应的命中计数器：
- **file_path**: 文件路径
- **statements**: 语句映射，记录每个语句的位置和计数器
- **branches**: 分支元数据，记录每个分支的类型和位置
- **functions**: 函数元数据，记录每个函数的位置和计数器
- **counters**: 命中计数器，记录每个语句、分支和函数的命中次数

### BranchMetadata
记录分支的类型（如 `cond-expr`）及其在源码中的物理位置：
- **branch_type**: 分支类型
- **start_line**: 分支开始行号
- **start_column**: 分支开始列号
- **end_line**: 分支结束行号
- **end_column**: 分支结束列号
- **true_counter**: 真分支的命中计数器
- **false_counter**: 假分支的命中计数器

### CoverageInstrumenter
执行插桩逻辑的主体类，负责递归遍历 AST 并生成注入代码：
- **instrument**: 对整个模块进行插桩
- **instrument_statement**: 对语句进行插桩
- **instrument_branch**: 对分支进行插桩
- **instrument_function**: 对函数进行插桩
- **instrument_template**: 对模板进行插桩

## 🔗 相关项目

- [nargo-test-runner](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-test-runner): 消费插桩后的代码并收集运行时的覆盖率数据
- [nargo-report](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-report): 将收集到的覆盖率数据转换为可视化的报告（如 LCOV/HTML）
- [nargo-ir](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-ir): 插桩操作的核心数据对象
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): 核心编译引擎，集成覆盖率插桩功能

## 📚 文档

- **[官方文档](https://docs.nargo.dev/coverage)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/coverage/api)** - 完整的 API 参考
- **[最佳实践](https://docs.nargo.dev/coverage/best-practices)** - 覆盖率测试最佳实践指南
- **[报告解读](https://docs.nargo.dev/coverage/report)** - 覆盖率报告解读指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
