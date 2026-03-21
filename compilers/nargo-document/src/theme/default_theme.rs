//! 默认主题实现
//! 提供完整的文档站点主题和样式，兼容 VuTeX 配置格式

use super::Theme;
use crate::config::{Config, FooterConfig, NavItem as ConfigNavItem, SidebarItem as ConfigSidebarItem};
use nargo_types::NargoValue;
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path};

/// 将类型转换为 NargoValue 的 trait
pub trait ToNargoValue {
    /// 将自身转换为 NargoValue
    fn to_nargo_value(&self) -> NargoValue;
}

impl ToNargoValue for String {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::String(self.clone())
    }
}

impl ToNargoValue for bool {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Bool(*self)
    }
}

impl<T: ToNargoValue> ToNargoValue for Vec<T> {
    fn to_nargo_value(&self) -> NargoValue {
        NargoValue::Array(self.iter().map(|v| v.to_nargo_value()).collect())
    }
}

impl ToNargoValue for NargoValue {
    fn to_nargo_value(&self) -> NargoValue {
        self.clone()
    }
}

/// 语言信息
#[derive(Debug, Clone, Serialize)]
pub struct LocaleInfo {
    /// 语言代码
    pub code: String,
    /// 语言标签
    pub label: String,
    /// 是否为当前语言
    pub is_current: bool,
}

/// 侧边栏组
#[derive(Debug, Clone, Serialize)]
pub struct SidebarGroup {
    /// 组标题
    pub text: String,
    /// 组内项目
    pub items: Vec<SidebarLink>,
}

impl ToNargoValue for SidebarGroup {
    fn to_nargo_value(&self) -> NargoValue {
        let mut map = HashMap::new();
        map.insert("text".to_string(), NargoValue::String(self.text.clone()));
        map.insert("items".to_string(), self.items.to_nargo_value());
        NargoValue::Object(map)
    }
}

/// 侧边栏链接
#[derive(Debug, Clone, Serialize)]
pub struct SidebarLink {
    /// 链接文本
    pub text: String,
    /// 链接地址
    pub link: String,
}

impl ToNargoValue for SidebarLink {
    fn to_nargo_value(&self) -> NargoValue {
        let mut map = HashMap::new();
        map.insert("text".to_string(), NargoValue::String(self.text.clone()));
        map.insert("link".to_string(), NargoValue::String(self.link.clone()));
        NargoValue::Object(map)
    }
}

/// 导航栏项
#[derive(Debug, Clone, Serialize)]
pub struct NavItem {
    /// 显示文本
    pub text: String,
    /// 链接
    pub link: String,
}

impl ToNargoValue for NavItem {
    fn to_nargo_value(&self) -> NargoValue {
        let mut map = HashMap::new();
        map.insert("text".to_string(), NargoValue::String(self.text.clone()));
        map.insert("link".to_string(), NargoValue::String(self.link.clone()));
        NargoValue::Object(map)
    }
}

/// 页面模板上下文
#[derive(Debug, Clone, Serialize)]
pub struct PageContext {
    /// 页面标题
    pub page_title: String,
    /// 站点标题
    pub site_title: String,
    /// 页面内容
    pub content: String,
    /// 导航栏项目
    pub nav_items: Vec<NavItem>,
    /// 侧边栏组
    pub sidebar_groups: Vec<SidebarGroup>,
    /// 当前页面路径
    pub current_path: String,
    /// 是否有页脚
    pub has_footer: bool,
    /// 是否有页脚消息
    pub has_footer_message: bool,
    /// 页脚消息
    pub footer_message: String,
    /// 是否有页脚版权
    pub has_footer_copyright: bool,
    /// 页脚版权
    pub footer_copyright: String,
    /// 当前语言
    pub current_lang: String,
    /// 可用语言列表
    pub available_locales: Vec<LocaleInfo>,
    /// 相对于根目录的路径前缀
    pub root_path: String,
}

/// Jinja2 模板渲染器
#[derive(Clone)]
struct Jinja2Renderer {
    template_content: String,
}

impl Jinja2Renderer {
    /// 创建新的 Jinja2 渲染器
    fn new(template_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let template_content = fs::read_to_string(template_path)?;
        Ok(Self { template_content })
    }

    /// 渲染模板
    fn render(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = self.template_content.clone();

        // 第一步：处理 for 循环
        result = self.process_for_loops(&result, context)?;

        // 第二步：处理 if 条件
        result = self.process_if_conditions(&result, context)?;

        // 第三步：处理变量替换
        result = self.process_variables(&result, context)?;

        Ok(result)
    }

    /// 处理 for 循环
    fn process_for_loops(&self, content: &str, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = content.to_string();
        let loop_pattern = regex::Regex::new(r"\{%\s*for\s+(\w+)\s+in\s+(\w+)\s*%\}([\s\S]*?)\{%\s*endfor\s*%\}")?;

        loop {
            let mut replaced = false;
            result = loop_pattern
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    let collection_name = &caps[2];
                    let loop_body = &caps[3];

                    let mut rendered = String::new();

                    match collection_name {
                        "sidebar_groups" => {
                            for (i, item) in context.sidebar_groups.iter().enumerate() {
                                let mut item_body = loop_body.to_string();
                                let group = item.to_nargo_value();
                                item_body = self.replace_nested_variables(&item_body, var_name, &group, &context.current_path);

                                // 处理侧边栏组内部的 items 循环
                                if let Some(group) = context.sidebar_groups.get(i) {
                                    let inner_loop_pattern = regex::Regex::new(r"\{%\s*for\s+(\w+)\s+in\s+group\.items\s*%\}([\s\S]*?)\{%\s*endfor\s*%\}").unwrap();
                                    item_body = inner_loop_pattern
                                        .replace_all(&item_body, |inner_caps: &regex::Captures| {
                                            let inner_var_name = &inner_caps[1];
                                            let inner_loop_body = &inner_caps[2];

                                            let mut inner_rendered = String::new();
                                            for inner_item in &group.items {
                                                let mut inner_item_body = inner_loop_body.to_string();
                                                let item_value = inner_item.to_nargo_value();
                                                inner_item_body = self.replace_nested_variables(&inner_item_body, inner_var_name, &item_value, &context.current_path);
                                                inner_rendered.push_str(&inner_item_body);
                                            }
                                            inner_rendered
                                        })
                                        .to_string();
                                }

                                rendered.push_str(&item_body);
                            }
                        }
                        "nav_items" => {
                            for item in &context.nav_items {
                                let mut item_body = loop_body.to_string();
                                let item_value = item.to_nargo_value();
                                item_body = self.replace_nested_variables(&item_body, var_name, &item_value, &context.current_path);
                                rendered.push_str(&item_body);
                            }
                        }
                        _ => return caps[0].to_string(),
                    }

                    replaced = true;
                    rendered
                })
                .to_string();

            if !replaced {
                break;
            }
        }

        Ok(result)
    }

    /// 处理 if 条件
    fn process_if_conditions(&self, content: &str, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = content.to_string();

        // 处理简单的 if 条件
        let if_pattern = regex::Regex::new(r"\{%\s*if\s+(\w+)\s*%\}([\s\S]*?)\{%\s*endif\s*%\}")?;

        loop {
            let mut replaced = false;
            result = if_pattern
                .replace_all(&result, |caps: &regex::Captures| {
                    let condition_var = &caps[1];
                    let if_body = &caps[2];

                    let condition = match condition_var {
                        "has_footer" => context.has_footer,
                        "has_footer_message" => context.has_footer_message,
                        "has_footer_copyright" => context.has_footer_copyright,
                        _ => false,
                    };

                    replaced = true;
                    if condition { if_body.to_string() } else { String::new() }
                })
                .to_string();

            if !replaced {
                break;
            }
        }

        // 处理带有 not 的条件
        let not_if_pattern = regex::Regex::new(r"\{%\s*if\s+!\s*(\w+)\s*%\}([\s\S]*?)\{%\s*endif\s*%\}")?;

        loop {
            let mut replaced = false;
            result = not_if_pattern
                .replace_all(&result, |caps: &regex::Captures| {
                    let condition_var = &caps[1];
                    let if_body = &caps[2];

                    let condition = match condition_var {
                        "group.items.is_empty()" => false,
                        _ => true,
                    };

                    replaced = true;
                    if condition { if_body.to_string() } else { String::new() }
                })
                .to_string();

            if !replaced {
                break;
            }
        }

        Ok(result)
    }

    /// 处理变量替换
    fn process_variables(&self, content: &str, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = content.to_string();

        // 处理带条件的 class 或属性：{% if item.link == current_path %} class="active"{% endif %}
        let active_class_pattern = regex::Regex::new(r#"\{%\s*if\s+(\w+)\.link\s*==\s*current_path\s*%\}(\s*class=\"active\")\{%\s*endif\s*%\}"#)?;
        result = active_class_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let class_str = &caps[2];
                // 这里我们在嵌套变量替换时处理，但为了简单，我们直接保留 class
                // 在实际使用时，我们会在 process_for_loops 中处理
                class_str.to_string()
            })
            .to_string();

        // 简单变量替换
        let var_pattern = regex::Regex::new(r"\{\{\s*(\w+)\s*\}\}")?;

        result = var_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                match var_name {
                    "page_title" => context.page_title.clone(),
                    "site_title" => context.site_title.clone(),
                    "content" => context.content.clone(),
                    "current_path" => context.current_path.clone(),
                    "footer_message" => context.footer_message.clone(),
                    "footer_copyright" => context.footer_copyright.clone(),
                    "current_lang" => context.current_lang.clone(),
                    "root_path" => context.root_path.clone(),
                    _ => caps[0].to_string(),
                }
            })
            .to_string();

        // 处理 |safe 过滤器
        let safe_pattern = regex::Regex::new(r"\{\{\s*(\w+)\s*\|\s*safe\s*\}\}")?;
        result = safe_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                match var_name {
                    "content" => context.content.clone(),
                    _ => caps[0].to_string(),
                }
            })
            .to_string();

        Ok(result)
    }

    /// 替换嵌套变量
    fn replace_nested_variables(&self, content: &str, var_name: &str, value: &NargoValue, current_path: &str) -> String {
        let mut result = content.to_string();

        // 首先处理条件类名：{% if item.link == current_path %} class="active"{% endif %}
        let active_class_pattern = regex::Regex::new(&format!(r#"\{{%\s*if\s+{}\.link\s*==\s*current_path\s*%\}}(\s*class=\"active\")\{{%\s*endif\s*%\}}"#, regex::escape(var_name))).unwrap();
        if let Some(link_val) = value.get("link").and_then(|v| v.as_str()) {
            let is_current = link_val == current_path;
            result = active_class_pattern.replace_all(&result, |_caps: &regex::Captures| if is_current { r#" class="active""#.to_string() } else { String::new() }).to_string();
        }

        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                let placeholder = format!("{{{{ {}.{} }}}}", var_name, key);
                let val_str = match val {
                    NargoValue::String(s) => s.clone(),
                    NargoValue::Number(n) => n.to_string(),
                    NargoValue::Bool(b) => b.to_string(),
                    _ => val.to_string(),
                };
                result = result.replace(&placeholder, &val_str);
            }
        }

        result
    }
}

/// 默认主题
#[derive(Clone)]
pub struct DefaultTheme {
    /// 主题配置
    pub config: Config,
    /// 模板渲染器
    renderer: Option<Jinja2Renderer>,
}

impl Theme for DefaultTheme {
    /// 获取主题名称
    fn name(&self) -> &str {
        "default"
    }

    /// 渲染页面
    fn render_page(&self, context: &PageContext) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(renderer) = &self.renderer { renderer.render(context) } else { Err("Template renderer not initialized".into()) }
    }

    /// 获取主题配置
    fn config(&self) -> &Config {
        &self.config
    }
}

impl DefaultTheme {
    /// 创建新的默认主题实例
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates").join("page.html");

        let renderer = if template_path.exists() { Some(Jinja2Renderer::new(&template_path)?) } else { None };

        Ok(Self { config, renderer })
    }

    /// 获取站点标题
    pub fn site_title(&self) -> &str {
        self.config.title.as_deref().unwrap_or("")
    }

    /// 将 ConfigNavItem 转换为主题 NavItem
    pub fn convert_nav_items(items: &[ConfigNavItem]) -> Vec<NavItem> {
        items.iter().filter_map(|item| item.link.as_ref().map(|link| NavItem { text: item.text.clone(), link: link.clone() })).collect()
    }

    /// 将侧边栏配置转换为 SidebarGroup
    pub fn convert_sidebar_groups(sidebar: &std::collections::HashMap<String, Vec<ConfigSidebarItem>>) -> Vec<SidebarGroup> {
        sidebar
            .iter()
            .map(|(group_text, items)| {
                let sidebar_links: Vec<SidebarLink> = items.iter().filter_map(|item| item.link.as_ref().map(|link| SidebarLink { text: item.text.clone(), link: link.clone() })).collect();

                SidebarGroup { text: group_text.clone(), items: sidebar_links }
            })
            .collect()
    }

    /// 获取页脚配置
    pub fn footer_config(&self) -> &Option<FooterConfig> {
        &self.config.theme.footer
    }
}
