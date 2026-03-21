# Nargo API 文档

## 1. 核心 API

### 1.1 响应式系统 API

#### signal

创建一个响应式信号，用于管理状态。

```typescript
import { signal } from '@nargo/core';

// 创建信号
const [count, setCount] = signal(0);

// 读取信号值
console.log(count()); // 输出: 0

// 更新信号值
setCount(1);
console.log(count()); // 输出: 1

// 使用函数更新信号值
setCount(prev => prev + 1);
console.log(count()); // 输出: 2
```

**参数**：
- `initialValue`：信号的初始值

**返回值**：
- 一个元组，包含：
  - 读取信号值的函数
  - 更新信号值的函数

#### effect

创建一个副作用，当依赖的信号变化时执行。

```typescript
import { signal, effect } from '@nargo/core';

const [count, setCount] = signal(0);

// 创建副作用
effect(() => {
  console.log(`Count: ${count()}`);
});

// 当信号更新时，副作用会自动执行
setCount(1); // 输出: Count: 1
setCount(2); // 输出: Count: 2
```

**参数**：
- `fn`：副作用函数
- `options`（可选）：配置选项
  - `scheduler`：调度器函数
  - `name`：副作用名称（用于调试）

**返回值**：
- 一个清理函数，调用后会停止副作用

#### computed

创建一个计算属性，基于其他信号的值计算得出。

```typescript
import { signal, computed } from '@nargo/core';

const [count, setCount] = signal(0);

// 创建计算属性
const doubled = computed(() => count() * 2);

// 读取计算属性值
console.log(doubled()); // 输出: 0

// 当依赖的信号更新时，计算属性会自动重新计算
setCount(1);
console.log(doubled()); // 输出: 2
```

**参数**：
- `fn`：计算函数
- `options`（可选）：配置选项
  - `name`：计算属性名称（用于调试）

**返回值**：
- 一个读取计算属性值的函数

### 1.2 组件 API

#### createComponent

创建一个 Nargo 组件。

```typescript
import { createComponent, signal } from '@nargo/core';

const Counter = createComponent({
  props: {
    initialCount: {
      type: Number,
      default: 0
    }
  },
  setup(props) {
    const [count, setCount] = signal(props.initialCount);
    
    const increment = () => {
      setCount(count() + 1);
    };
    
    return {
      count,
      increment
    };
  },
  template: `
    <div>
      <p>Count: {{ count }}</p>
      <button @click="increment">Increment</button>
    </div>
  `
});
```

**参数**：
- `options`：组件选项
  - `props`：组件属性定义
  - `setup`：组件初始化函数
  - `template`：组件模板
  - `styles`：组件样式

**返回值**：
- 一个组件构造函数

#### useComponent

在函数组件中使用 Nargo 组件。

```typescript
import { useComponent, signal } from '@nargo/core';

function Counter({ initialCount = 0 }) {
  const [count, setCount] = signal(initialCount);
  
  const increment = () => {
    setCount(count() + 1);
  };
  
  return useComponent({
    setup: () => ({ count, increment }),
    template: `
      <div>
        <p>Count: {{ count }}</p>
        <button @click="increment">Increment</button>
      </div>
    `
  });
}
```

**参数**：
- `options`：组件选项
  - `setup`：组件初始化函数
  - `template`：组件模板
  - `styles`：组件样式

**返回值**：
- 一个组件实例

### 1.3 DOM API

#### render

将组件渲染到 DOM 中。

```typescript
import { render } from '@nargo/core';
import App from './App';

render(App, document.getElementById('app'));
```

**参数**：
- `component`：要渲染的组件
- `container`：DOM 容器元素
- `options`（可选）：渲染选项
  - `hydrate`：是否进行 hydration（服务端渲染时使用）

**返回值**：
- 一个销毁函数，调用后会移除组件

#### createElement

创建一个 DOM 元素。

```typescript
import { createElement } from '@nargo/core';

const element = createElement('div', {
  className: 'container',
  onClick: () => console.log('Clicked')
}, [
  createElement('h1', null, 'Hello World'),
  createElement('p', null, 'Welcome to Nargo')
]);

document.body.appendChild(element);
```

**参数**：
- `tag`：元素标签名
- `props`：元素属性
- `children`：子元素

**返回值**：
- 一个 DOM 元素

### 1.4 国际化 API

#### useI18n

使用国际化功能。

```typescript
import { useI18n } from '@nargo/core';

function App() {
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

**参数**：
- `messages`：国际化消息对象
- `options`（可选）：配置选项
  - `defaultLocale`：默认语言
  - `fallbackLocale`：回退语言

**返回值**：
- 一个对象，包含：
  - `t`：翻译函数
  - `locale`：当前语言
  - `setLocale`：设置语言的函数

## 2. 包管理 API

### 2.1 命令行 API

#### nargo init

初始化一个新的 Nargo 项目。

```bash
# 创建新项目
nargo init my-app

# 初始化现有项目
nargo init
```

**选项**：
- `--template`：指定项目模板
- `--force`：强制初始化

#### nargo add

安装依赖。

```bash
# 安装生产依赖
nargo add react react-dom

# 安装开发依赖
nargo add typescript @types/react --dev

# 安装特定版本
nargo add react@18.2.0

# 安装从 GitHub
nargo add github:facebook/react
```

**选项**：
- `--dev`：安装为开发依赖
- `--peer`：安装为 peer 依赖
- `--optional`：安装为可选依赖

#### nargo remove

移除依赖。

```bash
# 移除依赖
nargo remove react typescript
```

**选项**：
- `--dev`：从开发依赖中移除
- `--peer`：从 peer 依赖中移除
- `--optional`：从可选依赖中移除

#### nargo update

更新依赖。

```bash
# 更新所有依赖
nargo update

# 更新特定依赖
nargo update react

# 更新到最新版本
nargo update react@latest
```

**选项**：
- `--latest`：更新到最新版本
- `--dry-run`：模拟更新，不实际修改

#### nargo list

查看依赖。

```bash
# 查看所有依赖
nargo list

# 查看开发依赖
nargo list --dev
```

**选项**：
- `--dev`：查看开发依赖
- `--peer`：查看 peer 依赖
- `--optional`：查看可选依赖
- `--tree`：以树状结构查看

### 2.2 编程 API

#### PackageManager

包管理器类，用于以编程方式管理依赖。

```typescript
import { PackageManager } from '@nargo/tools';

const pm = new PackageManager();

// 安装依赖
await pm.add('react', '^18.2.0');

// 移除依赖
await pm.remove('react');

// 更新依赖
await pm.update('react');

// 查看依赖
const dependencies = await pm.list();
console.log(dependencies);
```

**方法**：
- `add(name: string, version?: string, options?: AddOptions)`：安装依赖
- `remove(name: string, options?: RemoveOptions)`：移除依赖
- `update(name?: string, options?: UpdateOptions)`：更新依赖
- `list(options?: ListOptions)`：查看依赖
- `init(options?: InitOptions)`：初始化项目

## 3. 编译 API

### 3.1 命令行 API

#### nargo dev

启动开发服务器。

```bash
# 启动开发服务器
nargo dev

# 指定端口
nargo dev --port 3000

# 指定主机
nargo dev --host 0.0.0.0
```

**选项**：
- `--port`：指定端口
- `--host`：指定主机
- `--open`：自动打开浏览器
- `--no-hmr`：禁用热模块替换

#### nargo build

构建生产版本。

```bash
# 构建生产版本
nargo build

# 指定输出目录
nargo build --output ./build

# 启用源码映射
nargo build --sourcemap
```

**选项**：
- `--output`：指定输出目录
- `--sourcemap`：启用源码映射
- `--minify`：启用代码压缩
- `--analyze`：生成构建分析报告

#### nargo test

运行测试。

```bash
# 运行所有测试
nargo test

# 运行特定测试文件
nargo test ./src/__tests__/counter.test.tsx

# 启用覆盖率报告
nargo test --coverage
```

**选项**：
- `--coverage`：启用覆盖率报告
- `--watch`：监视模式
- `--reporter`：指定测试报告格式

### 3.2 编程 API

#### Compiler

编译器类，用于以编程方式编译代码。

```typescript
import { Compiler } from '@nargo/compiler';

const compiler = new Compiler({
  entry: './src/main.tsx',
  output: './dist',
  mode: 'production'
});

// 编译代码
const result = await compiler.compile();
console.log(result);
```

**方法**：
- `compile(options?: CompileOptions)`：编译代码
- `watch(options?: WatchOptions)`：监视模式编译

#### Bundler

打包器类，用于以编程方式打包代码。

```typescript
import { Bundler } from '@nargo/bundler';

const bundler = new Bundler({
  entry: './src/main.tsx',
  output: './dist',
  codeSplitting: 'by-route'
});

// 打包代码
const result = await bundler.bundle();
console.log(result);
```

**方法**：
- `bundle(options?: BundleOptions)`：打包代码
- `watch(options?: WatchOptions)`：监视模式打包

## 4. 服务端渲染 API

### 4.1 核心 API

#### renderToString

将组件渲染为 HTML 字符串。

```typescript
import { renderToString } from '@nargo/ssr';
import App from './src/App';

const html = await renderToString(App, { title: 'Nargo App' });
console.log(html);
```

**参数**：
- `component`：要渲染的组件
- `props`：组件属性
- `options`（可选）：渲染选项
  - `context`：渲染上下文
  - `timeout`：渲染超时时间

**返回值**：
- 一个 HTML 字符串

#### renderToStream

将组件渲染为 HTML 流。

```typescript
import { renderToStream } from '@nargo/ssr';
import App from './src/App';
import express from 'express';

const app = express();

app.get('*', async (req, res) => {
  res.write('<!DOCTYPE html><html><head><title>Nargo App</title></head><body><div id="app">');
  
  const stream = await renderToStream(App, { title: 'Nargo App' });
  stream.pipe(res, { end: false });
  
  stream.on('end', () => {
    res.write('</div><script src="/dist/bundle.js"></script></body></html>');
    res.end();
  });
});

app.listen(3000);
```

**参数**：
- `component`：要渲染的组件
- `props`：组件属性
- `options`（可选）：渲染选项
  - `context`：渲染上下文
  - `timeout`：渲染超时时间

**返回值**：
- 一个可读流

## 5. 工具 API

### 5.1 代码格式化

#### format

格式化代码。

```typescript
import { format } from '@nargo/formatter';

const code = `function hello() {console.log('Hello World');}`;
const formattedCode = format(code, { parser: 'typescript' });
console.log(formattedCode);
```

**参数**：
- `code`：要格式化的代码
- `options`：格式化选项
  - `parser`：代码解析器（typescript、javascript、html、css）
  - `semi`：是否使用分号
  - `singleQuote`：是否使用单引号
  - `tabWidth`：缩进宽度

**返回值**：
- 格式化后的代码

### 5.2 代码检查

#### lint

检查代码质量。

```typescript
import { lint } from '@nargo/linter';

const code = `function hello() { console.log('Hello World'); }`;
const result = lint(code, { parser: 'typescript' });
console.log(result);
```

**参数**：
- `code`：要检查的代码
- `options`：检查选项
  - `parser`：代码解析器（typescript、javascript、html、css）
  - `rules`：检查规则

**返回值**：
- 检查结果

### 5.3 类型检查

#### typecheck

检查 TypeScript 类型。

```typescript
import { typecheck } from '@nargo/tools';

const result = await typecheck('./src');
console.log(result);
```

**参数**：
- `path`：要检查的文件或目录路径
- `options`：检查选项
  - `tsconfig`：tsconfig 文件路径
  - `strict`：是否使用严格模式

**返回值**：
- 检查结果

## 6. 插件 API

### 6.1 创建插件

```typescript
import type { NargoPlugin } from '@nargo/core';

export default function myPlugin(): NargoPlugin {
  return {
    name: 'my-plugin',
    
    // 转换代码
    transform(code, id) {
      if (id.endsWith('.tsx')) {
        // 转换 TypeScript 代码
        return code.replace('console.log', 'console.warn');
      }
      return code;
    },
    
    // 解析模块
    resolveId(source, importer) {
      if (source === 'my-alias') {
        return './src/my-module.ts';
      }
      return null;
    },
    
    // 加载模块
    load(id) {
      if (id === './src/my-module.ts') {
        return 'export default "Hello World";';
      }
      return null;
    }
  };
}
```

**插件钩子**：
- `transform(code, id)`：转换代码
- `resolveId(source, importer)`：解析模块 ID
- `load(id)`：加载模块
- `buildStart(options)`：构建开始时执行
- `buildEnd(error)`：构建结束时执行
- `watchChange(id, change)`：文件变化时执行

### 6.2 使用插件

在 `nargo.toml` 中配置插件：

```toml
# nargo.toml
[plugins]
my-plugin = "./my-plugin.ts"
```

或者在代码中使用：

```typescript
import { createCompiler } from '@nargo/compiler';
import myPlugin from './my-plugin';

const compiler = createCompiler({
  entry: './src/main.tsx',
  output: './dist',
  plugins: [myPlugin()]
});
```

## 7. 配置 API

### 7.1 配置文件

Nargo 项目的配置文件是 `nargo.toml`，位于项目根目录：

```toml
# nargo.toml
[package]
name = "my-app"
version = "0.1.0"
description = "My Nargo Application"

[dependencies]
react = "^18.2.0"
react-dom = "^18.2.0"
typescript = "^5.0.0"

[dev-dependencies]
@types/react = "^18.2.0"
@types/react-dom = "^18.2.0"

[build]
entry = "./src/main.tsx"
output = "./dist"
mode = "production"
sourcemap = true

[server]
port = 3000
host = "0.0.0.0"
open = true

[plugins]
my-plugin = "./my-plugin.ts"

[formatter]
semi = true
singleQuote = false
tabWidth = 2

[linter]
rules = {
  "no-unused-vars" = "error",
  "no-console" = "warn"
}
```

### 7.2 编程配置

以编程方式配置 Nargo：

```typescript
import { createConfig } from '@nargo/core';

const config = createConfig({
  package: {
    name: 'my-app',
    version: '0.1.0',
    description: 'My Nargo Application'
  },
  dependencies: {
    react: '^18.2.0',
    react-dom: '^18.2.0'
  },
  build: {
    entry: './src/main.tsx',
    output: './dist'
  }
});
```

**配置选项**：
- `package`：包配置
- `dependencies`：生产依赖
- `devDependencies`：开发依赖
- `peerDependencies`：peer 依赖
- `optionalDependencies`：可选依赖
- `build`：构建配置
- `server`：开发服务器配置
- `plugins`：插件配置
- `formatter`：代码格式化配置
- `linter`：代码检查配置
- `typecheck`：类型检查配置
- `registry`：包注册表配置
- `resolver`：依赖解析配置
- `cache`：包缓存配置
- `workspace`：工作区配置

## 8. 工作区 API

### 8.1 工作区配置

在 `nargo.toml` 中配置工作区：

```toml
# nargo.toml
[workspace]
members = [
  "packages/*"
]

[workspace.dependencies]
react = "^18.2.0"
react-dom = "^18.2.0"
```

### 8.2 工作区命令

#### nargo workspace

管理工作区。

```bash
# 查看工作区成员
nargo workspace list

# 运行所有工作区成员的命令
nargo workspace run build

# 运行特定工作区成员的命令
nargo workspace run --package my-package build
```

**选项**：
- `list`：查看工作区成员
- `run`：运行命令
- `add`：添加工作区依赖
- `remove`：移除工作区依赖
- `update`：更新工作区依赖

## 9. 总结

Nargo 提供了丰富的 API，涵盖了前端开发的各个方面，包括：

- **核心 API**：响应式系统、组件系统、DOM 操作、国际化等
- **包管理 API**：依赖管理、项目初始化等
- **编译 API**：开发服务器、生产构建、测试等
- **服务端渲染 API**：HTML 字符串渲染、流式渲染等
- **工具 API**：代码格式化、代码检查、类型检查等
- **插件 API**：扩展 Nargo 功能
- **配置 API**：项目配置管理
- **工作区 API**：多包项目管理

通过这些 API，开发者可以灵活地使用 Nargo 的各种功能，构建高效、高质量的前端应用。

Nargo 的 API 设计注重简洁性和一致性，同时提供了足够的灵活性和扩展性，满足不同项目的需求。随着 Nargo 的不断发展，API 也会不断完善和扩展，为前端开发带来更多便利。