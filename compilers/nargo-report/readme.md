# nargo-report

> Nargo 框架的报告生成引擎，将编译与分析数据转化为直观的决策依据。

## 📖 简介

`nargo-report` 负责聚合 Nargo 编译流水线中产出的各类原始数据。它能够将 Lint 检查、单元测试、代码覆盖率以及产物体积分析等碎片化信息，通过灵活的模板引擎（如 Handlebars）转化为精美的 HTML 报告或结构化的 JSON/Markdown 文档，为团队提供一目了然的项目健康度概览。

## ✨ 核心特性

- **多维数据聚合**: 统合了 Lint 诊断、测试用例状态、格式化检查以及覆盖率摘要等全方位数据。
- **美观的 HTML 模板**: 内置响应式报告模版，支持在浏览器中直观查看错误堆栈与覆盖率趋势。
- **产物深度分析**: 配合打包器提供对产物文件大小、构成及占比的详细可视化展示。
- **灵活的导出格式**: 支持根据 CI/CD 需求，灵活产出适合人类阅读或机器解析的不同格式报告。

## 🏗️ 核心数据结构

- **ReportData**: 报告数据的顶级模型，统合了所有子模块的分析产物。
- **NargoReporter**: 报告生成器核心，负责模板注册、数据渲染以及文件持久化。
- **Result Models**: 包含 `LintFileResult`, `TestResultData` 等细分领域的执行结果模型。

## 🔗 相关项目

- [nargo-check](file:///e:/模板引擎/nargo/compilers/nargo-check): 作为报告数据的核心提供者，调用此引擎生成全量自检报告。
- [nargo-coverage](file:///e:/模板引擎/nargo/compilers/nargo-coverage): 为报告提供覆盖率核心数据。
