# Nargo Release GitHub 白皮书

**版本**: v0.1.0（草案）  
**日期**: 2026年8月  
**命令**: `nargo release github`（可简写 `nargo release`）

> **仓库落地约定（与交付形态对齐）**  
> - 用户安装 / CLI：`@nargo/nargo`（命令仍为 `nargo`；无 scope 的 `nargo` 被 npm 名相似策略拒绝）→ optionalDeps `@nargo/nargo-<platform>`  
> - 共享库：`@nargo/*`（如 `@nargo/core`）；**没有** `@nargo/cli` / `@nargo/binding`  
> - 核心逻辑：Rust `nargo-release` + `nargo-git`（gix），经 NAPI 暴露  
> - JSON / `package.json`：Oak（`oak_json` 等）  
> - Workspace：自动探测；存在 `pnpm-workspace.yaml` 时切 pnpm 兼容模式  
> - **缓存**：整个 workspace 共用一个目录 `<root>/.cache/nargo`（无 per-package cache）  
> - 与 `nargo trust` / placeholder 分轨：本命令管 GitHub Release；npm OIDC 发布走 trust

---

## 1. 引言：从脚本泥潭到声明式发布

在 Monorepo 的日常开发中，版本发布与变更日志生成是最容易被忽视却又最耗费心力的环节。观察大量开源项目后，我们发现一个普遍现象：**每个项目都在 `release.yml` 中重复编写大量易错的 Shell 脚本**——解析 commit 消息、拼接 Markdown、调用 GitHub API、处理特殊字符转义、判断版本号……这些脚本往往由维护者从上一个项目复制粘贴而来，稍作修改，却又在每次运行时因边界情况而崩溃。

更令人沮丧的是，这些脚本只服务于「发布」这一个动作，却横跨了 Git、GitHub CLI、sed、awk、jq 等多个工具链，调试成本极高。贡献者需要额外学习 commit 规范（或 changeset 语法），维护者需要反复提醒「请添加变更记录」，整个流程充满了「人肉检查」的痕迹。

Nargo 的 `release github` 子命令正是为解决这一痛点而生。它将发布流程提升为**声明式配置**，把原本散落在 YAML 里的命令式脚本，全部收敛到 `Nargo.toml` 中，并由 Rust 编写的确定性引擎执行。目标只有一个：**让维护者和贡献者都能忘记发布流程，让 CI 自动完成从 commit 到 GitHub Release 的全链路。**

---

## 2. 设计哲学

### 2.1 规范驱动，而非启发式推断

早期的自动化发布工具倾向于从 PR 标题或 commit 消息中「猜测」语义版本，这导致了不可预测的结果。Nargo 的设计原则是：**版本变更必须由显式的规范决定，而非模糊的模式匹配**。我们要求 commit 消息遵循一种可配置的规范（如 Gitmoji 或 Conventional Commits），并通过确定性的规则映射到 `major` / `minor` / `patch`。规范本身可以自由定制，但一旦定义，版本计算就是纯函数，结果唯一。

### 2.2 配置即脚本，所有逻辑内聚于 `Nargo.toml`

Nargo 坚决反对在 CLI 中传递大量参数。`release` 命令的所有行为——包括规范解析、版本策略、模板路径、GitHub 仓库信息——都应声明在项目根目录的 `Nargo.toml` 中。CLI 只提供少数覆盖选项（如 `--dry-run`、`--version`），用于临时调试或紧急发布。这样做的好处是：配置随代码版本管理，团队共享，CI 调用时无需重复指定。

### 2.3 零摩擦贡献者体验

贡献者不需要学习 changeset 文件格式，也不需要记忆 commit 规范中的所有 type。他们只需在提交时使用团队约定的前缀（如 emoji 或 `feat:`），剩下的全部由 Nargo 自动处理。对于新手，Nargo 还提供了交互式 `nargo commit` 辅助命令，引导填写符合规范的 commit message。

### 2.4 自举友好，安全可靠

作为自身使用 Nargo 的项目，发布必须遵守：**用当前稳定版（n）发布新版本（n+1）的源码；n+1 的 NAPI / 二进制只作为构建产物上传到 Release。** 绝不存在「用尚未发布的 n+1 去测发布流程」的循环。契约、锁定版本与回滚见第 5.2 节。

---

## 3. 核心功能：`nargo release github` 命令详解

### 3.1 工作流概览

执行 `nargo release github`（或简写 `nargo release`）时，引擎按以下步骤运行：

1. **确定 commit 范围**：从上一个 Git tag（默认）或用户指定的 base ref（如 `main`）开始，收集从 `HEAD` 到该范围之间的所有 commit。
2. **解析 commit 消息**：根据配置的规范（`style`），提取每个 commit 的类型（type）、影响范围（scope）、是否破坏性变更（breaking）、关联的 PR/Issue 号。
3. **计算版本**：按 scope 分组（即 workspace 中的各个包），对每个包根据其 commit 类型计算新的语义版本号（major/minor/patch/不变化）。若多个 commit 作用于同一包，取最高级别的版本变化（策略可配置为 `highest` 或 `cumulative`）。
4. **生成 Release Notes**：将 commit 按类别（Features / Fixes / Breaking Changes 等）分组，并合并同一包的变更，渲染为 Markdown。模板支持 Handlebars，内置默认模板。
5. **创建 Git tag**：在本地创建格式为 `v${version}` 的 tag，并推送到远程（可选）。
6. **创建 GitHub Release**：使用 GitHub API（通过 OIDC 或 token 认证）创建 Release，标题为 tag 名称，正文为生成的 Release Notes，若包含 `BREAKING CHANGE` 则自动标记为 `prerelease`（可配置）。

所有步骤都支持 `--dry-run` 预览，除最后一步外，其余步骤可在本地模拟执行。

### 3.2 Commit 解析引擎

解析引擎是整个流程的核心，它被设计为**可插拔的规则系统**。默认内置三种规范：`gitmoji`、`conventional` 和 `angular`，用户也可通过 `custom` 完全自定义。

#### 3.2.1 内置规范

**Gitmoji**：以 `:sparkles:`、`:bug:` 等 emoji 开头，后跟 `(scope)` 可选。例如：`:sparkles: (auth) add OAuth2 login`。

| Emoji | 版本变化 | Release 分类 |
|-------|---------|-------------|
| `:sparkles:` | minor | Features |
| `:bug:` | patch | Bug Fixes |
| `:boom:` | major | Breaking Changes |
| `:memo:` | patch | Documentation |
| `:zap:` | patch | Performance |
| `:recycle:` | none | Refactoring |
| `:art:` | none | Code Style |
| `:white_check_mark:` | none | Tests |
| `:wrench:` | none | Configuration |

**Conventional Commits**：格式为 `<type>(<scope>): <subject>`，其中 `type` 为 `feat`、`fix`、`docs`、`perf`、`refactor` 等。若 footer 包含 `BREAKING CHANGE:`，则视作 breaking。版本映射：`feat` → minor，`fix`/`docs`/`perf` → patch，其余 → none。

**Angular**：与 Conventional 类似，但更强调 scope 的使用，且 `BREAKING CHANGE` 必须出现在 commit body 中。

#### 3.2.2 自定义规范

用户可以通过 `[release.spec.rules]` 数组定义任意正则匹配规则，每条规则指定 `version`（`major`/`minor`/`patch`/`none`）和 `category`（显示在 Release Notes 中的分组标题）。未匹配的 commit 可由 `fallback` 统一处理。这种设计允许项目团队自由演变自己的提交习惯，无需等待 Nargo 更新。

#### 3.2.3 Scope 解析与包映射

从 commit 中提取 scope 的正则同样可配置（默认匹配括号内内容）。提取出的 scope 会通过 `[release.scope.mapping]` 映射到 workspace 中的具体包名（如 `"auth" -> "@scope/auth"`）。若未提供映射，则默认将 scope 作为包名的一部分。对于没有 scope 的 commit，将归入 `[release.scope.default]` 指定的包。

### 3.3 版本自动计算

版本计算遵循以下原则：

- 每个包的版本独立演化。
- 同一包若有多个 commit，按 `major > minor > patch > none` 取最高优先级（可配置为 cumulative，但实践中很少使用）。
- 若某包只有 `none` 类型的 commit，则版本不变，该包不会出现在 Release Notes 中（除非 `include` 设置为 true）。
- 预发布标签（如 `-alpha.1`）由 `[release.version.prerelease]` 控制，若设置，则所有包版本增加对应后缀。

计算出的新版本会写入 `[release.version.files]` 指定的文件中（如各 `package.json` 的 `version` 字段），确保版本号与代码始终保持一致。

### 3.4 Release Notes 生成

生成的质量直接决定发布体验。Nargo 内置了精美的默认模板，并支持完全自定义。

#### 3.4.1 默认模板结构

```markdown
# {{version}}

## 🚀 新特性 (Features)
{{#each features}}
### {{scope}}
{{#each commits}}
- {{subject}} ({{#if pr}}[#{{pr}}]({{pr_url}}){{/if}}) - 感谢 @{{author}}
{{/each}}
{{/each}}

## 🐛 Bug 修复 (Fixes)
...
## ⚠️ 破坏性变更 (Breaking Changes)
...

**完整变更对比**: [{{previous_tag}}...{{version}}]({{repo_url}}/compare/{{previous_tag}}...{{version}})

**参与贡献**: {{contributors}}
```

模板引擎使用 Handlebars，数据模型包含 `version`、`features`、`fixes`、`breaking` 等数组，每个元素包含 `scope`、`commits` 列表（含 `subject`、`pr`、`author` 等字段）。用户可通过 `[release.github.template]` 指定自己的 `.hbs` 文件路径。

#### 3.4.2 贡献者列表与 PR 关联

解析 commit 时，Nargo 会尝试从消息中提取 `Closes #123` 或 `Fixes #456` 等模式，关联 PR 号，并在 Release Notes 中生成超链接。同时，通过 Git 的 `--author` 信息收集贡献者用户名，去重后展示于底部，以表示对每一位贡献者的感谢。

### 3.5 GitHub Release 创建

使用 GitHub API 的 `POST /repos/{owner}/{repo}/releases` 端点。认证方式优先采用 OIDC（通过 GitHub Actions 的 `id-token`），若失败则 fallback 到 `GITHUB_TOKEN` 环境变量。发布前会检查 tag 是否已存在，若已存在则提示覆盖（需要 `--force` 选项）。所有 API 调用均支持超时和重试。

---

## 4. 配置体系：`Nargo.toml` 中的 Release 配置

所有与发布相关的配置都位于 `[release]` 及其子节下。以下列出全部可配置项，并给出示例：

```toml
[release]
# 核心模式：'from-commits'（默认）、'from-changesets'、'manual'
mode = "from-commits"

# 规范风格：'gitmoji'、'conventional'、'angular'、'custom'
style = "gitmoji"

# Git tag 前缀，例如 'v' 或 'release/'
tag-prefix = "v"

# 从哪个 commit 开始追溯：'last-tag'（默认）、'main'、'develop' 或具体 SHA
since = "last-tag"

# 是否自动创建 GitHub Release
auto-create = true

# 是否自动推送 tag 到远程
auto-push = true

# 发布前是否要求 CI 通过（需外部检查）
require-ci-pass = false

# 是否在 dry-run 模式下输出详细解释
explain = false

[release.spec]  # 仅当 style = 'custom' 时生效
[[release.spec.rules]]
name = "features"
pattern = "^:sparkles:"
version = "minor"
category = "🚀 新特性"

[[release.spec.rules]]
name = "breaking"
pattern = "^:boom:"
version = "major"
category = "⚠️ 破坏性变更"

[release.spec.fallback]
version = "patch"
category = "📦 其他变更"
include = true

[release.scope]
pattern = "\\(([^)]+)\\)"
default = "core"

[release.scope.mapping]
"auth" = "@scope/auth"
"core" = "@scope/core"

[release.version]
strategy = "highest"   # 或 "cumulative"
skip-patch = false
prerelease = ""        # 如 "alpha" 或 "beta"

[release.version.files]
"package.json" = "version"
"packages/*/package.json" = "version"

[release.github]
repository = "https://github.com/owner/repo"   # 自动检测时可不填
title = "v{{version}}"
template = ".github/release-template.hbs"      # 自定义模板
prerelease = false
generate-diff-link = true
list-contributors = true
link-prs = true

[release.checks]
pre = ["nargo fmt --check", "cargo test"]
post = []
```

这些配置项提供了足够的灵活性，同时也保持了合理的默认值，使得大多数项目只需指定 `style` 即可开箱即用。

---

## 5. CI/CD 集成与自举发布

### 5.1 在 CI 中使用 Nargo

Nargo 被设计为优先运行在 CI 环境（GitHub Actions、GitLab CI 等）中。它通过以下方式与 CI 深度融合：

- **OIDC 认证**：在 GitHub Actions 中，Nargo 自动使用 `ACTIONS_ID_TOKEN_REQUEST_URL` 获取 OIDC token，无需配置长期 `GITHUB_TOKEN` 即可创建 Release（最短权限原则）。
- **环境变量感知**：读取 `CI`、`GITHUB_REF` 等标准变量，自动推断仓库 URL、当前分支等信息。
- **缓存与幂等性**：解析 / Notes 基于确定性输入，多次运行结果一致，适合重试。

典型的 CI 工作流（**固定安装稳定版 nargo，禁止用待发布源码里的未构建 binding 跑 release**）：

```yaml
name: Release
on:
  push:
    branches: [main]
jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: write   # 用于创建 release
      id-token: write   # 用于 OIDC
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # 获取所有 tag 和 commit
      - name: Install stable Nargo (n)
        run: npm i -g nargo@1.0.0   # 固定上一稳定版，勿用 workspace 未发布绑定
      - name: Release (dry-run then real)
        run: |
          nargo release github --dry-run
          nargo release github
      - name: Build n+1 native artifacts
        run: |
          cargo build --release
          pnpm build   # 产出平台 .node
      - name: Upload assets to the Release just created
        run: gh release upload "v${{ env.VERSION }}" runtimes/nargo-linux-x64/*.node
```

若项目使用 pnpm，可将**稳定版** `nargo` 钉在 `devDependencies`，CI 用该钉死版本执行 release；**不要**用本次 checkout 里尚未构建的 `@nargo/nargo-*` 原生绑定跑发布命令。

### 5.2 自举发布：用稳定版 n 发布 n+1（正确模型）

**错误模型（已废弃）**：用尚未发布的 n+1 二进制「自测」再发布——n+1 可执行文件正是本次 Release 要产出的东西，不存在可用的 n+1。

**正确模型**：**用当前稳定版（n）解析 / 建 tag / 建 GitHub Release；n+1 源码由 CI 构建成 `.node` / 二进制后，作为 Release asset 上传。** 下次再用已安装的 n+1 去发布 n+2。

#### 5.2.1 流程示例（用 v1.0.0 发布 v2.0.0）

```text
1. 开发者提交 v2.0.0 源码（新功能 / 新配置项）
2. CI 触发 Release workflow
3. 安装并使用 v1.0.0（稳定版）：
     nargo release github --dry-run
     # 解析指向 v2.0.0 的 commits，预览 Notes
     # v1.0.0 能理解规范（规范在 Nargo.toml，字段可向前兼容忽略）
4. 确认后仍用 v1.0.0：
     nargo release github
     # 创建 tag、Notes、GitHub Release
5. CI 构建 v2.0.0 产物：
     cargo build --release / pnpm build → *.node
6. 将 v2.0.0 产物作为 asset 挂到刚创建的 Release
7. 用户安装 v2.0.0
8. 下一轮：用 v2.0.0 发布 v3.0.0
```

#### 5.2.2 发布逻辑与执行产物分离

```text
发布流程（旧版 nargo 执行）
  nargo release github
  ├── 解析 commit（规范配置可被旧版读懂 / 忽略未知字段）
  ├── 计算版本 / 生成 Notes
  ├── 创建 GitHub Release + tag
  └── 附加 artifact: nargo-v2.0.0 的 .node（由后续构建步上传）

新版二进制（CI 构建，非发布引擎）
  *.node / native lib
  → 仅作为 asset，供用户下载安装
```

CI 顺序必须避免死锁：

```text
1. checkout（含 n+1 源码）
2. 安装旧版 nargo（n，来自 npm）
3. 用旧版执行 release        ← 发布流程
4. 构建新版 native binding   ← 构建流程（不可被第 3 步调用）
5. 上传 binding 到 Release
```

第 3 步与第 4 步不可颠倒成「先用第 4 步产物跑第 3 步」。

#### 5.2.3 版本兼容性契约（自举场景）

旧版必须能处理「新版仓库里的配置与 commit」：

| 配置/行为 | 兼容性策略 |
|----------|----------|
| Commit 规范 | 规则写在 `Nargo.toml`；旧版能解析配置即可解析 commit |
| 新增配置字段 | 旧版忽略未知字段（serde `default` / deny_unknown_fields 关闭） |
| 移除配置字段 | 旧版已支持则无妨 |
| Release Notes 模板 | 独立 `.hbs`；旧版同样渲染 |
| 新增 CLI 参数 | 旧版调用路径不传这些参数即可 |

SemVer 对 `release github` 的承诺：

- **Patch**：不改配置结构与 CLI 契约  
- **Minor**：可增字段，旧配置仍可用  
- **Major**：允许破坏性配置 / CLI 变更（发布前需先升级钉死的稳定版）

#### 5.2.4 锁定发布时的 Nargo 版本

CI 与 `package.json` **钉死稳定版**（如 `nargo@1.0.0` / `npx nargo@1.0.0`），禁止 `latest` 漂移，避免自举中途换引擎。

#### 5.2.5 回滚与故障恢复

`nargo release github --revert` 可撤销最近的 Release（删 tag + GitHub Release），并重新生成正确版本（需额外权限）。

### 5.3 幂等性与安全性

创建前检查远程是否已有同名 tag；未指定 `--force` 则退出，避免覆盖。Notes 仅依赖 commit 图时，重跑输出一致。

---

## 6. 安全与权限

### 6.1 Token 最小权限

优先 OIDC（`id-token: write`）。必须用 token 时，经 `GITHUB_TOKEN` 注入，权限限 `contents: write`。

### 6.2 发布前检查清单

`[release.checks.pre]` 中的命令在任何变更前执行；任一失败则终止。典型：`nargo fmt --check`、测试、构建、分支守卫。

### 6.3 `--dry-run` 模式

执行解析 / 计算 / 生成，不写远程。建议 CI 先 dry-run，把 Notes 贴到 PR 评论做透明审计。

---

## 7. 性能与可靠性

### 7.1 Rust + NAPI

核心解析与计算在 Rust，经 NAPI-RS 暴露。大型 Monorepo 下相对纯 Node 工具有数量级优势（白皮书基准：约 5000 commits / Notes ≤ 200ms 量级）。

### 7.2 错误处理与超时

GitHub API：指数退避重试 + 默认 30s 超时；`--debug` 输出详细日志。

### 7.3 确定性输出

Notes 生成不依赖外部服务作为硬前提（PR 链接增强可降级）。API 不可用时仍可 `--dry-run` 拿到本地 Markdown，再手工粘贴。

---

## 8. 与其他工具对比

| 工具 | 配置复杂度 | 贡献者负担 | 版本自动计算 | Release Notes 质量 | CI 自举支持 |
|------|-----------|-----------|-------------|-------------------|------------|
| `standard-version` | 中等 | 需 conventional | 部分 | 一般 | 弱 |
| `changesets` | 高 | 需写 .md | 手动 | 优秀 | 一般 |
| `semantic-release` | 高 | 需配插件 | 自动 | 可定制 | 强 |
| **Nargo** | **低** | **极低** | **完全自动** | **精美+可定制** | **原生支持** |

Nargo 的独特优势：不仅是发布工具，更是 Monorepo 工程链的一部分（fmt、bump、trust 等），共享同一套 WorkspaceContext 与配置。

---

## 9. 未来展望

- 支持 GitLab / Gitee 等（抽象 Release Provider）
- squash merge 后自动提取最终 message
- AI 变更摘要（可选）
- 与 npm 发布编排：Release 后接 `nargo trust` / publish（分轨协作）

核心原则不变：**配置驱动、规范驱动、零摩擦**。

---

## 10. 结语

`nargo release github` 是最能体现「npm 上的 Cargo」愿景的功能之一：把发布脚本收成一行命令，把 commit 规范变成确定性的版本与 Notes，把人工判断交给规则引擎。通过灵活配置、安全自举和深度 CI 集成，为 Monorepo 维护者提供可预期的发布体验。

---

## 附录 A：本阶段实现优先级

1. `WorkspaceContext` 自动探测 + pnpm 成员 / scope 映射  
2. Commit Parser + 内置 `gitmoji` / `conventional` + `Nargo.toml` `[release]` 最小 schema  
3. Version Computer（`highest`）+ `--dry-run` Notes  
4. 写回 `package.json` version（Oak）  
5. GitHub Release（OIDC → `GITHUB_TOKEN` fallback）  
6. `--revert` / checks.pre / **自举：钉死稳定版 n 执行 release，n+1 `.node` 仅作 asset**  
7. 与 `nargo trust` 编排（后置）
