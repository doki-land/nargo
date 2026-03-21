# Nargo User Guide

## 1. Getting Started

### 1.1 Installing Nargo

Nargo is a standalone command-line tool that doesn't require node.js. You can install it using the following methods:

```bash
# Windows
iwr https://get.nargo.dev/win | iex

# macOS
curl -fsSL https://get.nargo.dev/mac | sh

# Linux
curl -fsSL https://get.nargo.dev/linux | sh
```

### 1.2 Initializing a Project

Initialize a new frontend project with Nargo:

```bash
# Create a new project
nargo init my-app

# Enter the project directory
cd my-app
```

### 1.3 Configuring Nargo

Nargo projects are configured with `nargo.toml` located in the project root:

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
@nargo/compiler = "^1.0.0"
@nargo/bundler = "^1.0.0"
@nargo/tools = "^1.0.0"

[build]
entry = "./src/main.tsx"
output = "./dist"
```

### 1.4 Running Nargo

```bash
# Development mode (with HMR)
nargo dev

# Build for production
nargo build

# Run tests
nargo test

# Install dependencies
nargo add react

# Remove dependencies
nargo remove react

# Update dependencies
nargo update
```

## 2. Core Concepts

### 2.1 Package Management

Nargo's core innovation is its package management system that completely replaces traditional npm/pnpm package management:

- **No node_modules**: Uses symlinks and path mapping, completely eliminating the node_modules directory
- **Dependency Locking**: Automatically generates `nargo.lock` files to ensure dependency version consistency
- **Package Cache**: Globally caches downloaded packages for faster installs
- **Multi-registry Support**: Supports npm registries, GitHub Packages, and other package sources

### 2.2 Component System

Nargo supports multiple component formats:

- **.nargo files**: Nargo native component format with template, script, and style
- **.tsx/.jsx files**: React components
- **.vue files**: Vue components (requires plugin installation)

### 2.3 Reactivity System

Nargo provides a signal-based reactivity system:

```typescript
import { signal, effect, computed } from '@nargo/core';

// Create a signal
const [count, setCount] = signal(0);

// Create a computed property
const doubled = computed(() => count() * 2);

// Create an effect
effect(() => {
  console.log(`Count: ${count()}, Doubled: ${doubled()}`);
});

// Update the signal
setCount(1); // Output: Count: 1, Doubled: 2
setCount(2); // Output: Count: 2, Doubled: 4
```

### 2.4 Template Syntax

Nargo template syntax is similar to Vue, supporting the following features:

- **Interpolation**: `{{ expression }}`
- **Directives**: `v-if`, `v-for`, `v-bind`, `v-on`, etc.
- **Event Handling**: `@click`, `@input`, etc.
- **Computed Properties**: Use computed properties directly in templates

## 3. Project Structure

Recommended Nargo project structure:

```
├── src/
│   ├── components/       # Components
│   │   ├── Counter.tsx
│   │   └── TodoList.nargo
│   ├── pages/            # Pages
│   │   ├── Home.tsx
│   │   └── About.nargo
│   ├── utils/            # Utility functions
│   │   └── helpers.ts
│   ├── styles/           # Global styles
│   │   └── global.css
│   ├── main.tsx          # Entry file
│   └── App.tsx           # Root component
├── public/               # Static assets
├── nargo.toml            # Nargo configuration
├── nargo.lock            # Dependency lock file
└── tsconfig.json         # TypeScript configuration
```

## 4. Development Workflow

### 4.1 Creating Components

#### React Component (.tsx)

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

#### Nargo Component (.nargo)

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

### 4.2 Using Components

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

### 4.3 Development Server

Start the development server:

```bash
nargo dev
```

The development server will automatically:
- Compile code
- Start Hot Module Replacement (HMR)
- Serve static files
- Automatically open the browser
- Watch for file changes

### 4.4 Building for Production

Build for production:

```bash
nargo build
```

The build process will:
- Compile code
- Minify JavaScript and CSS
- Generate optimized code
- Output to the `dist` directory
- Generate source maps

### 4.5 Running Tests

Run tests:

```bash
nargo test
```

Testing supports:
- Unit tests
- Component tests
- End-to-end tests
- Coverage reports

## 5. Package Management

### 5.1 Installing Dependencies

```bash
# Install production dependencies
nargo add react react-dom

# Install dependencies
nargo add typescript @types/react

# Install specific version
nargo add react@18.2.0

# Install from GitHub
nargo add github:facebook/react
```

### 5.2 Removing Dependencies

```bash
# Remove dependencies
nargo remove react typescript
```

### 5.3 Updating Dependencies

```bash
# Update all dependencies
nargo update

# Update specific dependency
nargo update react

# Update to latest version
nargo update react@latest
```

### 5.4 Viewing Dependencies

```bash
# View all dependencies
nargo list
```

### 5.5 Cache Management

```bash
# Clean cache
nargo cache clean

# View cache size
nargo cache size

# View cache contents
nargo cache list
```

## 6. Advanced Features

### 6.1 Server-Side Rendering

Nargo supports Server-Side Rendering (SSR):

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

### 6.2 Code Splitting

Nargo supports code splitting:

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

### 6.3 Internationalization

Nargo has built-in internationalization support:

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

### 6.4 Plugin System

Nargo supports a plugin system:

```typescript
// my-plugin.ts
import type { NargoPlugin } from '@nargo/core';

export default function myPlugin(): NargoPlugin {
  return {
    name: 'my-plugin',
    transform(code, id) {
      if (id.endsWith('.tsx')) {
        // Transform TypeScript code
        return code.replace('console.log', 'console.warn');
      }
      return code;
    }
  };
}
```

Using plugins in configuration:

```toml
# nargo.toml
[plugins]
my-plugin = "./my-plugin.ts"
```

## 7. Best Practices

### 7.1 Project Configuration

- **Use Workspaces**: Use Nargo workspaces for multi-package projects
- **Configure Dependencies Wisely**: Clearly distinguish between production and development dependencies
- **Lock Dependency Versions**: Use `nargo.lock` to ensure dependency version consistency

### 7.2 Component Design

- **Single Responsibility**: Each component should only be responsible for one function
- **Type Safety**: Use TypeScript to add types to your components
- **Performance Optimization**: Use React.memo, useMemo, etc., to optimize rendering

### 7.3 Performance Optimization

- **Code Splitting**: Split code for large applications
- **Lazy Loading**: Use lazy loading for components that aren't needed immediately
- **Virtual Lists**: Use virtual lists for long lists
- **Cache Computations**: Use useMemo or computed for expensive calculations

### 7.4 Code Quality

- **Code Formatting**: Use `nargo format` to format your code
- **Code Linting**: Use `nargo lint` to check code quality
- **Type Checking**: Use `nargo typecheck` for type checking
- **Testing**: Write tests for your components and utility functions

### 7.5 Deployment

- **Production Build**: Use `nargo build` to build for production
- **Static Assets**: Place static assets in the public directory
- **CDN**: Use a CDN to speed up static assets
- **Caching Strategy**: Set up reasonable caching strategies

## 8. Troubleshooting

### 8.1 Dependency Installation Failed

**Problem**: Dependency installation fails with network errors

**Solutions**:
- Check your network connection
- Configure a mirror source: `nargo config set registry https://registry.npmmirror.com`
- Clean the cache: `nargo cache clean`

### 8.2 Compilation Errors

**Problem**: Compilation fails with syntax errors

**Solutions**:
- Check that TypeScript types are correct
- Check that import paths are correct
- Check that component syntax is correct

### 8.3 Runtime Errors

**Problem**: Runtime errors occur

**Solutions**:
- Check browser console error messages
- Check that component props are passed correctly
- Check that reactive data is used correctly

### 8.4 HMR Not Working

**Problem**: Hot Module Replacement isn't working

**Solutions**:
- Check if the development server is running
- Check browser console for errors
- Check that files are saved
- Check network connection

### 8.5 Package Management Issues

**Problem**: Dependency version conflicts

**Solutions**:
- Run `nargo update` to resolve conflicts
- Explicitly specify dependency versions in `nargo.toml`
- Use `nargo list` to view the dependency tree

## 9. Summary

Nargo is a revolutionary frontend development toolchain and package management system that brings a new experience to frontend development through the following features:

- **Package Management Innovation**: Completely replaces npm, pnpm, and other package management tools, managing TypeScript projects with cargo's approach, completely eliminating node_modules
- **Extremely Fast Compilation**: Rust-based high-performance compiler chain, compilation speed 10-100x faster than traditional JS compilers
- **Rich Functionality**: Integrates the features of various frontend development tools, including compilation, bundling, testing, code checking, etc.
- **Great Experience**: Built-in Hot Module Replacement and smooth development workflow
- **Production Optimization**: Minimal runtime and package size
- **Extensibility**: Modular design and plugin system

Through this guide, you should now understand Nargo's basic usage and best practices. As you use Nargo more deeply, you'll discover more of its powerful features and advantages.

Nargo continues to evolve and improve. We welcome you to join the Nargo community to jointly advance frontend development.
