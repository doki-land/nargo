# HXO 工具链使用指南

本文档提供了 HXO 框架工具链的详细使用指南，包括各个工具模块的安装、配置和使用方法。

## 1. 工具链模块概览

HXO 框架的工具链由以下模块组成：

| 模块名称 | 功能描述 |
|---------|--------|
| nargo-tools | 命令行工具集，提供构建、格式化、linting 等命令 |
| nargo-bundler | 打包与运行时优化工具，实现按需打包 |
| nargo-formatter | 代码格式化工具，统一代码风格 |
| nargo-linter | 代码检查工具，检测代码质量问题 |
| nargo-compiler | 编译器，将 HXO 代码编译为 JavaScript |
| nargo-parser | 解析器，解析 HXO 代码为中间表示 |
| nargo-transformer | 代码转换器，执行代码优化和转换 |
| nargo-type-check | 类型检查工具，检查类型错误 |
| nargo-ssr | 服务端渲染工具，支持 SSR |
| nargo-hydrate | 客户端 hydration 工具 |
| nargo-git | Git 工具，提供 Git 相关功能 |
| nargo-document | 文档生成工具，生成项目文档 |
| nargo-ir | 中间表示层，为其他工具提供数据结构 |
| nargo-types | 类型定义，提供类型系统支持 |
| nargo-style-processor | 样式处理器，处理 CSS 样式 |
| nargo-script-analyzer | 脚本分析器，分析 JavaScript 代码 |
| nargo-optimizer | 优化器，优化编译产物 |
| nargo-lsp | 语言服务器协议实现，提供编辑器支持 |
| nargo-mcp | 模块通信协议，实现模块间通信 |

## 2. 安装方法

### 2.1 从源码安装

```bash
# 克隆仓库
git clone https://github.com/your-username/nargo.git
cd nargo

# 安装依赖
cargo build --workspace

# 安装命令行工具
cargo install --path compilers/nargo-tools
```

### 2.2 从预编译二进制文件安装

您可以从 GitHub Releases 下载预编译的二进制文件，无需编译即可使用。

## 3. 命令行工具使用

### 3.1 构建命令

```bash
# 基本构建
nargo build

# 指定输入文件
nargo build --input ./src/App.nargo

# 指定输出目录
nargo build --output ./dist

# 生产环境构建
nargo build --prod

# 启用 SSR 模式
nargo build --ssr

# 构建为独立项目
nargo build --standalone
```

### 3.2 格式化命令

```bash
# 格式化单个文件
nargo format ./src/App.nargo

# 格式化整个目录
nargo format ./src
```

### 3.3 代码检查命令

```bash
# 检查单个文件
nargo lint ./src/App.nargo

# 检查整个目录
nargo lint ./src
```

### 3.4 类型检查命令

```bash
# 类型检查
nargo typecheck
```

### 3.5 开发服务器命令

```bash
# 启动开发服务器
nargo dev

# 指定端口
nargo dev --port 3000
```

### 3.6 文档生成命令

```bash
# 生成文档
nargo doc
```

## 4. 配置选项

### 4.1 项目配置文件

在项目根目录创建 `nargo.config.ts` 文件，配置工具链的行为：

```typescript
import { defineConfig } from '@nargo/core';

export default defineConfig({
  formatter: {
    enabled: true,
    indentSize: 4,
    indentType: 'spaces',
    lineWidth: 80,
    singleQuote: true,
    semi: false,
    trailingComma: 'es5'
  },
  linter: {
    enabled: true,
    rules: {
      'no-unused-vars': 'error',
      'no-console': 'warn'
    }
  },
  typecheck: {
    enabled: true,
    strict: true
  }
});
```

### 4.2 命令行配置

大多数命令都支持通过命令行参数进行配置，例如：

```bash
# 配置格式化器
nargo format --indent-size 2 --single-quote false

# 配置 linter
nargo lint --rule no-unused-vars=error --rule no-console=warn
```

## 5. 常见用例

### 5.1 构建项目

```bash
# 开发环境构建
nargo build

# 生产环境构建
nargo build --prod

# 构建为独立项目（包含 HTML）
nargo build --standalone
```

### 5.2 代码格式化

```bash
# 格式化单个文件
nargo format ./src/App.nargo

# 格式化整个目录
nargo format ./src

# 检查格式化（不修改文件）
nargo format --check ./src
```

### 5.3 代码检查

```bash
# 检查单个文件
nargo lint ./src/App.nargo

# 检查整个目录
nargo lint ./src

# 自动修复问题
nargo lint --fix ./src
```

### 5.4 类型检查

```bash
# 类型检查
nargo typecheck

# 严格模式类型检查
nargo typecheck --strict
```

### 5.5 开发服务器

```bash
# 启动开发服务器
nargo dev

# 指定端口
nargo dev --port 3000

# 启用 HTTPS
nargo dev --https
```

## 6. 工具链模块详细使用

### 6.1 nargo-bundler

#### 功能
- 特性分析：深度扫描组件树，识别是否使用了 Signals、Effects、VDOM 或 SSR 等特性
- 定制化运行时生成：根据分析结果，仅打包必要的运行时代码
- 组件聚合：支持将多个 `.nargo` 编译产物合并为单一的可分发文件
- 依赖优化：智能处理组件间的循环依赖和公共模块提取

#### 使用方法

```typescript
import { Bundler } from '@nargo/bundler';

// 创建打包器实例
const bundler = new Bundler({ /* 配置选项 */ });

// 打包模块
const result = await bundler.bundle([
  /* 模块列表 */
]);

// 获取打包结果
console.log(result.outputs);
```

### 6.2 nargo-formatter

#### 功能
- SFC 全局感知：完美识别 HXO 组件结构，支持对不同区块应用针对性的格式化策略
- AST 驱动重构：基于解析后的 IR 进行代码重写，更加稳健且语义准确
- 自定义风格配置：支持通过配置文件自定义缩进、引号、末尾分号等常见风格偏好
- 高性能执行：纯 Rust 实现的格式化引擎，在大规模代码库下依然保持毫秒级的处理速度

#### 使用方法

```typescript
import { NargoFormatter } from '@nargo/formatter';

// 创建格式化器实例
const formatter = new NargoFormatter();

// 格式化代码
const formattedCode = await formatter.format(sourceCode);

// 输出格式化后的代码
console.log(formattedCode);
```

### 6.3 nargo-linter

#### 功能
- 代码质量检查：检测代码中的潜在问题，如未使用的变量、console 语句等
- 可扩展规则：支持自定义规则和插件
- 自动修复：部分问题支持自动修复
- 规则配置：支持通过配置文件启用/禁用规则
- 规则集管理：支持预定义规则集（如 recommended、strict 等）

#### 内置规则

| 规则名称 | 描述 | 默认级别 |
|---------|------|--------|
| no-console | 禁止使用 console.log 等控制台语句 | warning |
| no-debugger | 禁止使用 debugger 语句 | error |
| no-deprecated-tags | 禁止使用已废弃的 HTML 标签 | error |
| no-empty-template | 禁止空模板 | error |
| no-unused-vars | 禁止未使用的变量 | warning |
| no-unreachable-code | 禁止不可达代码 | warning |

#### 规则集

| 规则集名称 | 描述 |
|---------|------|
| recommended | 推荐的规则集，包含常用的代码质量检查规则 |
| strict | 严格的规则集，包含所有规则且级别为 error |

#### 配置示例

```typescript
// nargo.config.ts
import { defineConfig } from '@nargo/core';

export default defineConfig({
  linter: {
    enabled: true,
    extends: 'recommended', // 使用推荐规则集
    rules: {
      'no-console': 'error', // 覆盖默认级别
      'no-unused-vars': 'warn'
    }
  }
});
```

#### 使用方法

```typescript
import { NargoLinter } from '@nargo/linter';
import { NoConsole, NoDeprecatedTags, NoUnusedVars, NoUnreachableCode } from '@nargo/linter/rules';

// 创建 linter 实例
const linter = new NargoLinter();

// 添加规则
linter.addRule(new NoConsole());
linter.addRule(new NoDeprecatedTags());
linter.addRule(new NoUnusedVars());
linter.addRule(new NoUnreachableCode());

// 运行检查
const diagnostics = await linter.run('file.nargo', sourceCode);

// 输出检查结果
console.log(diagnostics);
```

#### 命令行使用

```bash
# 检查单个文件
nargo lint ./src/App.nargo

# 检查整个目录
nargo lint ./src

# 指定配置文件
nargo lint --config nargo.config.json ./src
```

### 6.4 nargo-type-check

#### 功能
- 类型检查：检查代码中的类型错误
- TypeScript 集成：支持 TypeScript 类型定义
- 严格模式：支持严格的类型检查模式

#### 使用方法

```typescript
import { TypeChecker } from '@nargo/type-check';

// 创建类型检查器实例
const typeChecker = new TypeChecker({ /* 配置选项 */ });

// 运行类型检查
const diagnostics = await typeChecker.check('file.nargo', sourceCode);

// 输出检查结果
console.log(diagnostics);
```

### 6.5 nargo-ssr

#### 功能
- 服务端渲染：在服务器端渲染 HXO 组件
- 预取数据：支持在 SSR 过程中预取数据
- 路由支持：集成路由功能，支持服务端路由

#### 使用方法

```typescript
import { renderToString } from '@nargo/ssr';

// 渲染组件为 HTML
const html = await renderToString(component, { /* 配置选项 */ });

// 输出 HTML
console.log(html);
```

## 7. 注意事项

1. **性能优化**：对于大型项目，建议使用 `--prod` 选项进行生产环境构建，以获得最佳性能
2. **类型安全**：启用 `typecheck` 配置，确保代码的类型安全
3. **代码质量**：定期运行 `nargo lint` 命令，确保代码质量
4. **格式化**：使用 `nargo format` 命令统一代码风格，提高代码可读性
5. **依赖管理**：定期更新依赖，确保使用最新版本的工具链

## 8. 故障排除

### 8.1 构建失败

- 检查输入文件路径是否正确
- 检查代码中是否存在语法错误
- 检查依赖是否安装正确

### 8.2 格式化错误

- 检查代码中是否存在语法错误
- 检查配置选项是否正确

### 8.3 代码检查错误

- 检查代码中是否存在未使用的变量或其他问题
- 配置适当的规则级别

### 8.4 类型检查错误

- 检查类型定义是否正确
- 检查类型注解是否完整

## 9. 最佳实践

1. **使用配置文件**：在项目根目录创建 `nargo.config.ts` 文件，统一配置工具链行为
2. **集成到 CI/CD**：在 CI/CD 流程中添加 `nargo lint` 和 `nargo typecheck` 命令，确保代码质量
3. **使用脚本**：在 `package.json` 中添加脚本，方便运行常用命令
4. **定期更新**：定期更新工具链版本，获得最新功能和修复
5. **文档化**：为项目添加文档，说明工具链的使用方法和配置选项

## 10. 总结

HXO 工具链提供了一套完整的工具，帮助开发者提高开发效率和代码质量。通过本文档的指南，您应该能够熟练使用 HXO 工具链的各个模块，为您的项目提供更好的开发体验。

如果您在使用过程中遇到问题，请参考本文档的故障排除部分，或访问 HXO 官方文档获取更多帮助。