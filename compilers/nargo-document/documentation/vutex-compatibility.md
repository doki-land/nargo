# VuTeX 兼容性说明

nargo-document 与 VuTeX 具有高度的兼容性，可以无缝处理 VuTeX 格式的文档。

## 兼容性概述

| 特性 | VuTeX | nargo-document | 兼容性 |
|------|--------|----------------|--------|
| 标准 Markdown | ✅ | ✅ | 完全兼容 |
| Frontmatter | ✅ | ✅ | 完全兼容 |
| KaTeX 数学公式 | ✅ | ✅ | 完全兼容 |
| Mermaid 图表 | ✅ | ✅ | 完全兼容 |
| 代码高亮 | ✅ | ✅ | 完全兼容 |
| 自定义容器 | ✅ | ✅ | 完全兼容 |
| 脚注 | ✅ | ✅ | 完全兼容 |
| 任务列表 | ✅ | ✅ | 完全兼容 |

## 如何使用

### 方法 1：直接使用 VuTeX 文档

nargo-document 可以直接处理 VuTeX 格式的 Markdown 文档，无需任何修改：

```bash
# 使用 nargo-document 处理 VuTeX 文档
nargo-document generate --input ./vutex-docs --output ./output
```

### 方法 2：转换配置

如果你有 VuTeX 的配置文件，可以轻松转换为 nargo-document 格式：

**VuTeX 配置 (vutex.config.ts):**
```typescript
import { defineConfig } from '@vutex/core'

export default defineConfig({
  title: '我的文档',
  description: '文档描述',
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '指南', link: '/guide/' }
    ]
  }
})
```

**nargo-document 配置 (nargodoc.config.toml):**
```toml
[general]
title = "我的文档"
description = "文档描述"

[theme_config.nav]
[[theme_config.nav.items]]
text = "首页"
link = "/"

[[theme_config.nav.items]]
text = "指南"
link = "/guide/"
```

## 测试兼容性

我们提供了一个完整的兼容性测试文档：[vutex-compatibility.md](../examples/vutex-compatibility.md)

你可以使用这个文档来测试 nargo-document 对 VuTeX 格式的支持：

```bash
# 生成测试文档
cd examples
cp vutex-compatibility.md test.md
nargo-document generate --input . --output ./output
```

## 差异说明

虽然两者高度兼容，但仍有一些小的差异需要注意：

### 配置格式

- VuTeX 使用 TypeScript 配置文件
- nargo-document 使用 TOML 配置文件

### 插件系统

两者的插件 API 略有不同，但功能类似。

## 迁移指南

如果你想从 VuTeX 迁移到 nargo-document：

1. **复制文档内容** - Markdown 文档可以直接复制
2. **转换配置文件** - 将 TypeScript 配置转换为 TOML 格式
3. **调整插件** - 根据需要调整插件配置
4. **测试生成** - 运行 nargo-document 生成文档并验证

## 示例

查看 [示例目录](../examples/) 中的文件：

- [vutex-compatibility.md](../examples/vutex-compatibility.md) - 完整的兼容性测试文档
- [nargodoc.config.toml](../examples/nargodoc.config.toml) - 示例配置文件
