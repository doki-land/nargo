# HXO Toolchain Usage Guide

This document provides a detailed guide to using the HXO framework toolchain, including installation, configuration, and usage methods for each tool module.

## 1. Toolchain Module Overview

The HXO framework's toolchain consists of the following modules:

| Module Name | Function Description |
|-------------|---------------------|
| nargo-tools | Command-line tool set, providing build, formatting, linting, and other commands |
| nargo-bundler | Bundling and runtime optimization tool, achieving on-demand bundling |
| nargo-formatter | Code formatting tool, unifying code style |
| nargo-linter | Code checking tool, detecting code quality issues |
| nargo-compiler | Compiler, compiling HXO code to JavaScript |
| nargo-parser | Parser, parsing HXO code to intermediate representation |
| nargo-transformer | Code transformer, performing code optimization and transformation |
| nargo-type-check | Type checking tool, checking for type errors |
| nargo-ssr | Server-side rendering tool, supporting SSR |
| nargo-hydrate | Client hydration tool |
| nargo-git | Git tool, providing Git-related functionality |
| nargo-document | Documentation generation tool, generating project documentation |
| nargo-ir | Intermediate representation layer, providing data structures for other tools |
| nargo-types | Type definitions, providing type system support |
| nargo-style-processor | Style processor, processing CSS styles |
| nargo-script-analyzer | Script analyzer, analyzing JavaScript code |
| nargo-optimizer | Optimizer, optimizing compilation output |
| nargo-lsp | Language Server Protocol implementation, providing editor support |
| nargo-mcp | Module Communication Protocol, implementing inter-module communication |

## 2. Installation Methods

### 2.1 Install from Source

```bash
# Clone the repository
git clone https://github.com/your-username/nargo.git
cd nargo

# Install dependencies
cargo build --workspace

# Install command-line tool
cargo install --path compilers/nargo-tools
```

### 2.2 Install from npm

```bash
# Install global command-line tool
npm install -g @nargo/tools

# Or install in project
npm install @nargo/tools --save-dev
```

## 3. Command-line Tool Usage

### 3.1 Build Commands

```bash
# Basic build
nargo build

# Specify input file
nargo build --input ./src/App.nargo

# Specify output directory
nargo build --output ./dist

# Production build
nargo build --prod

# Enable SSR mode
nargo build --ssr

# Build as standalone project
nargo build --standalone
```

### 3.2 Formatting Commands

```bash
# Format single file
nargo format ./src/App.nargo

# Format entire directory
nargo format ./src
```

### 3.3 Code Linting Commands

```bash
# Check single file
nargo lint ./src/App.nargo

# Check entire directory
nargo lint ./src
```

### 3.4 Type Checking Commands

```bash
# Type check
nargo typecheck
```

### 3.5 Development Server Commands

```bash
# Start development server
nargo dev

# Specify port
nargo dev --port 3000
```

### 3.6 Documentation Generation Commands

```bash
# Generate documentation
nargo doc
```

## 4. Configuration Options

### 4.1 Project Configuration File

Create a `nargo.config.ts` file in the project root directory to configure the toolchain behavior:

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

### 4.2 Command-line Configuration

Most commands support configuration via command-line parameters, for example:

```bash
# Configure formatter
nargo format --indent-size 2 --single-quote false

# Configure linter
nargo lint --rule no-unused-vars=error --rule no-console=warn
```

## 5. Common Use Cases

### 5.1 Building a Project

```bash
# Development build
nargo build

# Production build
nargo build --prod

# Build as standalone project (includes HTML)
nargo build --standalone
```

### 5.2 Code Formatting

```bash
# Format single file
nargo format ./src/App.nargo

# Format entire directory
nargo format ./src

# Check formatting (don't modify files)
nargo format --check ./src
```

### 5.3 Code Linting

```bash
# Check single file
nargo lint ./src/App.nargo

# Check entire directory
nargo lint ./src

# Auto-fix issues
nargo lint --fix ./src
```

### 5.4 Type Checking

```bash
# Type check
nargo typecheck

# Strict mode type check
nargo typecheck --strict
```

### 5.5 Development Server

```bash
# Start development server
nargo dev

# Specify port
nargo dev --port 3000

# Enable HTTPS
nargo dev --https
```

## 6. Toolchain Module Detailed Usage

### 6.1 nargo-bundler

#### Features
- Feature Analysis: Deep scans the component tree, identifying whether features such as Signals, Effects, VDOM, or SSR are used
- Custom Runtime Generation: Based on analysis results, only packages the necessary runtime code
- Component Aggregation: Supports merging multiple `.nargo` compilation products into a single distributable file
- Dependency Optimization: Intelligently handles circular dependencies and common module extraction between components

#### Usage

```typescript
import { Bundler } from '@nargo/bundler';

// Create bundler instance
const bundler = new Bundler({ /* configuration options */ });

// Bundle modules
const result = await bundler.bundle([
  /* module list */
]);

// Get bundling result
console.log(result.outputs);
```

### 6.2 nargo-formatter

#### Features
- SFC Global Awareness: Perfectly recognizes HXO component structure, supports applying targeted formatting strategies to different blocks
- AST-driven Refactoring: Code rewriting based on parsed IR, more robust and semantically accurate
- Custom Style Configuration: Supports customizing common style preferences such as indentation, quotes, trailing semicolons via configuration files
- High-performance Execution: Pure Rust implementation formatting engine, maintaining millisecond-level processing speed even for large codebases

#### Usage

```typescript
import { NargoFormatter } from '@nargo/formatter';

// Create formatter instance
const formatter = new NargoFormatter();

// Format code
const formattedCode = await formatter.format(sourceCode);

// Output formatted code
console.log(formattedCode);
```

### 6.3 nargo-linter

#### Features
- Code Quality Checking: Detects potential issues in code, such as unused variables, console statements, etc.
- Extensible Rules: Supports custom rules and plugins
- Auto-fixing: Some issues support auto-fixing
- Rule Configuration: Supports enabling/disabling rules via configuration files
- Rule Set Management: Supports predefined rule sets (such as recommended, strict, etc.)

#### Built-in Rules

| Rule Name | Description | Default Level |
|-----------|-------------|---------------|
| no-console | Disallow use of console.log and other console statements | warning |
| no-debugger | Disallow use of debugger statements | error |
| no-deprecated-tags | Disallow use of deprecated HTML tags | error |
| no-empty-template | Disallow empty templates | error |
| no-unused-vars | Disallow unused variables | warning |
| no-unreachable-code | Disallow unreachable code | warning |

#### Rule Sets

| Rule Set Name | Description |
|---------------|-------------|
| recommended | Recommended rule set, including common code quality checking rules |
| strict | Strict rule set, including all rules with level set to error |

#### Configuration Example

```typescript
// nargo.config.ts
import { defineConfig } from '@nargo/core';

export default defineConfig({
  linter: {
    enabled: true,
    extends: 'recommended', // use recommended rule set
    rules: {
      'no-console': 'error', // override default level
      'no-unused-vars': 'warn'
    }
  }
});
```

#### Usage

```typescript
import { NargoLinter } from '@nargo/linter';
import { NoConsole, NoDeprecatedTags, NoUnusedVars, NoUnreachableCode } from '@nargo/linter/rules';

// Create linter instance
const linter = new NargoLinter();

// Add rules
linter.addRule(new NoConsole());
linter.addRule(new NoDeprecatedTags());
linter.addRule(new NoUnusedVars());
linter.addRule(new NoUnreachableCode());

// Run check
const diagnostics = await linter.run('file.nargo', sourceCode);

// Output check results
console.log(diagnostics);
```

#### Command-line Usage

```bash
# Check single file
nargo lint ./src/App.nargo

# Check entire directory
nargo lint ./src

# Specify configuration file
nargo lint --config nargo.config.json ./src
```

### 6.4 nargo-type-check

#### Features
- Type Checking: Checks for type errors in code
- TypeScript Integration: Supports TypeScript type definitions
- Strict Mode: Supports strict type checking mode

#### Usage

```typescript
import { TypeChecker } from '@nargo/type-check';

// Create type checker instance
const typeChecker = new TypeChecker({ /* configuration options */ });

// Run type check
const diagnostics = await typeChecker.check('file.nargo', sourceCode);

// Output check results
console.log(diagnostics);
```

### 6.5 nargo-ssr

#### Features
- Server-side Rendering: Renders HXO components on the server side
- Prefetch Data: Supports prefetching data during SSR
- Routing Support: Integrates routing functionality, supports server-side routing

#### Usage

```typescript
import { renderToString } from '@nargo/ssr';

// Render component to HTML
const html = await renderToString(component, { /* configuration options */ });

// Output HTML
console.log(html);
```

## 7. Notes

1. **Performance Optimization**: For large projects, it is recommended to use the `--prod` option for production builds to get the best performance
2. **Type Safety**: Enable the `typecheck` configuration to ensure code type safety
3. **Code Quality**: Regularly run the `nargo lint` command to ensure code quality
4. **Formatting**: Use the `nargo format` command to unify code style and improve code readability
5. **Dependency Management**: Regularly update dependencies to ensure you're using the latest version of the toolchain

## 8. Troubleshooting

### 8.1 Build Failure

- Check if input file path is correct
- Check if there are syntax errors in the code
- Check if dependencies are installed correctly

### 8.2 Formatting Error

- Check if there are syntax errors in the code
- Check if configuration options are correct

### 8.3 Code Linting Error

- Check if there are unused variables or other issues in the code
- Configure appropriate rule levels

### 8.4 Type Checking Error

- Check if type definitions are correct
- Check if type annotations are complete

## 9. Best Practices

1. **Use Configuration Files**: Create a `nargo.config.ts` file in the project root directory to unify toolchain behavior configuration
2. **Integrate into CI/CD**: Add `nargo lint` and `nargo typecheck` commands to your CI/CD pipeline to ensure code quality
3. **Use Scripts**: Add scripts to `package.json` for convenient running of common commands
4. **Regular Updates**: Regularly update toolchain versions to get the latest features and fixes
5. **Documentation**: Add documentation to the project, explaining toolchain usage methods and configuration options

## 10. Summary

The HXO toolchain provides a complete set of tools to help developers improve development efficiency and code quality. Through this guide, you should be able to proficiently use each module of the HXO toolchain to provide a better development experience for your projects.

If you encounter problems during usage, please refer to the troubleshooting section of this document or visit the HXO official documentation for more help.
