#![warn(missing_docs)]

//! API元数据提取和管理模块
//!
//! 提供API元数据的提取、解析和管理功能，支持从源代码中提取API端点信息。

use nargo_ir::IRModule;
use nargo_types::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API元数据
///
/// 包含API的端点列表和类型定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiMetadata {
    /// API端点列表
    pub endpoints: Vec<ApiEndpoint>,
    /// 类型定义
    pub types: HashMap<String, TypeDefinition>,
    /// SDK版本
    pub version: String,
}

/// API端点
///
/// 表示一个API端点的完整信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiEndpoint {
    /// 控制器名称
    pub controller: String,
    /// 函数名称
    pub name: String,
    /// HTTP方法
    pub method: String,
    /// 路径
    pub path: String,
    /// 参数列表
    pub params: Vec<ApiParam>,
    /// 返回类型
    pub return_type: Option<String>,
    /// 是否异步
    pub is_async: bool,
}

/// API参数
///
/// 表示API端点的一个参数信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiParam {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub param_type: ApiParamType,
    /// 类型名称
    pub type_name: String,
    /// 提取器参数
    pub extractor_param: Option<String>,
}

/// API参数类型
///
/// 表示API参数的提取类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ApiParamType {
    /// 请求体
    Body,
    /// 路径参数
    Path,
    /// 查询参数
    Query,
    /// 请求头
    Header,
    /// 当前用户
    CurrentUser,
    /// 会话
    Session,
    /// 国际化
    I18n,
    /// 普通参数
    #[default]
    Regular,
}

/// 类型定义
///
/// 表示一个自定义类型的定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeDefinition {
    /// 类型名称
    pub name: String,
    /// 字段列表
    pub fields: Vec<TypeField>,
}

/// 类型字段
///
/// 表示类型的一个字段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub type_name: String,
    /// 是否可选
    pub optional: bool,
}

/// API元数据提取器
///
/// 负责从源代码和IR中提取API元数据
pub struct ApiMetadataExtractor;

impl ApiMetadataExtractor {
    /// 提取API元数据（从源文本和IR）
    ///
    /// # Arguments
    /// * `source` - 源代码文本
    /// * `_ir` - IR模块
    ///
    /// # Returns
    /// * `Result<ApiMetadata>` - 提取的API元数据
    pub fn extract(source: &str, _ir: &IRModule) -> Result<ApiMetadata> {
        let mut metadata = ApiMetadata::default();

        Self::extract_endpoints_from_source(source, &mut metadata)?;
        Self::extract_types_from_source(source, &mut metadata)?;

        // 设置默认版本
        metadata.version = "1.0.0".to_string();

        Ok(metadata)
    }

    /// 从源文本中提取API端点
    ///
    /// # Arguments
    /// * `source` - 源代码文本
    /// * `metadata` - 元数据对象
    ///
    /// # Returns
    /// * `Result<()>` - 操作结果
    fn extract_endpoints_from_source(source: &str, metadata: &mut ApiMetadata) -> Result<()> {
        let http_decorator_re = Regex::new(r#"@http\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]\s*\)"#)?;
        let export_function_re = Regex::new(r#"(async)?\s*function\s+(\w+)\s*\(([\s\S]*?)\)\s*(?::\s*([^\{]+?))?\s*\{"#)?;
        let export_async_function_re = Regex::new(r#"export\s+(async)\s+function\s+(\w+)\s*\(([\s\S]*?)\)\s*(?::\s*([^\{]+?))?\s*\{"#)?;
        let class_method_re = Regex::new(r#"(async)?\s*(\w+)\s*\(([\s\S]*?)\)\s*(?::\s*([^\{]+?))?\s*\{"#)?;
        let class_def_re = Regex::new(r#"class\s+(\w+)\s*\{"#)?;

        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        let mut current_controller = "Default".to_string();

        while i < lines.len() {
            let line = lines[i];

            if let Some(class_caps) = class_def_re.captures(line) {
                let class_name = class_caps[1].to_string();
                if class_name.ends_with("Controller") {
                    current_controller = class_name;
                }
            }

            if let Some(captures) = http_decorator_re.captures(line) {
                let method = captures[1].to_string().to_uppercase();
                let path = captures[2].to_string();

                i += 1;
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                if i < lines.len() {
                    let next_line = lines[i];

                    if let Some(func_captures) = export_async_function_re.captures(&format!("{}{}", next_line, Self::collect_until_brace(&lines, i))) {
                        let is_async = func_captures.get(1).is_some();
                        let full_name = func_captures[2].to_string();
                        let (controller, name) = Self::parse_controller_and_name(&full_name);
                        let params_str = func_captures[3].to_string();
                        let return_type = func_captures.get(4).map(|m| m.as_str().trim().to_string());

                        let params = Self::parse_params(&params_str);

                        metadata.endpoints.push(ApiEndpoint { controller: if controller.is_empty() { current_controller.clone() } else { controller }, name, method, path, params, return_type, is_async });
                    }
                    else if let Some(func_captures) = class_method_re.captures(&format!("{}{}", next_line, Self::collect_until_brace(&lines, i))) {
                        let is_async = func_captures.get(1).is_some();
                        let name = func_captures[2].to_string();
                        let params_str = func_captures[3].to_string();
                        let return_type = func_captures.get(4).map(|m| m.as_str().trim().to_string());

                        let params = Self::parse_params(&params_str);

                        if !name.starts_with("constructor") && !name.starts_with('_') {
                            metadata.endpoints.push(ApiEndpoint { controller: current_controller.clone(), name, method, path, params, return_type, is_async });
                        }
                    }
                    else if let Some(func_captures) = export_function_re.captures(&format!("{}{}", next_line, Self::collect_until_brace(&lines, i))) {
                        let is_async = func_captures.get(1).is_some();
                        let full_name = func_captures[2].to_string();
                        let (controller, name) = Self::parse_controller_and_name(&full_name);
                        let params_str = func_captures[3].to_string();
                        let return_type = func_captures.get(4).map(|m| m.as_str().trim().to_string());

                        let params = Self::parse_params(&params_str);

                        metadata.endpoints.push(ApiEndpoint { controller: if controller.is_empty() { current_controller.clone() } else { controller }, name, method, path, params, return_type, is_async });
                    }
                }
            }

            i += 1;
        }

        Ok(())
    }

    /// 从函数名中解析控制器和方法名
    ///
    /// # Arguments
    /// * `full_name` - 完整的函数名
    ///
    /// # Returns
    /// * `(String, String)` - (控制器名, 方法名)
    fn parse_controller_and_name(full_name: &str) -> (String, String) {
        if let Some(idx) = full_name.find('.') { (full_name[..idx].to_string(), full_name[idx + 1..].to_string()) } else { ("Default".to_string(), full_name.to_string()) }
    }

    /// 收集直到开括号的内容
    ///
    /// # Arguments
    /// * `lines` - 行数组
    /// * `start` - 起始行索引
    ///
    /// # Returns
    /// * `String` - 收集的内容
    fn collect_until_brace(lines: &[&str], start: usize) -> String {
        let mut result = String::new();
        let mut i = start;

        while i < lines.len() && !lines[i].contains('{') {
            result.push_str(lines[i]);
            i += 1;
        }

        if i < lines.len() {
            if let Some(idx) = lines[i].find('{') {
                result.push_str(&lines[i][0..idx]);
            }
        }

        result
    }

    /// 解析参数字符串
    ///
    /// # Arguments
    /// * `params_str` - 参数字符串
    ///
    /// # Returns
    /// * `Vec<ApiParam>` - 解析后的参数列表
    fn parse_params(params_str: &str) -> Vec<ApiParam> {
        let mut params = Vec::new();

        let param_re = Regex::new(r#"(?:@(\w+)(?:\s*\(\s*(?:['"]([^'"]*)['"])?\s*\))?\s+)?(\w+)\s*(?::\s*([^,\s]+))?"#).unwrap();

        for caps in param_re.captures_iter(params_str) {
            let decorator_name = caps.get(1).map(|m| m.as_str());
            let extractor_param = caps.get(2).map(|m| m.as_str().to_string());
            let name = caps[3].to_string();
            let type_name = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_else(|| "any".to_string());

            let param_type = match decorator_name {
                Some("Body") => ApiParamType::Body,
                Some("Path") => ApiParamType::Path,
                Some("Query") => ApiParamType::Query,
                Some("Header") => ApiParamType::Header,
                Some("CurrentUser") => ApiParamType::CurrentUser,
                Some("Session") => ApiParamType::Session,
                Some("I18n") => ApiParamType::I18n,
                _ => ApiParamType::Regular,
            };

            params.push(ApiParam { name, param_type, type_name, extractor_param });
        }

        params
    }

    /// 从源文本中提取类型定义
    ///
    /// # Arguments
    /// * `source` - 源代码文本
    /// * `metadata` - 元数据对象
    ///
    /// # Returns
    /// * `Result<()>` - 操作结果
    fn extract_types_from_source(source: &str, metadata: &mut ApiMetadata) -> Result<()> {
        let class_re = Regex::new(r#"class\s+(\w+)\s*\{([\s\S]*?)\}"#)?;
        let interface_re = Regex::new(r#"interface\s+(\w+)\s*\{([\s\S]*?)\}"#)?;
        let _type_alias_re = Regex::new(r#"type\s+(\w+)\s*=\s*([^;]+);"#)?;

        for caps in class_re.captures_iter(source) {
            let name = caps[1].to_string();
            if !name.ends_with("Controller") {
                let body = caps[2].to_string();
                let fields = Self::parse_fields(&body);

                metadata.types.insert(name.clone(), TypeDefinition { name, fields });
            }
        }

        for caps in interface_re.captures_iter(source) {
            let name = caps[1].to_string();
            let body = caps[2].to_string();
            let fields = Self::parse_fields(&body);

            metadata.types.insert(name.clone(), TypeDefinition { name, fields });
        }

        Ok(())
    }

    /// 解析类型字段
    ///
    /// # Arguments
    /// * `body` - 类型定义体
    ///
    /// # Returns
    /// * `Vec<TypeField>` - 解析后的字段列表
    fn parse_fields(body: &str) -> Vec<TypeField> {
        let mut fields = Vec::new();
        let field_re = Regex::new(r#"(\w+)([!?])?\s*:\s*([^;!\n]+)(?:[;!])?"#).unwrap();

        for caps in field_re.captures_iter(body) {
            let name = caps[1].to_string();
            let optional = caps.get(2).map_or(false, |m| m.as_str() == "?");
            let type_name = caps[3].to_string().trim().to_string();

            fields.push(TypeField { name, type_name, optional });
        }

        fields
    }
}
