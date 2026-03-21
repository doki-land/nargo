# nargo-bridge

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20type%20bridge%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Bridge Logo" width="150" height="150">
  <h3>🔗 Nargo 框架的类型桥接工具</h3>
  <p>连接 Rust 后端与 TypeScript 前端的类型纽带，确保全栈类型安全</p>
</div>

## 🎯 简介

`nargo-bridge` 是 Nargo 全栈开发中的核心组件，负责在 Rust 后端与 TypeScript 前端之间建立类型桥梁。它通过分析 Rust 中的结构体定义，自动生成对应的 TypeScript 接口（Interface），确保前后端在数据交互过程中拥有完整的类型安全保障，消除手动同步类型的开销与潜在错误。作为 Nargo 生态系统中的类型守护者，`nargo-bridge` 实现了真正的全栈类型安全。

## ✨ 核心特性

### 智能类型映射
- **全面类型支持**: 智能将 Rust 的基础类型（String, u32, bool 等）及复合类型（Vec, Option, Result, HashMap 等）映射为 TypeScript 的对应类型
- **泛型支持**: 支持复杂的泛型类型映射，包括嵌套泛型
- **枚举类型**: 自动将 Rust 枚举转换为 TypeScript 联合类型
- **自定义类型**: 支持自定义类型映射规则

### 全栈同步机制
- **简单注册**: 通过 `register` 机制轻松注册需要桥接的类型
- **一键生成**: 一键生成全量 TypeScript 定义文件
- **实时同步**: 在开发模式下实现类型的实时同步，后端类型变更立即反映到前端
- **增量更新**: 支持增量更新，只重新生成变更的类型

### 高性能与可靠性
- **高性能生成**: 纯 Rust 实现的生成引擎，支持秒级完成大规模类型库的转换
- **零依赖产物**: 生成的 `.ts` 文件不依赖任何第三方运行时，可直接在前端项目中引用
- **类型安全**: 确保生成的 TypeScript 类型与 Rust 类型完全匹配
- **错误处理**: 提供详细的错误信息，帮助开发者快速定位类型映射问题

### 高级功能
- **文档同步**: 自动将 Rust 文档注释转换为 TypeScript JSDoc
- **命名约定**: 支持自定义命名约定，如驼峰命名转换
- **类型修饰符**: 支持添加 TypeScript 特定的类型修饰符
- **模块化输出**: 支持将生成的类型按模块组织

## 🏗️ 核心数据结构

### NargoBridge
桥接管理器，负责类型的注册、维护以及最终的 TS 代码生成：
- **类型注册**: 注册需要桥接的 Rust 类型
- **类型分析**: 分析类型的结构和依赖关系
- **代码生成**: 生成 TypeScript 类型定义代码
- **配置管理**: 管理桥接配置选项

### BridgeType
类型的中间表示，记录了类型的详细信息：
- **类型名称**: 类型的名称
- **字段信息**: 类型包含的所有字段及其类型
- **类型参数**: 泛型类型的参数信息
- **文档注释**: 类型的文档注释

### TypeBridge Trait
插件化接口，Rust 结构体通过实现此 Trait 暴露其元数据给桥接引擎：
- **类型元数据**: 提供类型的元数据信息
- **字段信息**: 提供字段的详细信息
- **类型依赖**: 声明类型的依赖关系

## 🚀 使用方法

### 命令行使用

```bash
# 同步类型
nargo bridge

# 生成类型文档
nargo bridge --docs

# 自定义输出路径
nargo bridge --output src/types/rust.d.ts

# 只同步特定模块
nargo bridge --module api

# 启用实时同步
nargo bridge --watch
```

### Rust 后端集成

```rust
use nargo_bridge::TypeBridge;

// 定义需要桥接的结构体
#[derive(TypeBridge)]
#[bridge(name = "User")]
pub struct User {
    /// 用户ID
    pub id: u32,
    /// 用户名
    pub name: String,
    /// 邮箱
    pub email: Option<String>,
    /// 角色
    pub role: Role,
}

// 定义枚举类型
#[derive(TypeBridge)]
#[bridge(name = "Role")]
enum Role {
    /// 管理员
    Admin,
    /// 普通用户
    User,
    /// 访客
    Guest,
}

// 注册类型
fn main() {
    let mut bridge = nargo_bridge::NargoBridge::new();
    bridge.register::<User>();
    bridge.register::<Role>();
    
    // 生成 TypeScript 类型定义
    bridge.generate("src/types/rust.d.ts").unwrap();
}
```

### 前端使用

```typescript
// 导入生成的类型定义
import type { User, Role } from './types/rust';

// 使用类型
const user: User = {
  id: 1,
  name: 'John Doe',
  email: 'john@example.com',
  role: 'Admin'
};

// 类型安全的 API 调用
async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  return response.json();
}
```

## 🔧 配置选项

`nargo-bridge` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.bridge]
# 输出路径
output = "src/types/rust.d.ts"

# 命名约定 (snake_case, camelCase, PascalCase)
naming_convention = "camelCase"

# 启用文档同步
enable_docs = true

# 模块化输出
modular = true

# 包含/排除模块
include_modules = ["api", "models"]
exclude_modules = ["internal"]

# 自定义类型映射
[type_mappings]
"chrono::DateTime<Utc>" = "string"
"uuid::Uuid" = "string"
```

## 📊 与其他类型桥接工具对比

| 特性 | nargo-bridge | tauri-cli | ts-rs | typetag |
|------|--------------|-----------|-------|---------|
| 实时同步 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 泛型支持 | ✅ (完整) | ✅ (有限) | ✅ (完整) | ❌ (不支持) |
| 枚举支持 | ✅ (完整) | ✅ (有限) | ✅ (完整) | ❌ (不支持) |
| 文档同步 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 性能 | ⚡⚡⚡ (快速) | ⚡⚡ (中等) | ⚡⚡ (中等) | ⚡ (一般) |
| 零依赖产物 | ✅ (是) | ✅ (是) | ✅ (是) | ❌ (否) |
| 模块化输出 | ✅ (是) | ❌ (否) | ❌ (否) | ❌ (否) |

## 🎯 应用场景

### 全栈 Rust + TypeScript 项目
在全栈 Rust + TypeScript 项目中，`nargo-bridge` 确保前后端类型完全一致，消除类型错误，提高开发效率。

### API 开发
在 API 开发中，`nargo-bridge` 自动生成前端类型定义，确保 API 调用的类型安全。

### 微服务架构
在微服务架构中，`nargo-bridge` 可以在不同服务之间共享类型定义，确保服务间通信的类型安全。

### 大型项目
在大型项目中，`nargo-bridge` 减少了手动维护类型定义的工作量，确保类型一致性，降低维护成本。

## 🔧 核心 API

### 基本用法

```rust
use nargo_bridge::NargoBridge;

// 创建桥接实例
let mut bridge = NargoBridge::new();

// 注册类型
bridge.register::<User>();
bridge.register::<Product>();

// 配置生成选项
bridge
    .with_output_path("src/types/rust.d.ts")
    .with_naming_convention("camelCase")
    .with_enable_docs(true);

// 生成 TypeScript 类型定义
bridge.generate().unwrap();
```

### 高级用法

```rust
use nargo_bridge::{NargoBridge, TypeMapping};

// 创建桥接实例
let mut bridge = NargoBridge::new();

// 注册类型
bridge.register::<User>();

// 添加自定义类型映射
bridge.add_type_mapping(
    "chrono::DateTime<Utc>",
    TypeMapping::new("string", Some("ISO 8601 日期字符串"))
);

// 生成 TypeScript 类型定义
bridge.generate().unwrap();
```

## 🔗 相关项目

- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 提供桥接过程中使用的基础元数据定义
- [nargo-server](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-server): 在开发模式下可集成此工具实现类型的实时同步
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): Nargo 的核心编译引擎
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): Nargo 的命令行工具集合

## 📚 文档

- **[官方文档](https://docs.nargo.dev/bridge)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/bridge/api)** - 完整的 API 参考
- **[最佳实践](https://docs.nargo.dev/bridge/best-practices)** - 类型桥接最佳实践指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
