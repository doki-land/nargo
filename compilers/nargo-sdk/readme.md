# nargo-sdk

TypeScript 前端 SDK 生成器，基于 API 元数据自动生成类型安全的 SDK。

## 功能

- 自动生成 TypeScript 类型定义
- 生成类型安全的 API 客户端
- 支持控制器分组
- 包含完整的元数据信息
- 可与 @nargo/client 配合使用

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
nargo-sdk = { path = "../nargo-sdk" }
```

## 使用方法

### 基本使用

```rust
use nargo_metadata::{ApiMetadataExtractor, ApiMetadata};
use nargo_sdk::SdkGenerator;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let source = r#"
class UserController {
    @http("GET", "/users")
    async listUsers(@Query page: number = 1): Promise<User[]> {
    }

    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
    }
}

interface User {
    id: number;
    name: string;
    email: string;
}
"#;
    
    let ir = nargo_ir::IRModule::default();
    let metadata = ApiMetadataExtractor::extract(source, &ir)?;
    
    let output_dir = Path::new("./output");
    SdkGenerator::generate(&metadata, output_dir)?;
    
    println!("SDK generated successfully!");
    
    Ok(())
}
```

### 生成的文件

SdkGenerator 会生成两个文件：

1. **types.ts** - 所有的 TypeScript 类型定义
2. **api.ts** - API 客户端实现

## 生成的代码示例

### 类型定义 (types.ts)

```typescript
export interface User {
  id: number;
  name: string;
  email: string;
}

export interface CreateUserDto {
  name: string;
  email: string;
  password: string;
}
```

### API 客户端 (api.ts)

```typescript
/* eslint-disable */
// @ts-nocheck
import { useClient } from '@nargo/client';
import type * as Types from './types';

export const UserController = {
  async listUsers(page: number): Promise<Types.User[]> {
    const client = useClient();
    return client.get(`/users`, { params: { page } });
  },

  async getUser(id: number): Promise<Types.User> {
    const client = useClient();
    return client.get(`/users/${id}`, { params: {} });
  },
};

(UserController.listUsers as any)._nargo_id = 'UserController.listUsers';
(UserController.listUsers as any).url = '/users';
(UserController.listUsers as any).method = 'GET';

(UserController.getUser as any)._nargo_id = 'UserController.getUser';
(UserController.getUser as any).url = '/users/:id';
(UserController.getUser as any).method = 'GET';
```

## 在前端使用

### 安装依赖

```bash
npm install @nargo/client
```

### 使用 SDK

```typescript
import { UserController } from './sdk/api';

async function example() {
    const users = await UserController.listUsers(1);
    console.log(users);
    
    const user = await UserController.getUser(1);
    console.log(user);
}
```

## API

### SdkGenerator

主要的 SDK 生成器：

```rust
pub struct SdkGenerator;

impl SdkGenerator {
    pub fn generate(metadata: &ApiMetadata, output_dir: &Path) -> Result<()>;
}
```

## 数据结构

生成的 API 端点包含以下元数据：

- `_nargo_id` - 唯一标识符（格式：`ControllerName.methodName`）
- `url` - API 路径
- `method` - HTTP 方法

这些元数据可以用于调试、工具集成等。

## 完整示例

### 后端代码

```typescript
class UserController {
    @http("GET", "/users")
    async listUsers(@Query page: number = 1, @Query limit: number = 10): Promise<User[]> {
        // 实现
    }

    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
        // 实现
    }

    @http("POST", "/users")
    async createUser(@Body user: CreateUserDto): Promise<User> {
        // 实现
    }

    @http("PUT", "/users/:id")
    async updateUser(@Path id: number, @Body user: UpdateUserDto): Promise<User> {
        // 实现
    }

    @http("DELETE", "/users/:id")
    async deleteUser(@Path id: number): Promise<void> {
        // 实现
    }
}

interface User {
    id: number;
    name: string;
    email: string;
    createdAt: Date;
}

interface CreateUserDto {
    name: string;
    email: string;
    password: string;
}

interface UpdateUserDto {
    name?: string;
    email?: string;
}
```

### 生成的 SDK

```typescript
import { UserController } from './sdk/api';

const users = await UserController.listUsers(1, 10);
const user = await UserController.getUser(1);
const newUser = await UserController.createUser({ name: "Test", email: "test@example.com", password: "secret" });
const updatedUser = await UserController.updateUser(1, { name: "Updated" });
await UserController.deleteUser(1);
```

## 开发

### 运行测试

```bash
cargo test
```

### 构建

```bash
cargo build --release
```

## 许可证

MIT
