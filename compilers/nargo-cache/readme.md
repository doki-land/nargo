# nargo-cache

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20cache%20management%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Cache Logo" width="150" height="150">
  <h3>💾 Nargo 包管理器的全局缓存管理模块</h3>
  <p>提供高效的包缓存功能，避免重复下载，提升安装效率</p>
</div>

## 🎯 简介

`nargo-cache` 是 Nargo 包管理器的核心组件，负责管理已下载包的缓存功能，包括完整性验证和缓存清理。该模块通过本地存储管理包的 tarball 文件，避免重复下载，提升安装效率，为 Nargo 的快速安装和构建提供支持。

## ✨ 核心特性

### 高效缓存管理
- **本地缓存存储**: 安全存储包的 tarball 文件，避免重复下载
- **智能缓存策略**: 基于包名、版本和注册表的缓存键策略
- **缓存过期管理**: 支持设置缓存过期时间，自动清理过期缓存
- **多注册表支持**: 支持多个包注册表的缓存管理

### 完整性验证
- **多种哈希算法**: 支持 SHA-256、SHA-512 等哈希算法
- **完整性校验**: 确保缓存的包文件完整无误
- **防篡改保护**: 防止缓存文件被篡改，确保包的安全性

### 缓存维护
- **缓存统计**: 提供详细的缓存统计信息，包括包数量和占用空间
- **缓存清理**: 支持按大小、时间等条件清理缓存
- **缓存验证**: 验证缓存的完整性，修复损坏的缓存
- **增量更新**: 支持缓存的增量更新，只更新变更的部分

### 性能优化
- **并行操作**: 支持并行的缓存读写操作，提高性能
- **内存优化**: 高效的内存使用，支持处理大量包的缓存
- **磁盘空间管理**: 智能管理磁盘空间，避免缓存占用过多空间
- **快速检索**: 高效的缓存检索算法，快速找到所需的包

## 🚀 使用方法

### 命令行使用

```bash
# 查看缓存统计信息
nargo cache stats

# 清理缓存
nargo cache clean

# 清理特定包的缓存
nargo cache clean --package lodash

# 清理超过指定大小的缓存
nargo cache clean --size 1GB

# 清理超过指定时间的缓存
nargo cache clean --time 30d

# 验证缓存完整性
nargo cache verify
```

### API 使用

```rust
use nargo_cache::{Cache, CacheEntry, CacheStats};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建缓存实例
    let mut cache = Cache::new()?;

    // 添加包到缓存
    let tarball = b"fake tarball content";
    let entry = cache.add(
        "lodash",
        "4.17.21",
        tarball,
        "https://registry.npmjs.org"
    ).await?;

    // 检查包是否在缓存中
    if cache.has("lodash", "4.17.21") {
        // 获取缓存的 tarball
        let cached = cache.get_tarball("lodash", "4.17.21").await?;
        println!("Cached tarball size: {} bytes", cached.len());
    }

    // 获取缓存统计信息
    let stats = cache.stats();
    println!("Total packages: {}", stats.total_packages);
    println!("Total size: {} bytes", stats.total_size);

    // 清理缓存
    cache.clean().await?;

    Ok(())
}
```

## 🔧 配置选项

`nargo-cache` 支持通过 `nargo.config.toml` 文件进行配置：

```toml
[tool.nargo.cache]
# 缓存目录路径
cache_dir = "~/.nargo/cache"

# 缓存大小限制
max_size = "10GB"

# 缓存过期时间
max_age = "30d"

# 完整性验证算法
algorithm = "sha256"

# 并行操作数量
parallelism = 4

# 清理策略
[tool.nargo.cache.clean]
enable_auto_clean = true
clean_interval = "7d"
min_free_space = "5GB"
```

## 📊 与其他缓存工具对比

| 特性 | nargo-cache | npm cache | yarn cache | pnpm store |
|------|-------------|-----------|------------|------------|
| 多注册表支持 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 完整性验证 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |
| 缓存统计 | ✅ (完整) | ✅ (有限) | ✅ (有限) | ✅ (有限) |
| 智能清理 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 并行操作 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 内存优化 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 多平台支持 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |

## 🎯 应用场景

### 包管理器缓存
在 Nargo 包管理器中，`nargo-cache` 作为核心组件，提供高效的包缓存功能，避免重复下载，提升安装速度。

### 持续集成/持续部署
在 CI/CD 环境中，`nargo-cache` 可以缓存依赖包，减少构建时间，提高 CI/CD 效率。

### 多项目共享缓存
在多个项目中共享同一个缓存，减少磁盘空间使用，提高资源利用率。

### 离线环境支持
在离线环境中，`nargo-cache` 可以提供已缓存的包，支持离线安装和构建。

## 🔧 核心 API

### 基本用法

```rust
use nargo_cache::{Cache, CacheOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建缓存实例
    let options = CacheOptions {
        cache_dir: Some("~/.nargo/cache".into()),
        max_size: Some("10GB".into()),
        max_age: Some("30d".into()),
        ..Default::default()
    };
    
    let mut cache = Cache::with_options(options)?;

    // 添加包到缓存
    let tarball = b"package content";
    let entry = cache.add(
        "package-name",
        "1.0.0",
        tarball,
        "https://registry.npmjs.org"
    ).await?;

    // 检查包是否在缓存中
    if cache.has("package-name", "1.0.0") {
        println!("Package is in cache");
    }

    // 获取缓存的 tarball
    let cached_tarball = cache.get_tarball("package-name", "1.0.0").await?;

    // 获取缓存统计信息
    let stats = cache.stats();
    println!("Total packages: {}", stats.total_packages);
    println!("Total size: {} bytes", stats.total_size);

    // 清理缓存
    cache.clean().await?;

    Ok(())
}
```

### 高级用法

```rust
use nargo_cache::{Cache, CacheOptions, CleanOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建缓存实例
    let options = CacheOptions {
        cache_dir: Some("~/.nargo/cache".into()),
        max_size: Some("5GB".into()),
        algorithm: "sha512".into(),
        ..Default::default()
    };
    
    let mut cache = Cache::with_options(options)?;

    // 清理特定条件的缓存
    let clean_options = CleanOptions {
        package: Some("lodash".into()),
        size: Some("1GB".into()),
        time: Some("7d".into()),
        ..Default::default()
    };
    
    cache.clean_with_options(&clean_options).await?;

    // 验证缓存完整性
    let validation_result = cache.verify().await?;
    println!("Valid packages: {}", validation_result.valid);
    println!("Invalid packages: {}", validation_result.invalid);

    Ok(())
}
```

## 🏗️ 模块结构

### cache
缓存管理核心实现，负责缓存的添加、获取、删除等操作：
- **缓存操作**: 添加、获取、删除缓存
- **缓存查询**: 检查缓存是否存在
- **缓存统计**: 获取缓存统计信息
- **缓存清理**: 清理缓存

### storage
存储后端实现，负责缓存的持久化存储：
- **文件系统存储**: 将缓存存储在文件系统中
- **存储管理**: 管理存储的组织结构
- **存储优化**: 优化存储结构，提高存取效率

### integrity
完整性校验工具，负责验证缓存的完整性：
- **哈希计算**: 计算文件的哈希值
- **完整性验证**: 验证文件的完整性
- **防篡改保护**: 防止缓存文件被篡改

## 🔗 相关项目

- [nargo-registry](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-registry): 包注册表客户端，与缓存模块配合使用
- [nargo-resolver](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-resolver): 依赖解析器，使用缓存模块提高解析效率
- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 提供类型定义和工具函数
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): Nargo 的命令行工具集合

## 📚 文档

- **[官方文档](https://docs.nargo.dev/cache)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/cache/api)** - 完整的 API 参考
- **[最佳实践](https://docs.nargo.dev/cache/best-practices)** - 缓存管理最佳实践指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)
