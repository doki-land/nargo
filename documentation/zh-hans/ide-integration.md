# IDE 集成指南

本指南介绍如何在不同的 IDE 和编辑器中集成 Nargo 语言支持。

## Visual Studio Code

Nargo 提供了官方的 VS Code 插件，支持完整的语言服务功能。

### 安装方法

1. 从 VS Code 扩展市场搜索并安装 "Nargo" 插件
2. 或者手动构建并安装：
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   code --install-extension nargo-vscode-0.1.0.vsix
   ```

### 功能特性

- 语法高亮
- 代码补全
- 定义跳转
- 实时错误检查
- 代码分析
- 命令集成（build、run、test）

## Vim / Neovim

通过 LSP 客户端插件集成 Nargo 语言服务器。

### 安装步骤

1. 安装 LSP 客户端插件：
   - Vim: [coc.nvim](https://github.com/neoclide/coc.nvim)
   - Neovim: [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)

2. 配置 Nargo LSP：

   **使用 coc.nvim**：
   ```json
   {
     "languageserver": {
       "nargo": {
         "command": "nargo-lsp",
         "filetypes": ["nargo", "oak"],
         "rootPatterns": ["Cargo.toml", "nargo.config.toml"]
       }
     }
   }
   ```

   **使用 nvim-lspconfig**：
   ```lua
   require('lspconfig').nargo.setup({
     cmd = {"nargo-lsp"},
     filetypes = {"nargo", "oak"},
     root_dir = require('lspconfig.util').root_pattern("Cargo.toml", "nargo.config.toml"),
   })
   ```

## Emacs

使用 `lsp-mode` 集成 Nargo 语言服务器。

### 安装步骤

1. 安装 `lsp-mode`：
   ```elisp
   (use-package lsp-mode
     :ensure t
     :commands lsp)
   ```

2. 配置 Nargo LSP：
   ```elisp
   (defun lsp-nargo-enable ()
     (interactive)
     (lsp-register-client
      (make-lsp-client
       :new-connection (lsp-stdio-connection "nargo-lsp")
       :major-modes '(nargo-mode oak-mode)
       :server-id 'nargo-lsp
       :priority 1
       :initialization-options '()
       :activation-fn (lsp-activate-on "nargo" "oak"))))
   ```

## Sublime Text

通过 LSP 插件集成 Nargo 语言服务器。

### 安装步骤

1. 安装 Sublime Text LSP 插件
2. 配置 Nargo LSP：
   ```json
   {
     "clients": {
       "nargo": {
         "command": ["nargo-lsp"],
         "scopes": ["source.nargo", "source.oak"],
         "syntaxes": ["Packages/Nargo/Nargo.sublime-syntax"],
         "languageId": "nargo"
       }
     }
   }
   ```

## 通用 LSP 客户端配置

对于其他支持 LSP 的编辑器，可以按照以下通用步骤配置：

1. 确保 `nargo-lsp` 可执行文件在系统 PATH 中
2. 配置编辑器的 LSP 客户端，指向 `nargo-lsp` 命令
3. 设置文件类型关联为 `nargo` 或 `oak`
4. 配置项目根目录识别模式（通常为 `Cargo.toml` 或 `nargo.config.toml`）

## 构建和测试

### 构建语言服务器

```bash
cargo build --package nargo-lsp
```

### 测试 VS Code 插件

1. 构建插件：
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   ```

2. 运行扩展开发主机：
   - 按 F5 在 VS Code 中启动扩展开发主机
   - 在新窗口中打开 Nargo 项目
   - 测试语法高亮、代码补全等功能

### 验证语言服务器

```bash
nargo-lsp --help
```

## 故障排除

### 语言服务器无法启动

1. 检查 `nargo-lsp` 是否正确安装
2. 确保 `nargo-lsp` 在系统 PATH 中
3. 检查编辑器的 LSP 配置是否正确
4. 查看编辑器的日志输出以获取详细错误信息

### 功能不工作

1. 确保项目根目录包含 `Cargo.toml` 或 `nargo.config.toml`
2. 检查语言服务器日志输出
3. 尝试重启编辑器或语言服务器

## 贡献

如果您为其他编辑器创建了 Nargo 集成，请提交 PR 来更新本指南。