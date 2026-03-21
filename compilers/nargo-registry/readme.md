# nargo-registry

Nargo 包管理器的注册表客户端模块。

## 概述

`nargo-registry` 提供 NPM 注册表 API 交互、语义化版本解析、Git 依赖克隆和本地路径依赖解析功能。

## 功能特性

- NPM 注册表 API 交互
- 语义化版本约束匹配
- Git 依赖克隆与检出
- 本地路径依赖解析
- 统一依赖解析接口

## 基本用法

```rust
use nargo_registry::{RegistryClient, DependencyResolver, DependencySpec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RegistryClient::new()?;

    let metadata = client.get_package_metadata("lodash").await?;
    println!("Package: {}", metadata.name);
    println!("Versions: {:?}", metadata.versions.keys().collect::<Vec<_>>());

    let resolver = DependencyResolver::new()?;

    let spec = DependencyResolver::parse_dependency("lodash@4.17.21")?;
    let resolved = resolver.resolve(&spec).await?;

    Ok(())
}
```

## 依赖规范解析

支持多种依赖格式：

| 格式 | 示例 |
|------|------|
| NPM | `lodash@4.17.21` |
| 作用域包 | `@types/node@18.0.0` |
| Git | `git+https://github.com/user/repo.git` |
| GitHub | `github:user/repo` |
| 本地路径 | `file:./local-package` |

## 模块结构

- `client` - 注册表客户端
- `git` - Git 依赖解析器
- `path` - 本地路径解析器
- `semver` - 语义化版本工具
- `types` - 类型定义
