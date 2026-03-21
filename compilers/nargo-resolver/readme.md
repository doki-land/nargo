# nargo-resolver

Nargo 包管理器的依赖解析模块。

## 概述

`nargo-resolver` 提供依赖解析、冲突检测和拓扑排序功能，用于解析项目依赖关系图。

## 功能特性

- 依赖图构建
- 版本冲突检测
- 拓扑排序
- 多源依赖支持（注册表、Git、本地路径）

## 基本用法

```rust
use nargo_resolver::{Resolver, ResolveOptions};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ResolveOptions {
        include_dev: true,
        include_optional: true,
        allow_prerelease: false,
        max_depth: 100,
        registry: "https://registry.npmjs.org".to_string(),
    };

    let mut resolver = Resolver::with_options(options);

    let mut dependencies = HashMap::new();
    dependencies.insert(
        "lodash".to_string(),
        nargo_config::Dependency::Version("4.17.21".to_string()),
    );

    let result = resolver.resolve(&dependencies, &HashMap::new()).await?;

    println!("Total packages: {}", result.stats.total_packages);
    println!("Conflicts: {}", result.conflicts.len());

    for solution in &result.solutions {
        println!("Suggested: {:?}", solution);
    }

    Ok(())
}
```

## 解析结果

`ResolveResult` 包含：

- `graph` - 依赖关系图
- `conflicts` - 检测到的冲突列表
- `solutions` - 冲突解决方案建议
- `stats` - 解析统计信息

## 模块结构

- `resolver` - 主解析器实现
- `graph` - 依赖图数据结构
- `conflict` - 冲突检测与解决
