# nargo-metadata

> Nargo 框架的 API 元数据提取与管理工具，为代码与文档之间搭建智能桥梁。

## 📖 简介

`nargo-metadata` 是 Nargo 编译工具链中的 API 元数据提取组件。它能够从源代码和 IR（中间表示）中自动提取 API 端点信息、类型定义和其他结构化元数据，为文档生成、类型检查和开发工具提供基础数据支持。

## ✨ 核心特性

- **API 端点提取**: 自动识别和解析源代码中的 HTTP 装饰器（如 `@http`），提取控制器、方法、路径和参数信息。
- **类型定义解析**: 从源代码中提取类和接口定义，构建完整的类型系统元数据。
- **多格式支持**: 支持从不同格式的源代码中提取元数据，包括普通函数、异步函数和类方法。
- **结构化输出**: 所有提取的元数据均以统一的 JSON 可序列化格式输出，便于集成到其他工具中。

## 🏗️ 核心数据结构

- **ApiMetadata**: 元数据的顶层容器，包含 API 端点列表和类型定义映射。
- **ApiEndpoint**: 表示单个 API 端点，包含控制器、方法名、HTTP 方法、路径、参数和返回类型等信息。
- **ApiParam**: 表示 API 端点的参数，包含名称、参数类型（Body、Path、Query 等）、类型名称和提取器参数。
- **TypeDefinition**: 表示自定义类型定义，包含类型名称和字段列表。
- **ApiMetadataExtractor**: 元数据提取引擎，负责从源代码和 IR 中提取元数据。

## 🔗 相关项目

- [nargo-ir](file:///e://nargo/compilers/nargo-ir): 提供中间表示，用于更深入的元数据提取。
- [nargo-types](file:///e://nargo/compilers/nargo-types): 定义了元数据中使用的通用类型和错误处理系统。
- [nargo-document](file:///e://nargo/compilers/nargo-document): 利用提取的元数据生成 API 文档。