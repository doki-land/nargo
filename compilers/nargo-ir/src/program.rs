#![warn(missing_docs)]

//! 程序模块
//!
//! 提供 JavaScript 程序和 IR 模块的表示和相关功能。

use nargo_types::{NargoValue, Result, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    stmt::JsStmt,
    template::{CustomBlockIR, StyleIR, TemplateIR, TestIR},
    types::{IRError, MAX_ARRAY_LENGTH, MAX_OBJECT_SIZE, MAX_STRING_LENGTH, Trivia},
};

/// JavaScript 程序
///
/// 表示一个完整的 JavaScript 程序，包含多个语句
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct JsProgram {
    /// 程序体
    ///
    /// 包含程序的所有语句
    pub body: Vec<JsStmt>,
    /// 位置信息
    ///
    /// 程序在源文件中的位置
    #[serde(default)]
    pub span: Span,
    /// Trivia 信息
    ///
    /// 包含程序的空白和注释信息
    #[serde(default)]
    pub trivia: Trivia,
}

impl JsProgram {
    /// 验证程序的有效性
    ///
    /// 检查程序是否符合大小限制，并验证所有语句的有效性
    ///
    /// # Returns
    /// - `Ok(())` 如果程序有效
    /// - `Err(Error)` 如果程序无效
    pub fn validate(&self) -> Result<()> {
        if self.body.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Program body length exceeded".to_string()).into());
        }
        for stmt in &self.body {
            stmt.validate(0)?;
        }
        Ok(())
    }

    /// 优化程序
    pub fn optimize(&mut self) {
        crate::optimizer::ProgramOptimizer::optimize(self);
    }

    /// 检查程序是否为空
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
}

/// IR 模块
///
/// 表示一个完整的 HXO IR 模块，包含脚本、模板、样式等信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IRModule {
    /// 模块名称
    ///
    /// 模块的唯一标识符
    pub name: String,
    /// 元数据
    ///
    /// 模块的附加信息
    pub metadata: HashMap<String, NargoValue>,
    /// 脚本
    ///
    /// 通用脚本代码
    pub script: Option<JsProgram>,
    /// 服务端脚本
    ///
    /// 仅在服务端执行的脚本代码
    pub script_server: Option<JsProgram>,
    /// 客户端脚本
    ///
    /// 仅在客户端执行的脚本代码
    pub script_client: Option<JsProgram>,
    /// 脚本元数据
    ///
    /// 用于脚本分析结果的元数据
    pub script_meta: Option<NargoValue>,
    /// 模板
    ///
    /// 模块的模板结构
    pub template: Option<TemplateIR>,
    /// 提升节点
    ///
    /// 被提升的模板节点
    pub hoisted_nodes: HashMap<String, crate::template::TemplateNodeIR>,
    /// 样式
    ///
    /// 模块的样式定义
    pub styles: Vec<StyleIR>,
    /// 国际化
    ///
    /// 模块的国际化文本
    pub i18n: Option<HashMap<String, HashMap<String, String>>>,
    /// WASM
    ///
    /// WebAssembly 模块
    pub wasm: Vec<Vec<u8>>,
    /// 自定义块
    ///
    /// 模块的自定义块
    pub custom_blocks: Vec<CustomBlockIR>,
    /// 测试
    ///
    /// 模块的测试用例
    pub tests: Vec<TestIR>,
    /// 依赖关系
    ///
    /// 模块依赖的其他模块
    pub dependencies: Vec<String>,
    /// 被依赖关系
    ///
    /// 依赖此模块的其他模块
    pub dependents: Vec<String>,
    /// 位置信息
    ///
    /// 模块在源文件中的位置
    pub span: Span,
}

impl IRModule {
    /// 创建一个新的 IR 模块
    ///
    /// # Parameters
    /// - `name`: 模块名称
    pub fn new(name: String) -> Self {
        Self { name, metadata: HashMap::with_capacity(16), script: None, script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::with_capacity(16), styles: Vec::with_capacity(8), i18n: None, wasm: Vec::with_capacity(4), custom_blocks: Vec::with_capacity(4), tests: Vec::with_capacity(4), dependencies: Vec::with_capacity(8), dependents: Vec::with_capacity(8), span: Span::unknown() }
    }

    /// 验证 IR 模块的有效性
    pub fn validate(&self) -> Result<()> {
        if self.name.len() > MAX_STRING_LENGTH {
            return Err(IRError::SizeLimitExceeded("Module name length exceeded".to_string()).into());
        }

        // 验证元数据
        if self.metadata.len() > MAX_OBJECT_SIZE {
            return Err(IRError::SizeLimitExceeded("Metadata size exceeded".to_string()).into());
        }
        for (key, value) in &self.metadata {
            if key.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Metadata key length exceeded".to_string()).into());
            }
            value.validate(0)?;
        }

        // 验证脚本
        if let Some(script) = &self.script {
            script.validate()?;
        }
        if let Some(script_server) = &self.script_server {
            script_server.validate()?;
        }
        if let Some(script_client) = &self.script_client {
            script_client.validate()?;
        }

        // 验证脚本元数据
        if let Some(meta) = &self.script_meta {
            meta.validate(0)?;
        }

        // 验证模板
        if let Some(template) = &self.template {
            template.validate()?;
        }

        // 验证提升节点
        if self.hoisted_nodes.len() > MAX_OBJECT_SIZE {
            return Err(IRError::SizeLimitExceeded("Hoisted nodes size exceeded".to_string()).into());
        }
        for (key, node) in &self.hoisted_nodes {
            if key.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("Hoisted node key length exceeded".to_string()).into());
            }
            node.validate(0)?;
        }

        // 验证样式
        if self.styles.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Styles length exceeded".to_string()).into());
        }
        for style in &self.styles {
            style.validate()?;
        }

        // 验证国际化
        if let Some(i18n) = &self.i18n {
            if i18n.len() > MAX_OBJECT_SIZE {
                return Err(IRError::SizeLimitExceeded("I18n size exceeded".to_string()).into());
            }
            for (lang, translations) in i18n {
                if lang.len() > MAX_STRING_LENGTH {
                    return Err(IRError::SizeLimitExceeded("I18n language length exceeded".to_string()).into());
                }
                if translations.len() > MAX_OBJECT_SIZE {
                    return Err(IRError::SizeLimitExceeded("I18n translations size exceeded".to_string()).into());
                }
                for (key, value) in translations {
                    if key.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("I18n key length exceeded".to_string()).into());
                    }
                    if value.len() > MAX_STRING_LENGTH {
                        return Err(IRError::SizeLimitExceeded("I18n value length exceeded".to_string()).into());
                    }
                }
            }
        }

        // 验证 WASM
        if self.wasm.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("WASM length exceeded".to_string()).into());
        }
        for wasm in &self.wasm {
            if wasm.len() > MAX_STRING_LENGTH {
                return Err(IRError::SizeLimitExceeded("WASM size exceeded".to_string()).into());
            }
        }

        // 验证自定义块
        if self.custom_blocks.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Custom blocks length exceeded".to_string()).into());
        }
        for block in &self.custom_blocks {
            block.validate()?;
        }

        // 验证测试
        if self.tests.len() > MAX_ARRAY_LENGTH {
            return Err(IRError::SizeLimitExceeded("Tests length exceeded".to_string()).into());
        }
        for test in &self.tests {
            test.validate()?;
        }

        Ok(())
    }

    /// 清理模块中的空元素
    pub fn cleanup(&mut self) {
        // 清理脚本
        if let Some(script) = &mut self.script {
            if script.body.is_empty() {
                self.script = None;
            }
        }
        if let Some(script_server) = &mut self.script_server {
            if script_server.body.is_empty() {
                self.script_server = None;
            }
        }
        if let Some(script_client) = &mut self.script_client {
            if script_client.body.is_empty() {
                self.script_client = None;
            }
        }

        // 清理脚本元数据
        if let Some(meta) = &self.script_meta {
            if *meta == NargoValue::Null {
                self.script_meta = None;
            }
        }

        // 清理模板
        if let Some(template) = &mut self.template {
            if template.nodes.is_empty() {
                self.template = None;
            }
        }

        // 清理国际化
        if let Some(i18n) = &self.i18n {
            if i18n.is_empty() {
                self.i18n = None;
            }
        }

        // 清理依赖关系
        if self.dependencies.is_empty() {
            self.dependencies = Vec::new();
        }
        if self.dependents.is_empty() {
            self.dependents = Vec::new();
        }
    }

    /// 优化模块
    pub fn optimize(&mut self) {
        crate::optimizer::IROptimizer::optimize(self);
    }
}
