# Nargo 基础使用示例

这个示例展示了如何在项目中使用 Nargo 的基础功能。

## 功能演示

1. **配置构建器** - 使用 `ConfigBuilder` 创建和配置 Nargo 配置
2. **路径操作** - 基础的文件路径处理操作

## 项目结构

```
nargo-basic/
├── Cargo.toml          # 项目配置文件
├── README.md           # 说明文档
└── src/
    └── main.rs         # 主程序代码
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

## 代码说明

### 配置构建器示例

```rust
let config = ConfigBuilder::new()
    .package_name("my-app")
    .package_version("0.1.0")
    .build()?;
```

### 路径操作示例

```rust
let current_dir = std::env::current_dir()?;
let src_path = Path::new("src");
```

## 依赖项

- `nargo-types` - Nargo 类型系统
- `nargo-config` - Nargo 配置管理
- `anyhow` - 错误处理
- `thiserror` - 自定义错误类型

## 学习要点

1. 如何使用 `ConfigBuilder` 构建配置
2. 基础的文件路径操作
3. 错误处理模式
4. 测试编写
