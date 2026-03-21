#![warn(missing_docs)]

use nargo_types::Result;
use std::{fs, path::Path};

/// 项目模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    /// 基础项目模板
    Basic,
    /// 文档项目模板
    Document,
    /// 全栈项目模板
    FullStack,
}

impl TemplateType {
    /// 获取所有可用的模板类型
    pub fn all() -> &'static [Self] {
        &[Self::Basic, Self::Document, Self::FullStack]
    }

    /// 获取模板的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Basic => "基础项目",
            Self::Document => "文档项目",
            Self::FullStack => "全栈项目",
        }
    }

    /// 获取模板的描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Basic => "简单的前端项目，包含基础配置",
            Self::Document => "文档站点项目，基于 Nargo-document",
            Self::FullStack => "全栈项目，包含前后端配置",
        }
    }
}

/// 项目配置
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// 项目名称
    pub name: String,
    /// 模板类型
    pub template: TemplateType,
    /// 是否使用 TypeScript
    pub use_typescript: bool,
    /// 作者名称
    pub author: Option<String>,
    /// 描述
    pub description: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self { name: "my-nargo-project".to_string(), template: TemplateType::Basic, use_typescript: true, author: None, description: None }
    }
}

/// 通过交互式提示获取项目配置
#[cfg(feature = "interactive")]
pub fn interactive_config(default_name: &str) -> Result<ProjectConfig> {
    use dialoguer::{Confirm, Input, Select};

    let name = Input::new().with_prompt("项目名称").default(default_name.to_string()).interact_text()?;

    let templates = TemplateType::all();
    let template_names: Vec<_> = templates.iter().map(|t| format!("{} - {}", t.display_name(), t.description())).collect();

    let template_index = Select::new().with_prompt("选择项目模板").items(&template_names).default(0).interact()?;

    let use_typescript = Confirm::new().with_prompt("使用 TypeScript").default(true).interact()?;

    let author: String = Input::new().with_prompt("作者名称（可选）").allow_empty(true).interact_text()?;

    let author = if author.is_empty() { None } else { Some(author) };

    let description: String = Input::new().with_prompt("项目描述（可选）").allow_empty(true).interact_text()?;

    let description = if description.is_empty() { None } else { Some(description) };

    Ok(ProjectConfig { name, template: templates[template_index], use_typescript, author, description })
}

/// 使用非交互式初始化项目
pub fn init(name: &str) -> Result<()> {
    init_with_config(ProjectConfig { name: name.to_string(), ..Default::default() })
}

/// 使用指定配置初始化项目
pub fn init_with_config(config: ProjectConfig) -> Result<()> {
    let root = Path::new(&config.name);

    match config.template {
        TemplateType::Basic => generate_basic_project(&config, root)?,
        TemplateType::Document => generate_document_project(&config, root)?,
        TemplateType::FullStack => generate_fullstack_project(&config, root)?,
    }

    Ok(())
}

/// 生成基础项目
fn generate_basic_project(config: &ProjectConfig, root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("public"))?;

    let ext = if config.use_typescript { "ts" } else { "js" };

    fs::write(root.join(format!("src/main.{}", ext)), if config.use_typescript { "console.log('Hello Nargo!');\n" } else { "console.log('Hello Nargo!');\n" })?;

    fs::write(
        root.join("index.nargo"),
        r#"<template>
  <div>
    <h1>Hello Nargo!</h1>
    <p>欢迎使用 Nargo 项目</p>
  </div>
</template>
"#,
    )?;

    generate_package_json(config, root)?;
    generate_readme(config, root, "基础项目")?;
    generate_gitignore(root)?;

    Ok(())
}

/// 生成文档项目
fn generate_document_project(config: &ProjectConfig, root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("docs"))?;
    fs::create_dir_all(root.join("public"))?;

    fs::write(
        root.join("docs/index.md"),
        format!(
            r#"# {}

{}

## 快速开始

```bash
nargo doc
```
"#,
            config.name,
            config.description.as_deref().unwrap_or("欢迎使用 Nargo-document")
        ),
    )?;

    fs::write(
        root.join("nargodoc.config.toml"),
        r#"[project]
name = "文档项目"
description = "基于 Nargo-document 生成的文档"

[theme]
name = "default"
"#,
    )?;

    generate_package_json(config, root)?;
    generate_readme(config, root, "文档项目")?;
    generate_gitignore(root)?;

    Ok(())
}

/// 生成全栈项目
fn generate_fullstack_project(config: &ProjectConfig, root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src/client"))?;
    fs::create_dir_all(root.join("src/server"))?;
    fs::create_dir_all(root.join("public"))?;

    let ext = if config.use_typescript { "ts" } else { "js" };

    fs::write(root.join(format!("src/client/main.{}", ext)), "console.log('Client initialized');\n")?;

    fs::write(root.join(format!("src/server/main.{}", ext)), "console.log('Server initialized');\n")?;

    fs::write(
        root.join("index.nargo"),
        r#"<template>
  <div>
    <h1>全栈 Nargo 项目</h1>
    <p>包含客户端和服务端</p>
  </div>
</template>
"#,
    )?;

    generate_package_json(config, root)?;
    generate_readme(config, root, "全栈项目")?;
    generate_gitignore(root)?;

    Ok(())
}

/// 生成 package.json
fn generate_package_json(config: &ProjectConfig, root: &Path) -> Result<()> {
    let mut pkg = serde_json::json!({
        "name": config.name,
        "version": "0.1.0",
        "type": "module",
    });

    if let Some(author) = &config.author {
        pkg["author"] = serde_json::json!(author);
    }

    if let Some(description) = &config.description {
        pkg["description"] = serde_json::json!(description);
    }

    let content = serde_json::to_string_pretty(&pkg).map_err(|e| nargo_types::Error::external_error("serde_json".to_string(), e.to_string(), nargo_types::Span::unknown()))?;

    fs::write(root.join("package.json"), content)?;

    Ok(())
}

/// 生成 README.md
fn generate_readme(config: &ProjectConfig, root: &Path, template_name: &str) -> Result<()> {
    let content = format!(
        r#"# {}

{}

## 项目类型

{}

## 快速开始

```bash
# 安装依赖
pnpm install

# 开发模式
nargo dev

# 构建项目
nargo build
```

## 许可证

MIT
"#,
        config.name,
        config.description.as_deref().unwrap_or("一个使用 Nargo 构建的项目"),
        template_name
    );

    fs::write(root.join("README.md"), content)?;

    Ok(())
}

/// 生成 .gitignore
fn generate_gitignore(root: &Path) -> Result<()> {
    fs::write(
        root.join(".gitignore"),
        r#"node_modules/
dist/
build/
.env
.env.local
.DS_Store
*.log
"#,
    )?;

    Ok(())
}
