# nargo-config

<div align="center">
  <img src="https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=modern%20config%20management%20tool%20logo%20with%20blue%20and%20orange%20gradient%2C%20minimalist%2C%20professional&image_size=square_hd" alt="Nargo Config Logo" width="150" height="150">
  <h3>⚙️ Nargo 框架的配置管理模块</h3>
  <p>用于读取和处理 Nargo 项目的配置文件，提供类型安全的配置结构定义</p>
</div>

## 🎯 简介

`nargo-config` 是 Nargo 框架的核心配置管理模块，负责读取和处理 Nargo 项目的配置文件。它支持多种配置文件格式，提供类型安全的配置结构定义，并与 Nargo 工具链深度集成，为 Nargo 项目提供统一、灵活的配置管理方案。

## ✨ 核心特性

### 多格式支持
- **TypeScript 配置**: 支持 `nargo.config.ts` 配置文件，提供类型安全的配置定义
- **JSON 配置**: 支持 `nargo.config.json` 配置文件，提供简单直接的配置方式
- **TOML 配置**: 支持 `nargo.config.toml` 配置文件，提供简洁易读的配置格式

### 智能执行
- **Node.js 版本适配**: 根据 Node.js 版本自动选择执行 TypeScript 配置文件的方式
  - Node.js 22+：直接运行 TypeScript 文件（使用 `--experimental-strip-types` 选项）
  - Node.js < 22：将 TypeScript 转换为 JavaScript 并执行
- **跨平台支持**: 支持 Windows、macOS、Linux 等多种平台
- **环境变量支持**: 支持在配置文件中使用环境变量，提高配置的灵活性

### 类型安全
- **Rust 类型定义**: 提供完整的 Rust 类型定义，确保配置的类型安全
- **TypeScript 类型定义**: 提供 TypeScript 类型定义，确保配置文件的类型安全
- **配置验证**: 自动验证配置的完整性和有效性，提供详细的错误信息

### 工具链集成
- **深度集成**: 与 Nargo 工具链深度集成，为所有 Nargo 工具提供统一的配置管理
- **插件支持**: 支持插件配置，扩展 Nargo 的功能
- **路径别名**: 支持路径别名配置，简化模块导入

### 高级功能
- **配置继承**: 支持配置继承，减少配置重复
- **配置合并**: 支持多个配置文件的合并，提高配置的灵活性
- **配置迁移**: 支持旧版本配置的自动迁移，确保向后兼容
- **配置导出**: 支持配置的导出和分享，便于团队协作

## 🚀 使用方法

### 命令行使用

```bash
# 查看当前配置
nargo config show

# 导出配置
nargo config export

# 验证配置
nargo config validate

# 迁移配置
nargo config migrate
```

### API 使用

```rust
use nargo_config::{ConfigLoader, NargoConfig};
use std::path::PathBuf;

// 查找配置文件
let config_path = ConfigLoader::find_config_file(&PathBuf::from(".")).unwrap();

// 创建配置加载器
let loader = ConfigLoader::new(config_path);

// 加载配置
let config = loader.load().unwrap();

// 使用配置
println!("Project name: {:?}", config.name);
println!("Build config: {:?}", config.build);
println!("Format config: {:?}", config.format);
println!("Lint config: {:?}", config.lint);
```

## 🔧 配置文件示例

### TypeScript 配置文件 (`nargo.config.ts`)

```typescript
import { NargoConfig } from 'nargo-config';

const config: NargoConfig = {
  name: 'my-project',
  version: '1.0.0',
  build: {
    out_dir: 'dist',
    prod: true,
    sourcemap: false,
    browserslist: ['> 0.5%', 'last 2 versions', 'not dead'],
    targets: ['es2015']
  },
  dev: {
    port: 3000,
    host: 'localhost',
    hot: true,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true
      }
    }
  },
  plugins: [
    {
      name: 'nargo-plugin-example',
      options: {
        enabled: true,
        debug: false
      }
    }
  ],
  aliases: {
    '@': './src',
    '@components': './src/components'
  },
  format: {
    indent_size: 2,
    indent_type: 'Spaces',
    line_width: 80,
    single_quote: true,
    semi: false,
    trailing_comma: 'Es5',
    attr_spacing: true,
    self_closing_space: true,
    tag_spacing: true,
    arrow_parens: true,
    object_spacing: true,
    array_spacing: true,
    function_spacing: true,
    block_spacing: true
  },
  lint: {
    rules: {
      'no-console': {
        enabled: true,
        severity: 'Warning'
      }
    },
    extends: 'recommended'
  }
};

// 注意：配置文件必须输出配置对象
console.log(JSON.stringify(config));
```

### TOML 配置文件 (`nargo.config.toml`)

```toml
name = "my-project"
version = "1.0.0"

[build]
out_dir = "dist"
prod = true
sourcemap = false
browserslist = ["> 0.5%", "last 2 versions", "not dead"]
targets = ["es2015"]

[dev]
port = 3000
host = "localhost"
hot = true

[dev.proxy."/api"]
target = "http://localhost:8080"
changeOrigin = true

[[plugins]]
name = "nargo-plugin-example"

[plugins.options]
enabled = true
debug = false

[aliases]
"@" = "./src"
"@components" = "./src/components"

[format]
indent_size = 2
indent_type = "Spaces"
line_width = 80
single_quote = true
semi = false
trailing_comma = "Es5"
attr_spacing = true
self_closing_space = true
tag_spacing = true
arrow_parens = true
object_spacing = true
array_spacing = true
function_spacing = true
block_spacing = true

[lint]
extends = "recommended"

[lint.rules."no-console"]
enabled = true
severity = "Warning"
```

### JSON 配置文件 (`nargo.config.json`)

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "build": {
    "out_dir": "dist",
    "prod": true,
    "sourcemap": false,
    "browserslist": ["> 0.5%", "last 2 versions", "not dead"],
    "targets": ["es2015"]
  },
  "dev": {
    "port": 3000,
    "host": "localhost",
    "hot": true,
    "proxy": {
      "/api": {
        "target": "http://localhost:8080",
        "changeOrigin": true
      }
    }
  },
  "plugins": [
    {
      "name": "nargo-plugin-example",
      "options": {
        "enabled": true,
        "debug": false
      }
    }
  ],
  "aliases": {
    "@": "./src",
    "@components": "./src/components"
  },
  "format": {
    "indent_size": 2,
    "indent_type": "Spaces",
    "line_width": 80,
    "single_quote": true,
    "semi": false,
    "trailing_comma": "Es5",
    "attr_spacing": true,
    "self_closing_space": true,
    "tag_spacing": true,
    "arrow_parens": true,
    "object_spacing": true,
    "array_spacing": true,
    "function_spacing": true,
    "block_spacing": true
  },
  "lint": {
    "rules": {
      "no-console": {
        "enabled": true,
        "severity": "Warning"
      }
    },
    "extends": "recommended"
  }
}
```

## 📊 与其他配置工具对比

| 特性 | nargo-config | Vite config | Webpack config | Rollup config |
|------|-------------|-------------|----------------|---------------|
| 多格式支持 | ✅ (TS, JSON, TOML) | ✅ (TS, JS, JSON) | ✅ (JS, JSON) | ✅ (JS, JSON) |
| 类型安全 | ✅ (完整) | ✅ (通过 TypeScript) | ❌ (有限) | ❌ (有限) |
| 环境变量支持 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (通过插件) |
| 配置继承 | ✅ (完整) | ❌ (有限) | ❌ (有限) | ❌ (有限) |
| 配置迁移 | ✅ (完整) | ❌ (不支持) | ❌ (不支持) | ❌ (不支持) |
| 工具链集成 | ✅ (深度) | ✅ (深度) | ✅ (深度) | ✅ (深度) |
| 插件支持 | ✅ (完整) | ✅ (完整) | ✅ (完整) | ✅ (完整) |

## 🎯 应用场景

### 项目配置管理
在 Nargo 项目中，`nargo-config` 作为核心配置管理模块，为所有 Nargo 工具提供统一的配置管理，确保项目配置的一致性和可维护性。

### 多环境配置
在多环境部署中，`nargo-config` 支持通过环境变量和配置继承，为不同环境提供不同的配置，简化多环境部署的配置管理。

### 团队协作
在团队协作中，`nargo-config` 提供类型安全的配置定义，确保团队成员使用一致的配置格式，减少配置错误。

### 插件开发
在插件开发中，`nargo-config` 提供插件配置支持，允许插件定义自己的配置选项，扩展 Nargo 的功能。

## 🔧 核心 API

### ConfigLoader
配置加载器，负责查找和加载配置文件：
- **find_config_file**: 查找配置文件
- **new**: 创建新的配置加载器
- **load**: 加载配置
- **load_from_str**: 从字符串加载配置
- **validate**: 验证配置

### NargoConfig
配置的核心数据结构，包含项目的所有配置：
- **name**: 项目名称
- **version**: 项目版本
- **build**: 构建配置
- **dev**: 开发服务器配置
- **plugins**: 插件配置
- **aliases**: 路径别名配置
- **format**: 格式化配置
- **lint**: lint 配置

### BuildConfig
构建配置，控制构建过程的各个方面：
- **out_dir**: 输出目录
- **prod**: 是否启用生产模式
- **sourcemap**: 是否启用源码映射
- **browserslist**: 目标浏览器配置
- **targets**: 构建目标

### DevConfig
开发服务器配置，控制开发服务器的行为：
- **port**: 开发服务器端口
- **host**: 开发服务器主机
- **hot**: 是否启用热更新
- **proxy**: 代理配置

### PluginConfig
插件配置，定义插件的名称和选项：
- **name**: 插件名称
- **options**: 插件选项

## 🏗️ 模块结构

### loader
配置加载器的实现，负责查找和加载配置文件：
- **ConfigLoader**: 配置加载器
- **find_config_file**: 查找配置文件的逻辑
- **load_config**: 加载配置的逻辑

### types
配置类型的定义，提供类型安全的配置结构：
- **NargoConfig**: 主配置结构
- **BuildConfig**: 构建配置结构
- **DevConfig**: 开发服务器配置结构
- **PluginConfig**: 插件配置结构
- **FormatConfig**: 格式化配置结构
- **LintConfig**: lint 配置结构

### validation
配置验证的实现，确保配置的完整性和有效性：
- **validate_config**: 验证配置的逻辑
- **validate_build_config**: 验证构建配置
- **validate_dev_config**: 验证开发服务器配置

### migrate
配置迁移的实现，支持旧版本配置的自动迁移：
- **migrate_config**: 迁移配置的逻辑
- **migrate_v1_to_v2**: 从 v1 版本迁移到 v2 版本

## 🔗 相关项目

- [nargo-types](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-types): 基础类型定义
- [nargo-tools](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-tools): 命令行工具集成
- [nargo-compiler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-compiler): 核心编译引擎
- [nargo-bundler](https://github.com/nargo-js/nargo/tree/main/compilers/nargo-bundler): 代码打包器

## 📚 文档

- **[官方文档](https://docs.nargo.dev/config)** - 详细的使用文档
- **[API 参考](https://docs.nargo.dev/config/api)** - 完整的 API 参考
- **[配置指南](https://docs.nargo.dev/config/guide)** - 配置最佳实践指南
- **[插件开发](https://docs.nargo.dev/config/plugins)** - 插件配置开发指南

## 🤝 贡献

我们欢迎所有形式的贡献！请查看 [贡献指南](https://github.com/nargo-js/nargo/blob/main/CONTRIBUTING.md) 了解如何开始。

## ⚖️ License

MIT.

## 📞 联系我们

如有任何问题或建议，欢迎通过以下方式联系我们：

- **Email**: team@nargo.dev
- **GitHub**: [nargo-js/nargo](https://github.com/nargo-js/nargo)