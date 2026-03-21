# Nargo: The Future of Frontend Engineering

> **One for All, All in One.** 
> The next-generation frontend debugging and build tool based on Rust full-stack philosophy.

Nargo is not just another tool that mimics Vite or Rolldown. It's a complete rethinking of the frontend engineering pipeline. We've abandoned dependencies on external commands (Git, Husky, Lint, Docker) in favor of Rust-native implementations, creating an extremely fast, lightweight, and zero-configuration single-binary environment.

## 🚀 Core Philosophy

### 1. CaaS (Compiler as a Service)
Nargo deeply integrates `rustc_interface`, turning compiler capabilities into a service.
- **Instant Cold Start**: Leverages incremental compilation and a persistent in-memory compiler session.
- **Smart Hot Reload**: More than just TS HMR, it provides smart incremental reloading for the Rust backend.

### 2. RaaS (Runtime as a Service) — *New Concept*
The **R** here stands for **Runtime**, not Rust.
Nargo introduces the **Runtime as a Service** concept:
- **Seamless Environment**: Developers don't need to worry about Node.js, Bun, or Deno. Nargo includes a high-performance runtime service providing real-time type bridging (Type Bridge) and Mock capabilities for the frontend.
- **Types as a Service**: Through `nargo bridge`, Rust backend structs are converted to TS definitions in real-time, making frontend-backend type contracts a real-time service.
- **On-demand Loading**: Runtime capabilities are dynamically injected as needed, ensuring the development environment always remains lightweight.

### 3. All in One Binary
- **Native Git Audit**: No `git` command calls, directly reads/writes `.git` directory via `gix`, speed increases several times.
- **Native Hooks Management**: Completely replaces Husky with a Git Hooks engine implemented natively in Rust.
- **Zero-dependency Monorepo**: Built-in Workspace task parallel orchestrator, supporting HMR for hybrid projects.

## 🛠️ Quick Start

```bash
# Start hybrid hot-reload (Rust + TS)
nargo run dev --hybrid

# Sync frontend-backend types
nargo bridge

# Native Git status view
nargo git status

# Install native Git Hooks
nargo hooks install
```

## 📂 Project Structure

- `compilers/`: Rust core components (Compiler, Parser, Bundler, Bridge, etc.)
- `runtimes/`: Frontend runtime and type definitions
- `documentation/`: Official documentation

## ⚖️ License

MIT.
