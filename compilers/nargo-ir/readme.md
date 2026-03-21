# nargo-ir

> HXO 框架的自定义中间表示 (IR)，连接解析与生成的桥梁。

## 📖 简介

`nargo-ir` 定义了 HXO 编译过程中使用的中间表示。它通过一套针对 HXO 优化的 AST 结构，实现了对脚本、模板和样式的统一建模，支持 HXO 特有的语义（如 `TseElement`）。

## ✨ 核心特性

- **脚本表示**: 提供了 `JsExpr` 和 `JsStmt` 枚举，涵盖了 ES6+ 语法，并原生支持 `TseElement`。
- **语义化模板 IR**: `TemplateIR` 将模板结构抽象为元素、文本、插值和注释，为后续的代码生成提供直接指令。
- **零外部依赖**: AST 结构完全自包含，确保了编译链路的纯粹性和可控性。
- **序列化支持**: 全面支持 `serde`，方便将编译后的中间结果进行缓存或持久化。

## 🏗️ 核心数据结构

- **JsExpr**: 表达式枚举，支持标识符、字面量、函数调用、二元运算、箭头函数以及特殊的 **TseElement**（模板脚本元素）。
- **JsStmt**: 语句枚举，支持变量声明（`VariableDecl`）、导入导出（`Import`/`Export`）、函数定义（`FunctionDecl`）等。
- **IRModule**: 顶层容器，统合了 `JsProgram` (脚本)、`TemplateIR` (模板)、`StyleIR` (样式) 及其他元数据。

## 🔗 相关项目

- [nargo-compiler](file:///e:/模板引擎/nargo/compilers/nargo-compiler): 负责生成此 IR。
- [nargo-target-js](file:///e:/模板引擎/nargo/compilers/nargo-target-js): 消费此 IR 并生成最终代码。
