//! Nargo MCP Prompts
//!
//! This module defines and implements all the prompts available through
//! the Nargo MCP server for guiding AI assistants in common workflows.

use std::collections::HashMap;

use crate::types::{Content, GetPromptResult, Prompt, PromptArgument, PromptMessage, Role};

/// Returns all available Nargo prompts.
///
/// Each prompt provides a guided workflow for common Nargo operations.
///
/// # Available Prompts
///
/// - `nargo_project_setup` - Create a new Nargo project
/// - `nargo_dependency_analysis` - Analyze project dependencies
/// - `nargo_migration_guide` - Migrate from npm/yarn/pnpm
/// - `nargo_troubleshoot` - Diagnose and solve problems
/// - `nargo_best_practices` - Best practices guide
/// - `nargo_workspace_setup` - Set up a monorepo workspace
pub fn get_nargo_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "nargo_project_setup".to_string(),
            description: Some("创建新 Nargo 项目的完整指南".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("项目名称".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("项目类型：lib、app、component".to_string()),
                    required: Some(false),
                },
            ]),
        },
        Prompt {
            name: "nargo_dependency_analysis".to_string(),
            description: Some("分析项目依赖并提供优化建议".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "focus".to_string(),
                description: Some("分析重点：security、performance、size、updates".to_string()),
                required: Some(false),
            }]),
        },
        Prompt {
            name: "nargo_migration_guide".to_string(),
            description: Some("从 npm/yarn/pnpm 迁移到 Nargo 的指南".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "from_tool".to_string(),
                    description: Some("源工具：npm、yarn、pnpm".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "project_size".to_string(),
                    description: Some("项目规模：small、medium、large".to_string()),
                    required: Some(false),
                },
            ]),
        },
        Prompt {
            name: "nargo_troubleshoot".to_string(),
            description: Some("诊断和解决 Nargo 项目问题".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "issue_type".to_string(),
                    description: Some("问题类型：build、install、runtime、config".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "error_message".to_string(),
                    description: Some("错误消息（可选）".to_string()),
                    required: Some(false),
                },
            ]),
        },
        Prompt {
            name: "nargo_best_practices".to_string(),
            description: Some("Nargo 项目最佳实践指南".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "topic".to_string(),
                description: Some("主题：dependencies、scripts、workspace、build".to_string()),
                required: Some(false),
            }]),
        },
        Prompt {
            name: "nargo_workspace_setup".to_string(),
            description: Some("设置 Nargo monorepo 工作区".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "workspace_name".to_string(),
                    description: Some("工作区名称".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "packages".to_string(),
                    description: Some("初始包列表（逗号分隔）".to_string()),
                    required: Some(false),
                },
            ]),
        },
    ]
}

/// Gets a prompt by name with optional arguments.
///
/// # Arguments
///
/// * `name` - The name of the prompt to get.
/// * `arguments` - Optional arguments to fill in the prompt template.
///
/// # Returns
///
/// Result containing the prompt result or an error message.
pub fn get_nargo_prompt(
    name: &str,
    arguments: Option<HashMap<String, String>>,
) -> Result<GetPromptResult, String> {
    let args = arguments.unwrap_or_default();

    match name {
        "nargo_project_setup" => {
            let project_name = args
                .get("project_name")
                .ok_or("Missing 'project_name' argument")?;
            let project_type = args
                .get("project_type")
                .map(|s| s.as_str())
                .unwrap_or("app");

            Ok(GetPromptResult {
                description: Some(format!(
                    "创建 {} 类型的 Nargo 项目: {}",
                    project_type, project_name
                )),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!(
                                "我想创建一个名为 {} 的 {} 项目",
                                project_name, project_type
                            ),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"好的，我来帮你创建 Nargo 项目。

## 步骤 1: 创建项目
```bash
nargo new {} --template {}
```

## 步骤 2: 进入项目目录
```bash
cd {}
```

## 步骤 3: 项目结构
```
{}/
├── Nargo.toml          # 项目配置
├── Nargo.lock          # 依赖锁定
├── src/
│   ├── index.ts        # 入口文件
│   └── lib/            # 库代码
├── tests/              # 测试文件
└── target/             # 构建产物
```

## 步骤 4: 添加依赖
```bash
nargo add react         # 添加生产依赖
nargo add vitest --dev  # 添加开发依赖
```

## 步骤 5: 运行项目
```bash
nargo run dev    # 开发模式
nargo build      # 构建
nargo test       # 测试
```
"#,
                                project_name, project_type, project_name, project_name
                            ),
                        },
                    },
                ],
            })
        }
        "nargo_dependency_analysis" => {
            let focus = args.get("focus").map(|s| s.as_str()).unwrap_or("all");

            Ok(GetPromptResult {
                description: Some(format!("分析依赖（重点: {}）", focus)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!("请分析我的项目依赖，重点关注 {}", focus),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"我将为你执行依赖分析，重点关注 {}。

## 执行分析
```bash
nargo analyze --scope {}
```

## 分析维度

### 安全性 (security)
- 检查已知漏洞
- 扫描恶意代码
- 验证来源可信度

### 性能 (performance)
- Bundle 大小分析
- Tree-shaking 效果
- 运行时性能影响

### 体积 (size)
- 依赖总大小
- 重复依赖检测
- 可替代的轻量方案

### 更新 (updates)
- 过时依赖检测
- 破坏性变更警告
- 推荐更新策略

## 输出报告
分析完成后，你将获得：
1. 问题列表（按严重程度排序）
2. 优化建议
3. 自动修复命令
"#,
                                focus, focus
                            ),
                        },
                    },
                ],
            })
        }
        "nargo_migration_guide" => {
            let from_tool = args
                .get("from_tool")
                .ok_or("Missing 'from_tool' argument")?;
            let project_size = args
                .get("project_size")
                .map(|s| s.as_str())
                .unwrap_or("medium");

            Ok(GetPromptResult {
                description: Some(format!("从 {} 迁移到 Nargo", from_tool)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!(
                                "我想从 {} 迁移到 Nargo，项目规模是 {}",
                                from_tool, project_size
                            ),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"好的，这是从 {} 迁移到 Nargo 的完整指南。

## 迁移前准备

### 1. 备份当前项目
```bash
cp -r . ../project-backup
```

### 2. 清理旧依赖
```bash
rm -rf node_modules
rm package-lock.json yarn.lock pnpm-lock.yaml
```

## 迁移步骤

### 步骤 1: 初始化 Nargo
```bash
nargo init
```
这会自动检测并转换 package.json。

### 步骤 2: 转换依赖
Nargo 会自动转换：
- `dependencies` → `[dependencies]`
- `devDependencies` → `[dev-dependencies]`
- `scripts` → `[scripts]`

### 步骤 3: 验证迁移
```bash
nargo check
nargo test
nargo build
```

## 命令对照表

| {} | Nargo |
|---|---|
| install / i | nargo install |
| add <pkg> | nargo add <pkg> |
| remove <pkg> | nargo remove <pkg> |
| run <script> | nargo run <script> |
| update | nargo update |
| outdated | nargo outdated |

## 注意事项
- 工作区配置需要手动调整
- 某些 npm 特有功能可能需要替代方案
- 建议逐步迁移，先确保核心功能正常
"#,
                                from_tool, from_tool
                            ),
                        },
                    },
                ],
            })
        }
        "nargo_troubleshoot" => {
            let issue_type = args
                .get("issue_type")
                .ok_or("Missing 'issue_type' argument")?;
            let error_message = args.get("error_message");

            Ok(GetPromptResult {
                description: Some(format!("诊断 {} 问题", issue_type)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!(
                                "我遇到了 {} 问题{}",
                                issue_type,
                                error_message
                                    .map(|e| format!(": {}", e))
                                    .unwrap_or_default()
                            ),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"让我帮你诊断这个 {} 问题。

## 诊断步骤

### 1. 检查配置
```bash
nargo check --verbose
```

### 2. 清理缓存
```bash
nargo clean
rm -rf target
nargo install
```

### 3. 查看详细日志
```bash
nargo <command> --verbose --log-level debug
```

## 常见 {} 问题

### 构建问题 (build)
- 检查 TypeScript 配置
- 验证入口文件路径
- 检查循环依赖

### 安装问题 (install)
- 清理注册表缓存
- 检查网络连接
- 验证包名称和版本

### 运行时问题 (runtime)
- 检查环境变量
- 验证依赖版本兼容性
- 查看运行时日志

### 配置问题 (config)
- 验证 Nargo.toml 语法
- 检查路径配置
- 确认特性标志

## 获取帮助
如果问题仍未解决：
1. 运行 `nargo doctor` 进行全面诊断
2. 查看文档: https://nargo.dev/docs
3. 提交 issue: https://github.com/nargo/nargo/issues
"#,
                                issue_type, issue_type
                            ),
                        },
                    },
                ],
            })
        }
        "nargo_best_practices" => {
            let topic = args.get("topic").map(|s| s.as_str()).unwrap_or("all");

            Ok(GetPromptResult {
                description: Some(format!("Nargo 最佳实践: {}", topic)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!("请分享关于 {} 的 Nargo 最佳实践", topic),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"以下是关于 {} 的 Nargo 最佳实践。

## 依赖管理 (dependencies)

### 版本指定
```toml
[dependencies]
react = "18.2.0"           # 精确版本
lodash = "^4.17.0"         # 兼容版本
typescript = "~5.0.0"      # 补丁版本
```

### 特性选择
```toml
[dependencies]
lodash = {{ version = "4.17.21", features = ["es", "fp"] }}
```

### 可选依赖
```toml
[dependencies]
optional-lib = {{ version = "1.0.0", optional = true }}

[features]
extra = ["optional-lib"]
```

## 脚本配置 (scripts)

### 开发脚本
```toml
[scripts]
dev = "nargo run --watch --port 3000"
build = "nargo build --release"
test = "nargo test --coverage"
lint = "nargo lint --fix"
```

### 组合脚本
```toml
[scripts]
ci = "nargo lint && nargo test && nargo build"
```

## 工作区 (workspace)

### 配置结构
```toml
[workspace]
members = ["packages/*", "apps/*"]

[workspace.dependencies]
shared-dep = "1.0.0"
```

### 包继承
```toml
# packages/core/Nargo.toml
[package]
name = "core"
version.workspace = true
```

## 构建优化 (build)

### 目标配置
```toml
[build]
targets = ["web", "node"]
minify = true
sourcemap = true
```

### 缓存策略
```toml
[build.cache]
enabled = true
directory = ".nargo-cache"
```
"#,
                                topic
                            ),
                        },
                    },
                ],
            })
        }
        "nargo_workspace_setup" => {
            let workspace_name = args
                .get("workspace_name")
                .ok_or("Missing 'workspace_name' argument")?;
            let packages = args
                .get("packages")
                .map(|s| {
                    s.split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            Ok(GetPromptResult {
                description: Some(format!("设置工作区: {}", workspace_name)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: Content::Text {
                            text: format!(
                                "我想创建一个名为 {} 的工作区{}",
                                workspace_name,
                                if packages.is_empty() {
                                    "".to_string()
                                } else {
                                    format!("，包含包: {:?}", packages)
                                }
                            ),
                        },
                    },
                    PromptMessage {
                        role: Role::Assistant,
                        content: Content::Text {
                            text: format!(
                                r#"好的，我来帮你设置 Nargo 工作区。

## 创建工作区

### 步骤 1: 初始化
```bash
mkdir {} && cd {}
nargo init --workspace
```

### 步骤 2: 配置工作区
```toml
# Nargo.toml
[workspace]
name = "{}"
members = ["packages/*", "apps/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Your Name"]

[workspace.dependencies]
react = "18.2.0"
typescript = "5.0.0"
```

### 步骤 3: 创建包
```bash
# 创建库包
nargo new packages/core --template lib

# 创建应用
nargo new apps/web --template app
```

### 步骤 4: 包配置
```toml
# packages/core/Nargo.toml
[package]
name = "core"
version.workspace = true
edition.workspace = true

[dependencies]
react.workspace = true
```

## 工作区命令

```bash
# 在所有包中运行命令
nargo test --workspace

# 在特定包中运行
nargo build --package core

# 添加共享依赖
nargo add react --workspace
```

## 目录结构
```
{}/
├── Nargo.toml          # 工作区配置
├── Nargo.lock          # 统一锁文件
├── packages/
│   ├── core/           # 核心库
│   └── utils/          # 工具库
├── apps/
│   ├── web/            # Web 应用
│   └── cli/            # CLI 工具
└── target/             # 统一构建产物
```
"#,
                                workspace_name, workspace_name, workspace_name, workspace_name
                            ),
                        },
                    },
                ],
            })
        }
        _ => Err(format!("Unknown prompt: {}", name)),
    }
}
