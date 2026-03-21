# Nargo Metadata + Nargo SDK 使用示例

这个示例展示了如何使用 Nargo Metadata 提取 API 元数据，并使用 Nargo SDK 生成 TypeScript SDK。

## 功能演示

1. **元数据提取** - 从 TypeScript 源代码中提取 API 端点和类型定义
2. **SDK 生成** - 基于提取的元数据生成类型安全的 TypeScript SDK

## 项目结构

```
nargo-sdk-example/
├── Cargo.toml          # 项目配置文件
├── README.md           # 说明文档
├── src/
│   └── main.rs         # 主程序代码
└── output/             # SDK 输出目录（运行后生成）
    ├── types.ts        # TypeScript 类型定义
    └── api.ts        # API 客户端实现
```

## 运行示例

### 构建项目

```bash
cargo build
```

### 运行示例

```bash
cargo run
```

### 运行测试

```bash
cargo test
```

## API 装饰器语法

### HTTP 方法装饰器：

```typescript
@http("GET", "/users")
@http("POST", "/users")
@http("PUT", "/users/:id")
@http("DELETE", "/users/:id")
@http("PATCH", "/users/:id")
```

### 参数装饰器：

```typescript
@Path id: number
@Query page: number
@Body user: User
@Header authorization: string
@CurrentUser user: User
@Session session: Session
@I18n i18n: I18n
```

## 代码说明

### 提取元数据

```rust
let ir = nargo_ir::IRModule::default();
let metadata = ApiMetadataExtractor::extract(source, &ir)?;

println!("找到 {} 个 API 端点", metadata.endpoints.len());
println!("找到 {} 个类型定义", metadata.types.len());
```

### 生成 SDK

```rust
let output_dir = Path::new("./output");
SdkGenerator::generate(&metadata, output_dir)?;
```

## 生成的文件说明

### types.ts

包含所有 TypeScript 类型定义：

```typescript
export interface User {
  id: number;
  name: string;
  email: string;
  createdAt: Date;
}

export interface CreateUserDto {
  name: string;
  email: string;
  password: string;
}
```

### api.ts

包含类型安全的 API 客户端：

```typescript
import { useClient } from '@nargo/client';
import type * as Types from './types';

export const UserController = {
  async listUsers(page: number, limit: number): Promise<Types.User[]> {
    const client = useClient();
    return client.get(`/users`, { params: { page, limit } });
  },
};
```

## 在前端使用生成的 SDK

### 安装依赖

```bash
npm install @nargo/client
```

### 使用 SDK

```typescript
import { UserController, ProductController } from './output/api';

async function example() {
    const users = await UserController.listUsers(1, 10);
    console.log(users);
    
    const user = await UserController.getUser(1);
    console.log(user);
    
    const products = await ProductController.listProducts("electronics");
    console.log(products);
}
```

## 依赖项

- `nargo-metadata` - API 元数据提取器
- `nargo-sdk` - TypeScript SDK 生成器
- `nargo-types` - Nargo 类型系统
- `nargo-ir` - Nargo 中间表示
- `anyhow` - 错误处理

## 学习要点

1. 如何从 TypeScript 源代码中提取 API 元数据
2. 如何使用装饰器标记 API 端点
3. 如何生成类型安全的 TypeScript SDK
4. 如何在前端使用生成的 SDK
5. 如何编写元数据提取和 SDK 生成的测试
