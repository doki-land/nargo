//! Nargo MCP Tools
//!
//! This module defines and implements all the tools available through
//! the Nargo MCP server for package management operations.

use std::collections::HashMap;

use crate::types::*;
use nargo_types::NargoValue;

/// Returns all available Nargo tools.
///
/// Each tool represents a Nargo CLI command that can be invoked
/// through the MCP protocol.
///
/// # Available Tools
///
/// - `nargo_add` - Add dependencies
/// - `nargo_remove` - Remove dependencies
/// - `nargo_update` - Update dependencies
/// - `nargo_build` - Build the project
/// - `nargo_test` - Run tests
/// - `nargo_lint` - Run linter
/// - `nargo_fmt` - Format code
/// - `nargo_info` - Get package info
/// - `nargo_tree` - Show dependency tree
/// - `nargo_new` - Create new project
/// - `nargo_init` - Initialize project
/// - `nargo_publish` - Publish package
/// - `nargo_search` - Search packages
/// - `nargo_run` - Run scripts
/// - `nargo_analyze` - Analyze project
pub fn get_nargo_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "nargo_add".to_string(),
            description: "添加依赖包到项目中。类似于 cargo add，支持版本指定和特性选择。"
                .to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "package".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some(
                                "包名称，例如 'lodash' 或 'react@18.0.0'".to_string(),
                            ),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "features".to_string(),
                        PropertySchema {
                            prop_type: Some("array".to_string()),
                            description: Some("要启用的特性列表".to_string()),
                            items: Some(Box::new(PropertySchema {
                                prop_type: Some("string".to_string()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "dev".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否作为开发依赖添加".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "optional".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否作为可选依赖添加".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: Some(vec!["package".to_string()]),
            },
        },
        Tool {
            name: "nargo_remove".to_string(),
            description: "从项目中移除依赖包。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "package".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("要移除的包名称".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "dev".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否从开发依赖中移除".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: Some(vec!["package".to_string()]),
            },
        },
        Tool {
            name: "nargo_update".to_string(),
            description: "更新项目依赖到最新版本。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "package".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some(
                                "要更新的包名称（可选，不指定则更新所有）".to_string(),
                            ),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "precise".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否精确更新到锁文件中的版本".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_build".to_string(),
            description: "构建项目，生成目标产物到 /target 目录。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "release".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否以 release 模式构建".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "target".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("目标平台，例如 'web'、'node'、'wasm'".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "watch".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否启用监听模式".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_test".to_string(),
            description: "运行项目测试。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "filter".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("测试名称过滤器".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "coverage".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否生成覆盖率报告".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "parallel".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否并行运行测试".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_lint".to_string(),
            description: "对项目代码进行静态分析检查。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "fix".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否自动修复问题".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "rules".to_string(),
                        PropertySchema {
                            prop_type: Some("array".to_string()),
                            description: Some("要应用的规则列表".to_string()),
                            items: Some(Box::new(PropertySchema {
                                prop_type: Some("string".to_string()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_fmt".to_string(),
            description: "格式化项目代码。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "check".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("仅检查格式，不修改文件".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "files".to_string(),
                        PropertySchema {
                            prop_type: Some("array".to_string()),
                            description: Some("要格式化的文件列表".to_string()),
                            items: Some(Box::new(PropertySchema {
                                prop_type: Some("string".to_string()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_info".to_string(),
            description: "获取包或项目的详细信息。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "package".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("包名称（可选，不指定则显示项目信息）".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_tree".to_string(),
            description: "显示依赖树结构。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "depth".to_string(),
                        PropertySchema {
                            prop_type: Some("number".to_string()),
                            description: Some("显示深度".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "duplicates".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("是否显示重复依赖".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_new".to_string(),
            description: "创建新的 Nargo 项目。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "name".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("项目名称".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "template".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some(
                                "项目模板，例如 'lib'、'app'、'component'".to_string(),
                            ),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "path".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("创建路径（可选）".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: Some(vec!["name".to_string()]),
            },
        },
        Tool {
            name: "nargo_init".to_string(),
            description: "在当前目录初始化 Nargo 项目。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "name".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("项目名称（可选，默认使用目录名）".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "template".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("项目模板".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_publish".to_string(),
            description: "发布包到注册表。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "registry".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("目标注册表 URL（可选）".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "dry_run".to_string(),
                        PropertySchema {
                            prop_type: Some("boolean".to_string()),
                            description: Some("模拟发布，不实际上传".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "access".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("访问级别：'public' 或 'restricted'".to_string()),
                            enum_values: Some(vec!["public".to_string(), "restricted".to_string()]),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
        Tool {
            name: "nargo_search".to_string(),
            description: "搜索注册表中的包。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "query".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("搜索关键词".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "limit".to_string(),
                        PropertySchema {
                            prop_type: Some("number".to_string()),
                            description: Some("返回结果数量限制".to_string()),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: Some(vec!["query".to_string()]),
            },
        },
        Tool {
            name: "nargo_run".to_string(),
            description: "运行项目脚本或二进制文件。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "script".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("脚本名称或命令".to_string()),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "args".to_string(),
                        PropertySchema {
                            prop_type: Some("array".to_string()),
                            description: Some("传递给脚本的参数".to_string()),
                            items: Some(Box::new(PropertySchema {
                                prop_type: Some("string".to_string()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: Some(vec!["script".to_string()]),
            },
        },
        Tool {
            name: "nargo_analyze".to_string(),
            description: "分析项目代码质量和结构。".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert(
                        "scope".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some(
                                "分析范围：'all'、'dependencies'、'security'、'performance'"
                                    .to_string(),
                            ),
                            enum_values: Some(vec![
                                "all".to_string(),
                                "dependencies".to_string(),
                                "security".to_string(),
                                "performance".to_string(),
                            ]),
                            ..Default::default()
                        },
                    );
                    props.insert(
                        "output".to_string(),
                        PropertySchema {
                            prop_type: Some("string".to_string()),
                            description: Some("输出格式：'json'、'markdown'、'html'".to_string()),
                            enum_values: Some(vec![
                                "json".to_string(),
                                "markdown".to_string(),
                                "html".to_string(),
                            ]),
                            ..Default::default()
                        },
                    );
                    props
                }),
                required: None,
            },
        },
    ]
}

/// Executes a tool by name with the given arguments.
///
/// # Arguments
///
/// * `name` - The name of the tool to execute.
/// * `arguments` - Optional arguments for the tool.
///
/// # Returns
///
/// Result containing the tool execution result or an error message.
pub fn execute_tool(
    name: &str,
    arguments: Option<HashMap<String, NargoValue>>,
) -> Result<CallToolResult, String> {
    let args = arguments.unwrap_or_default();

    match name {
        "nargo_add" => {
            let package = args
                .get("package")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'package' parameter")?;
            let dev = args.get("dev").and_then(|v| v.as_bool()).unwrap_or(false);
            let optional = args
                .get("optional")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let features: Vec<String> = args
                .get("features")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "添加依赖: {}{}\n特性: {:?}\n开发依赖: {}\n可选依赖: {}",
                        package,
                        if dev { " (dev)" } else { "" },
                        features,
                        dev,
                        optional
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_remove" => {
            let package = args
                .get("package")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'package' parameter")?;
            let dev = args.get("dev").and_then(|v| v.as_bool()).unwrap_or(false);

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "移除依赖: {}{}",
                        package,
                        if dev { " (dev)" } else { "" }
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_build" => {
            let release = args
                .get("release")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("web");
            let watch = args.get("watch").and_then(|v| v.as_bool()).unwrap_or(false);

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "构建项目\n模式: {}\n目标: {}\n监听: {}",
                        if release { "release" } else { "debug" },
                        target,
                        watch
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_test" => {
            let filter = args.get("filter").and_then(|v| v.as_str()).unwrap_or("");
            let coverage = args
                .get("coverage")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let parallel = args
                .get("parallel")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "运行测试\n过滤器: '{}'\n覆盖率: {}\n并行: {}",
                        filter, coverage, parallel
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_lint" => {
            let fix = args.get("fix").and_then(|v| v.as_bool()).unwrap_or(false);
            let rules: Vec<String> = args
                .get("rules")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "代码检查\n自动修复: {}\n规则: {:?}",
                        fix,
                        if rules.is_empty() {
                            vec!["all".to_string()]
                        } else {
                            rules
                        }
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_fmt" => {
            let check = args.get("check").and_then(|v| v.as_bool()).unwrap_or(false);
            let files: Vec<String> = args
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "格式化代码\n仅检查: {}\n文件: {}",
                        check,
                        if files.is_empty() {
                            "全部".to_string()
                        } else {
                            files.join(", ")
                        }
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_info" => {
            let package = args.get("package").and_then(|v| v.as_str());

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: if let Some(pkg) = package {
                        format!("包信息: {}", pkg)
                    } else {
                        "项目信息".to_string()
                    },
                }],
                is_error: Some(false),
            })
        }
        "nargo_tree" => {
            let depth = args
                .get("depth")
                .and_then(|v| v.as_number())
                .unwrap_or(10.0) as u64;
            let duplicates = args
                .get("duplicates")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!("依赖树\n深度: {}\n显示重复: {}", depth, duplicates),
                }],
                is_error: Some(false),
            })
        }
        "nargo_new" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'name' parameter")?;
            let template = args
                .get("template")
                .and_then(|v| v.as_str())
                .unwrap_or("app");
            let path = args.get("path").and_then(|v| v.as_str());

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "创建新项目\n名称: {}\n模板: {}\n路径: {}",
                        name,
                        template,
                        path.unwrap_or(".")
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_init" => {
            let name = args.get("name").and_then(|v| v.as_str());
            let template = args
                .get("template")
                .and_then(|v| v.as_str())
                .unwrap_or("app");

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "初始化项目\n名称: {}\n模板: {}",
                        name.unwrap_or("当前目录"),
                        template
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_publish" => {
            let registry = args.get("registry").and_then(|v| v.as_str());
            let dry_run = args
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let access = args
                .get("access")
                .and_then(|v| v.as_str())
                .unwrap_or("public");

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "发布包\n注册表: {}\n模拟: {}\n访问级别: {}",
                        registry.unwrap_or("默认注册表"),
                        dry_run,
                        access
                    ),
                }],
                is_error: Some(false),
            })
        }
        "nargo_search" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'query' parameter")?;
            let limit = args
                .get("limit")
                .and_then(|v| v.as_number())
                .unwrap_or(10.0) as u64;

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!("搜索包: '{}'\n限制: {} 条结果", query, limit),
                }],
                is_error: Some(false),
            })
        }
        "nargo_run" => {
            let script = args
                .get("script")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'script' parameter")?;
            let script_args: Vec<String> = args
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!("运行脚本: {} {}", script, script_args.join(" ")),
                }],
                is_error: Some(false),
            })
        }
        "nargo_analyze" => {
            let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("all");
            let output = args
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("json");

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!("分析项目\n范围: {}\n输出格式: {}", scope, output),
                }],
                is_error: Some(false),
            })
        }
        "nargo_update" => {
            let package = args.get("package").and_then(|v| v.as_str());
            let precise = args
                .get("precise")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Ok(CallToolResult {
                content: vec![Content::Text {
                    text: format!(
                        "更新依赖\n包: {}\n精确模式: {}",
                        package.unwrap_or("全部"),
                        precise
                    ),
                }],
                is_error: Some(false),
            })
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}
