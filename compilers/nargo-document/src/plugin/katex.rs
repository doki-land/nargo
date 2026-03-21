//! KaTeX 数学公式渲染插件
//!
//! 提供对 Markdown 中 LaTeX 数学公式的支持，包括行内公式 `$...$` 和块级公式 `$$...$$`

use crate::plugin::{DocumentPlugin, PluginContext, PluginMeta};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// 匹配块级公式 `$$...$$` 或 `\[...\]` 的正则表达式
    static ref BLOCK_MATH_RE: Regex = Regex::new(r"(?:\$\$|\\\[)([\s\S]*?)(?:\$\$|\\\])").unwrap();
    /// 匹配行内公式 `$...$` 或 `\(...\)` 的正则表达式
    static ref INLINE_MATH_RE: Regex = Regex::new(r"(?:\$|\\\()([^$\n]+?)(?:\$|\\\))").unwrap();
}

/// KaTeX 数学公式渲染插件
pub struct KaTeXPlugin {
    /// 插件元数据
    meta: PluginMeta,
}

impl KaTeXPlugin {
    /// 创建新的 KaTeX 插件实例
    pub fn new() -> Self {
        Self { meta: PluginMeta::new("nargo-document-plugin-katex".to_string(), "0.1.0".to_string(), "KaTeX 数学公式渲染插件，支持行内公式和块级公式".to_string()) }
    }

    /// 处理数学公式，将 `$$...$$`、`\[...\]`、`$...$` 和 `\(...\)` 替换为对应的 HTML
    ///
    /// # Arguments
    ///
    /// * `content` - 包含数学公式的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    pub fn process_katex(&self, content: &str) -> String {
        let mut result = content.to_string();
        result = self.process_block_math(&result);
        result = INLINE_MATH_RE
            .replace_all(&result, |caps: &regex::Captures| {
                let math = &caps[1];
                format!("<span class=\"katex\">{}</span>", math.trim())
            })
            .to_string();
        result
    }

    /// 处理块级公式，将 `$$...$$` 替换为 `<div class="katex-display">...</div>`
    ///
    /// # Arguments
    ///
    /// * `content` - 包含数学公式的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    fn process_block_math(&self, content: &str) -> String {
        BLOCK_MATH_RE
            .replace_all(content, |caps: &regex::Captures| {
                let math = &caps[1];
                format!("<div class=\"katex-display\">{}</div>", math.trim())
            })
            .to_string()
    }

    /// 处理行内公式，将 `$...$` 替换为 `<span class="katex">...</span>`
    ///
    /// # Arguments
    ///
    /// * `content` - 包含数学公式的文本内容
    ///
    /// # Returns
    ///
    /// 替换后的文本内容
    fn process_inline_math(&self, content: &str) -> String {
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        let mut in_math = false;
        let mut current_math = String::new();

        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '$' {
                        chars.next();
                        if in_math {
                            result.push_str("$$");
                            result.push_str(&current_math);
                            result.push_str("$$");
                            in_math = false;
                            current_math.clear();
                        }
                        else {
                            result.push_str("$$");
                        }
                        continue;
                    }
                }

                if in_math {
                    result.push_str(&format!("<span class=\"katex\">{}</span>", current_math));
                    in_math = false;
                    current_math.clear();
                }
                else {
                    in_math = true;
                }
            }
            else {
                if in_math {
                    current_math.push(c);
                }
                else {
                    result.push(c);
                }
            }
        }

        if in_math {
            result.push('$');
            result.push_str(&current_math);
        }

        result
    }
}

impl Default for KaTeXPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentPlugin for KaTeXPlugin {
    /// 获取插件元数据
    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    /// 渲染前钩子，在 Markdown 解析后、HTML 渲染前处理数学公式
    ///
    /// # Arguments
    ///
    /// * `context` - 插件上下文，包含文档内容等信息
    ///
    /// # Returns
    ///
    /// 处理后的插件上下文
    fn before_render(&self, context: PluginContext) -> PluginContext {
        let content = self.process_katex(&context.content);

        PluginContext { content, frontmatter: context.frontmatter, path: context.path }
    }
}
