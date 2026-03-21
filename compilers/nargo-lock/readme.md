# nargo-lock

Nargo 包管理器的锁文件管理模块。

## 概述

`nargo-lock` 提供锁文件的生成、解析和验证功能，确保可重复构建。锁文件记录了所有依赖的精确版本和完整性信息。

## 功能特性

- 锁文件生成与解析（TOML 格式）
- 依赖版本锁定
- 完整性哈希验证
- 锁文件验证

## 基本用法

```rust
use nargo_lock::{LockFile, LockEntry};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let mut lock = LockFile::new();

    let entry = LockEntry::new("vue", "3.4.0")
        .with_source("https://registry.npmjs.org/vue/-/vue-3.4.0.tgz")
        .with_integrity("sha512-abc123");

    lock.add_package(entry);

    lock.save("./nargo.lock")?;

    let loaded = LockFile::load("./nargo.lock")?;
    if let Some(pkg) = loaded.get_package("vue", "3.4.0") {
        println!("Package: {}@{}", pkg.name, pkg.version);
    }

    Ok(())
}
```

## 锁文件格式

锁文件使用 TOML 格式，包含以下信息：

- 元数据（版本、生成时间等）
- 包名称和版本
- 来源 URL 或路径
- 完整性哈希（SRI 格式）
- 依赖关系

## 模块结构

- `lock` - 锁文件数据结构与操作
- `verify` - 锁文件验证器
