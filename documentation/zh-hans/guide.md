# Nargo 使用指南

## 1. 快速上手

### 1.1 安装 Nargo

Nargo 是一个独立的命令行工具，不需要依赖 node.js。你可以通过以下方式安装：

```bash
# 使用 Cargo 安装（推荐）
cargo install nargo

# 或从 GitHub Releases 下载预编译二进制文件
```

### 1.2 初始化项目

使用 Nargo 初始化一个新的前端项目：

```bash
# 创建新项目
nargo init my-app

# 进入项目目录
cd my-app
```

### 1.3 配置 Nargo

Nargo 项目的配置文件是 `Nargo.toml`，位于项目根目录：

```toml
# Nargo.toml
[package]
name = "my-app"
version = "0.1.0"
description = "My Nargo Application"

[dependencies]
react = "^18.2.0"
react-dom = "^18.2.0"
typescript = "^5.0.0"

[build]
entry = "./src/main.tsx"
output = "./dist"
```

### 1.4 运行 Nargo

```bash
# 开发模式（带 HMR）
nargo dev

# 构建生产版本
nargo build

# 运行测试
nargo test

# 安装依赖
nargo add react

# 移除依赖
nargo remove react

# 更新依赖
nargo update

# 格式化代码
nargo format

# 检查代码质量
nargo lint

# 检查类型错误
nargo typecheck
```

## 2. 核心概念

### 2.1 包管理

Nargo 的核心创新在于其包管理系统，完全替代了传统的 npm/pnpm 包管理方式：

- **无 node_modules**：使用符号链接和路径映射，完全移除 node_modules 目录
- **依赖锁定**：自动生成 `nargo.lock` 文件，确保依赖版本一致性
- **包缓存**：全局缓存已下载的包，提高安装速度
- **多注册表支持**：支持 npm 注册表、GitHub Packages 等多个包源

### 2.2 组件系统

Nargo 支持多种组件格式，包括：

- **.nargo** 文件：Nargo 原生组件格式，包含模板、脚本和样式
- **.tsx**/.jsx 文件：React 组件
- **.vue** 文件：Vue 组件（需要安装相应插件）

### 2.3 响应式系统

Nargo 提供了基于信号（Signal）的响应式系统：

```typescript
import { signal, effect, computed } from '@nargo/core';

// 创建信号
const [count, setCount] = signal(0);

// 创建计算属性
const doubled = computed(() => count() * 2);

// 创建副作用
effect(() => {
  console.log(`Count: ${count()}, Doubled: ${doubled()}`);
});

// 更新信号
setCount(1); // 输出: Count: 1, Doubled: 2
setCount(2); // 输出: Count: 2, Doubled: 4
```

### 2.4 模板语法

Nargo 模板语法与 Vue 类似，支持以下特性：

- **插值**：`{{ expression }}`
- **指令**：`v-if`, `v-for`, `v-bind`, `v-on` 等
- **事件处理**：`@click`, `@input` 等
- **计算属性**：在模板中直接使用计算属性

## 3. 项目结构

推荐的 Nargo 项目结构：

```
├── src/
│   ├── components/       # 组件
│   │   ├── Counter.tsx
│   │   └── TodoList.nargo
│   ├── pages/            # 页面
│   │   ├── Home.tsx
│   │   └── About.nargo
│   ├── utils/            # 工具函数
│   │   └── helpers.ts
│   ├── styles/           # 全局样式
│   │   └── global.css
│   ├── main.tsx          # 入口文件
│   └── App.tsx           # 根组件
├── public/               # 静态资源
├── nargo.toml            # Nargo 配置
├── nargo.lock            # 依赖锁定文件
└── tsconfig.json         # TypeScript 配置
```

## 4. 开发流程

### 4.1 创建组件

#### React 组件 (.tsx)

```tsx
// src/components/Counter.tsx
import { useState } from 'react';

export default function Counter() {
  const [count, setCount] = useState(0);
  
  return (
    <div className="counter">
      <h1>Counter</h1>
      <p>Count: {count}</p>
      <button onClick={() => setCount(count + 1)}>Increment</button>
    </div>
  );
}
```

#### Nargo 组件 (.nargo)

```nargo
<template>
  <div class="counter">
    <h1>{{ title }}</h1>
    <p>Count: {{ count }}</p>
    <button @click="increment">Increment</button>
  </div>
</template>

<script>
  import { signal } from '@nargo/core';
  
  export default {
    props: {
      title: {
        type: String,
        default: 'Counter'
      }
    },
    setup(props) {
      const [count, setCount] = signal(0);
      
      const increment = () => {
        setCount(count() + 1);
      };
      
      return {
        count,
        increment
      };
    }
  };
</script>

<style scoped>
  .counter {
    font-family: Arial, sans-serif;
    text-align: center;
    padding: 20px;
    border: 1px solid #ccc;
    border-radius: 8px;
    max-width: 300px;
    margin: 0 auto;
  }
  
  button {
    background-color: #4CAF50;
    color: white;
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  button:hover {
    background-color: #45a049;
  }
</style>
```

### 4.2 使用组件

```tsx
// src/App.tsx
import React from 'react';
import Counter from './components/Counter';

export default function App() {
  return (
    <div>
      <h1>My App</h1>
      <Counter />
    </div>
  );
}
```

### 4.3 开发服务器

启动开发服务器：

```bash
nargo dev
```

开发服务器会自动：
- 编译代码
- 启动热模块替换（HMR）
- 提供静态文件服务
- 自动打开浏览器
- 监听文件变化

### 4.4 构建生产版本

构建生产版本：

```bash
nargo build
```

构建过程会：
- 编译代码
- 压缩 JavaScript 和 CSS
- 生成优化后的代码
- 输出到 `dist` 目录
- 生成 source map

### 4.5 运行测试

运行测试：

```bash
nargo test
```

测试支持：
- 单元测试
- 组件测试
- 端到端测试
- 覆盖率报告

## 5. 包管理

### 5.1 安装依赖

```bash
# 安装生产依赖
nargo add react react-dom

# 安装依赖
nargo add typescript @types/react

# 安装特定版本
nargo add react@18.2.0

# 安装从 GitHub
nargo add github:facebook/react
```

### 5.2 移除依赖

```bash
# 移除依赖
nargo remove react typescript
```

### 5.3 更新依赖

```bash
# 更新所有依赖
nargo update

# 更新特定依赖
nargo update react

# 更新到最新版本
nargo update react@latest
```

### 5.4 查看依赖

```bash
# 查看所有依赖
nargo list
```

### 5.5 缓存管理

```bash
# 清理缓存
nargo cache clean

# 查看缓存大小
nargo cache size

# 查看缓存内容
nargo cache list
```

## 6. 高级特性

### 6.1 服务端渲染

Nargo 支持服务端渲染（SSR）：

```typescript
// server.tsx
import { renderToString } from '@nargo/ssr';
import App from './src/App';
import express from 'express';

const app = express();

app.get('*', async (req, res) => {
  const html = await renderToString(App, { title: 'Nargo App' });
  res.send(`
    <!DOCTYPE html>
    <html>
      <head>
        <title>Nargo App</title>
      </head>
      <body>
        <div id="app">${html}</div>
        <script src="/dist/bundle.js"></script>
      </body>
    </html>
  `);
});

app.listen(3000, () => {
  console.log('Server is running on port 3000');
});
```

### 6.2 代码分割

Nargo 支持代码分割：

```typescript
// src/App.tsx
import React, { lazy, Suspense } from 'react';

const LazyComponent = lazy(() => import('./components/LazyComponent'));

export default function App() {
  return (
    <div>
      <h1>My App</h1>
      <Suspense fallback={<div>Loading...</div>}>
        <LazyComponent />
      </Suspense>
    </div>
  );
}
```

### 6.3 国际化

Nargo 内置了国际化支持：

```typescript
import { useI18n } from '@nargo/core';

export default function App() {
  const { t, locale, setLocale } = useI18n({
    en: {
      hello: 'Hello {name}',
      welcome: 'Welcome to Nargo'
    },
    zh: {
      hello: '你好 {name}',
      welcome: '欢迎使用 Nargo'
    }
  });
  
  return (
    <div>
      <h1>{t('welcome')}</h1>
      <p>{t('hello', { name: 'World' })}</p>
      <button onClick={() => setLocale('en')}>English</button>
      <button onClick={() => setLocale('zh')}>中文</button>
    </div>
  );
}
```

### 6.4 插件系统

Nargo 支持插件系统：

```typescript
// my-plugin.ts
import type { NargoPlugin } from '@nargo/core';

export default function myPlugin(): NargoPlugin {
  return {
    name: 'my-plugin',
    transform(code, id) {
      if (id.endsWith('.tsx')) {
        // 转换 TypeScript 代码
        return code.replace('console.log', 'console.warn');
      }
      return code;
    }
  };
}
```

在配置中使用插件：

```toml
# nargo.toml
[plugins]
my-plugin = "./my-plugin.ts"
```

## 7. 最佳实践

### 7.1 项目配置

- **使用工作区**：对于多包项目，使用 Nargo 工作区
- **合理配置依赖**：明确区分生产依赖和开发依赖
- **锁定依赖版本**：使用 `nargo.lock` 确保依赖版本一致性

### 7.2 组件设计

- **单一职责**：每个组件只负责一个功能
- **类型安全**：使用 TypeScript 为组件添加类型
- **性能优化**：合理使用 React.memo、useMemo 等优化渲染

### 7.3 性能优化

- **代码分割**：对大型应用进行代码分割
- **懒加载**：对不立即需要的组件使用懒加载
- **虚拟列表**：对于长列表使用虚拟列表
- **缓存计算**：对于昂贵的计算使用 useMemo 或 computed

### 7.4 代码质量

- **代码格式化**：使用 `nargo format` 格式化代码
- **代码检查**：使用 `nargo lint` 检查代码质量
- **类型检查**：使用 `nargo typecheck` 进行类型检查
- **测试**：为组件和工具函数编写测试

### 7.5 部署

- **生产构建**：使用 `nargo build` 构建生产版本
- **静态资源**：将静态资源放在 public 目录
- **CDN**：使用 CDN 加速静态资源
- **缓存策略**：设置合理的缓存策略

## 8. 常见问题

### 8.1 依赖安装失败

**问题**：依赖安装失败，提示网络错误

**解决方案**：
- 检查网络连接是否正常
- 配置镜像源：`nargo config set registry https://registry.npmmirror.com`
- 清理缓存：`nargo cache clean`

### 8.2 编译错误

**问题**：编译失败，提示语法错误

**解决方案**：
- 检查 TypeScript 类型是否正确
- 检查导入路径是否正确
- 检查组件语法是否正确

### 8.3 运行时错误

**问题**：运行时出现错误

**解决方案**：
- 检查浏览器控制台错误信息
- 检查组件 props 是否正确传递
- 检查响应式数据是否正确使用

### 8.4 HMR 不工作

**问题**：热模块替换不工作

**解决方案**：
- 检查是否启动了开发服务器
- 检查浏览器控制台是否有错误
- 检查文件是否保存
- 检查网络连接是否正常

### 8.5 包管理问题

**问题**：依赖版本冲突

**解决方案**：
- 运行 `nargo update` 解决冲突
- 在 `nargo.toml` 中明确指定依赖版本
- 使用 `nargo list` 查看依赖树

## 9. 总结

Nargo 是一个革命性的前端开发工具链和包管理系统，通过以下特性为前端开发带来全新体验：

- **包管理创新**：完全替代 npm、pnpm 等包管理工具，使用 cargo 思路管理 TypeScript 项目，完全移除 node_modules
- **极速编译**：基于 Rust 的高性能编译器链，编译速度比传统 JS 编译器快 10-100 倍
- **丰富功能**：集成了多种前端开发工具的功能，包括编译、打包、测试、代码检查等
- **良好体验**：内置热模块替换和流畅的开发流程
- **生产优化**：最小化的运行时和包体积
- **可扩展性**：模块化设计和插件系统

通过本指南，你应该已经了解了 Nargo 的基本使用方法和最佳实践。随着你对 Nargo 的深入使用，你会发现它的更多强大功能和优势。

Nargo 正在不断发展和完善，欢迎你参与到 Nargo 的社区中来，共同推动前端开发的进步。