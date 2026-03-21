# nargo-script-analyzer

> HXO 脚本语义分析器，驱动响应式系统的元数据提取引擎和静态分析规则引擎。

## 📖 简介

`nargo-script-analyzer` 负责对 HXO 组件中的脚本块进行静态分析。它直接操作 `nargo-ir` 定义的 JS 语法树，从中提取出 `createSignal` 和 `createComputed` 等响应式定义，为后续的代码生成和类型推导提供关键的元数据支持。同时，它还提供了静态分析规则引擎，支持代码质量检查和分析报告生成。

## ✨ 核心特性

- **响应式特征提取**: 扫描变量声明，自动识别并收集 `createSignal`（支持数组解构）和 `createComputed` 调用。
- **元数据生成**: 生成包含 `signals`, `props`, `emits` 的 `ScriptMetadata` 结构，作为组件的“数字指纹”。
- **轻量化分析**: 直接运行在精简的 `nargo-ir` 结构上，分析过程极快且无外部重型库依赖。
- **解构语法支持**: 能够正确处理数组解构赋值，准确捕捉 Signal 的读操作变量名。
- **静态分析规则引擎**: 支持代码质量检查，包括未使用变量、未定义变量、不安全操作等规则。
- **分析报告生成**: 提供控制台输出和 JSON 格式的分析报告，方便集成到 CI/CD 流程中。

## 🏗️ 核心逻辑

1. **AST Traversal**: 深度遍历由解析器生成的 `JsProgram` 节点。
2. **Pattern Matching**: 在变量声明（`VariableDecl`）语句中寻找特定的响应式函数调用模式。
3. **Destructuring Analysis**: 专门处理 `[count, setCount] = createSignal(0)` 模式，提取出 `count` 作为 Signal 标识。
4. **Metadata Assembly**: 汇总分析结果并构建 `ScriptMetadata` 供编译器后续阶段使用。
5. **Static Analysis**: 应用规则引擎对代码进行质量检查，生成分析报告。

## 🛠️ 静态分析规则

### 内置规则

| 规则名称 | 规则代码 | 级别 | 描述 |
|---------|---------|------|------|
| 未使用变量 | unused-variable | Warning | 检测未使用的变量声明 |
| 未定义变量 | undefined-variable | Error | 检测使用未定义的变量 |
| 不安全操作 | unsafe-operation | Error/Warning | 检测除以零、使用 eval 等不安全操作 |

## 📊 分析报告

### 控制台报告

分析报告会在控制台输出详细的问题列表，包括问题级别、代码、描述和位置信息。

### JSON 报告

分析报告可以生成为 JSON 格式，方便集成到 CI/CD 流程中进行自动化检查。

## 🚀 使用方法

### 基本使用

```rust
use nargo_script_analyzer::ScriptAnalyzer;
use nargo_ir::JsProgram;

let analyzer = ScriptAnalyzer::new();
let program: JsProgram = /* 解析得到的语法树 */;

// 传统分析（仅提取元数据）
let meta = analyzer.analyze(&program).unwrap();

// 带规则检查的分析（提取元数据 + 静态分析）
let (meta, report) = analyzer.analyze_with_rules(&program, Some("path/to/file.js")).unwrap();

// 生成控制台报告
println!("{}", report.generate_console_report());

// 保存 JSON 报告
report.save_to_file("analysis-report.json").unwrap();
```

### 自定义规则

```rust
use nargo_script_analyzer::{Rule, RuleEngine, AnalysisReport, ScriptMetadata};
use nargo_ir::JsProgram;

struct CustomRule;

impl Rule for CustomRule {
    fn code(&self) -> String {
        "custom-rule".to_string()
    }

    fn description(&self) -> String {
        "自定义规则描述".to_string()
    }

    fn check(&self, program: &JsProgram, meta: &ScriptMetadata, report: &mut AnalysisReport) {
        // 实现自定义规则逻辑
    }
}

// 创建规则引擎并添加自定义规则
let mut engine = RuleEngine::new();
engine.add_rule(Box::new(CustomRule));

// 执行规则检查
let mut report = AnalysisReport::new(Some("path/to/file.js"));
engine.run(&program, &meta, &mut report);
```

## 🔗 相关项目

- [nargo-ir](file:///e:/模板引擎/nargo/compilers/nargo-ir): 提供分析所需的 AST 输入。
- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 调用此包提取组件核心元数据。
- [nargo-target-dts](file:///e:/模板引擎/nargo/compilers/nargo-target-dts): 利用提取的元数据生成类型定义。

