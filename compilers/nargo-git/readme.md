# nargo-git

HXO 项目的 Git 操作模块。

## 概述

`nargo-git` 提供了 Git 仓库操作功能，使用原生 Rust Git 实现（gix）。支持克隆、初始化、提交、推送、拉取等常见 Git 操作。

## 功能特性

- Git 仓库克隆与初始化
- 分支信息获取
- 工作区状态查询
- 提交、推送、拉取操作

## 基本用法

```rust
use nargo_git;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    nargo_git::clone("https://github.com/user/repo.git", None)?;

    nargo_git::init(Path::new("./my-project"))?;

    let branch = nargo_git::get_current_branch(Path::new("."))?;
    println!("Current branch: {}", branch);

    let status = nargo_git::get_status(Path::new("."))?;
    for (path, state) in status {
        println!("{}: {}", path, state);
    }

    Ok(())
}
```

## 支持的操作

| 函数 | 描述 |
|------|------|
| `clone` | 克隆远程仓库 |
| `init` | 初始化本地仓库 |
| `get_current_branch` | 获取当前分支名 |
| `get_status` | 获取工作区状态 |
| `commit_all` | 暂存并提交所有更改 |
| `push` | 推送到远程仓库 |
| `pull` | 从远程仓库拉取更新 |
| `create_branch` | 创建新分支 |
| `switch_branch` | 切换到指定分支 |
