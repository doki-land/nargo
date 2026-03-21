# Nargo 包管理器架构与职责

## Nargo 的定位

Nargo 是一个**现代化的前端包管理器**，参考 Cargo（Rust 的包管理器）设计，为前端项目提供高效、可靠的依赖管理和构建系统。Nargo 无 `node_modules` 和 `package.json`，使用 `Nargo.toml` 作为配置文件，`target/` 目录存放构建产物。

### Nargo 的核心价值

- **简化依赖管理**：通过 `Nargo.toml` 统一管理依赖，避免 `node_modules` 带来的体积膨胀和依赖地狱
- **高性能构建**：集成 HXO 前端工具链，提供基于 Rust 的高性能编译和构建
- **统一的开发体验**：从依赖管理到构建部署的全流程工具链
- **跨平台支持**：支持 Windows、macOS 和 Linux 等主流操作系统

## Nargo 与传统包管理器对比

| 特性 | 传统包管理器 (npm/yarn/pnpm) | Nargo | 优势 |
| :--- | :--- | :--- | :--- |
| **配置文件** | package.json | Nargo.toml | 更简洁的 TOML 格式，支持更灵活的配置 |
| **依赖存储** | node_modules | 全局缓存 + target/ | 避免重复依赖，减少磁盘占用 |
| **依赖解析** | 扁平/嵌套/符号链接 | 严格的版本锁定 | 避免依赖冲突，确保构建一致性 |
| **构建工具** | 需单独配置 | 集成 HXO 工具链 | 一站式解决方案，无需额外配置 |
| **性能** | 较慢的安装和构建 | 基于 Rust 的高性能实现 | 安装和构建速度提升 10-100 倍 |

## Nargo 的核心架构

### 1. 核心组件

- **nargo-tools**：命令行工具，提供用户交互界面
- **nargo-core**：核心依赖管理逻辑，包括依赖解析、版本管理和安装
- **nargo-build**：构建系统，集成 HXO 前端工具链
- **nargo-registry**：包注册表客户端，负责包的发布和下载
- **nargo-lock**：依赖锁定机制，确保构建的可重现性

### 2. 工作流程

1. **依赖解析**：解析 `Nargo.toml` 中的依赖声明
2. **版本锁定**：生成 `Nargo.lock` 文件，锁定依赖版本
3. **依赖安装**：从注册表下载依赖到全局缓存
4. **构建过程**：调用 HXO 工具链编译和打包项目
5. **产物输出**：将构建产物输出到 `target/` 目录

## Nargo.toml 配置示例

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "A sample Nargo project"

[dependencies]
react = "^18.2.0"

[dev-dependencies]
nargo = "^1.0.0"
```

## HXO 集成

Nargo 集成了 HXO 前端工具链，为前端开发提供完整的编译、构建和测试能力：

- **nargo-compiler**：基于 Rust 的高性能编译器
- **nargo-bundler**：打包构建工具，支持热模块替换
- **nargo-tools**：开发工具集合，包含代码检查和类型检查
- **nargo-testing**：测试工具，与 HXO 运行时深度集成
- **nargo-style**：CSS 处理工具，提供编译期 CSS 优化

HXO 作为 Nargo 的构建工具链，负责将前端代码转化为高效的可执行代码，而 Nargo 则负责依赖管理和构建流程的协调。

## 命令行工具

Nargo 提供了丰富的命令行工具，包括：

- `nargo init`：初始化新项目
- `nargo add`：添加依赖
- `nargo remove`：移除依赖
- `nargo install`：安装依赖
- `nargo build`：构建项目
- `nargo test`：运行测试
- `nargo run`：运行脚本
- `nargo publish`：发布包
- `nargo format`：格式化代码
- `nargo lint`：检查代码质量
- `nargo typecheck`：检查类型错误
- `nargo dev`：启动开发服务器

## 未来发展

Nargo 将继续发展和完善，提供更强大、更高效的前端包管理和构建系统。未来的发展方向包括：

1. 进一步优化依赖解析和安装速度
2. 增强包注册表的功能和性能
3. 扩展插件系统，支持更多的构建场景
4. 完善跨平台支持，确保在所有主流操作系统上的一致性
5. 提供更丰富的命令行工具和配置选项
6. 增强与 HXO 工具链的集成，提供更完整的前端开发体验

Nargo 将始终保持开放、灵活和高效的特性，为前端开发提供最佳的包管理和构建支持。