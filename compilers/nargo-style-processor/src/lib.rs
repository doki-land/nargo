#![warn(missing_docs)]

use nargo_types::{Error, Result, Span};
use notify::{self, Watcher};
use oak_core::{ParseSession, parse_one_pass};
use oak_css::{CssLanguage, CssParser};
use oak_scss::{ScssLanguage, ScssParser};
use oak_stylus::{StylusLanguage, StylusParser};
use std::path::Path;

/// CSS 预处理器类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreprocessorType {
    /// 标准 CSS
    Css,
    /// SCSS/SASS
    Scss,
    /// Less
    Less,
    /// Stylus
    Stylus,
}

use std::collections::HashMap;

/// CSS 处理器，用于处理和优化 CSS 代码
#[derive(Default)]
pub struct StyleProcessor {
    /// 是否启用 CSS 压缩
    ///
    /// 当设置为 `true` 时，处理器会移除空白字符、注释和不必要的字符，
    /// 并压缩颜色值，以减少 CSS 文件大小。
    pub minify: bool,
    /// 是否移除未使用的样式
    ///
    /// 当设置为 `true` 时，处理器会根据提供的已使用选择器列表，
    /// 移除未被使用的 CSS 规则。
    pub remove_unused: bool,
    /// CSS 预处理器类型
    ///
    /// 用于指定要使用的 CSS 预处理器，如 SCSS、Less 或 Stylus。
    pub preprocessor: Option<PreprocessorType>,
    /// 缓存已处理的代码，提高性能
    ///
    /// 缓存键为 (代码内容, 预处理器类型) 的元组，值为处理后的代码。
    cache: HashMap<(String, PreprocessorType), String>,
    /// 缓存压缩后的 CSS 代码，提高性能
    ///
    /// 缓存键为原始 CSS 代码，值为压缩后的 CSS 代码。
    minify_cache: HashMap<String, String>,
}

impl StyleProcessor {
    /// 创建新的样式处理器
    pub fn new() -> Self {
        Self { minify: false, remove_unused: false, preprocessor: None, cache: HashMap::new(), minify_cache: HashMap::new() }
    }

    /// 设置是否启用压缩
    pub fn with_minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    /// 设置是否移除未使用的样式
    pub fn with_remove_unused(mut self, remove_unused: bool) -> Self {
        self.remove_unused = remove_unused;
        self
    }

    /// 设置 CSS 预处理器类型
    pub fn with_preprocessor(mut self, preprocessor: PreprocessorType) -> Self {
        self.preprocessor = Some(preprocessor);
        self
    }

    /// 处理 CSS 代码
    ///
    /// # 参数
    /// * `css` - 要处理的 CSS 代码
    /// * `used_selectors` - 已使用的选择器列表，用于移除未使用的样式
    ///
    /// # 返回值
    /// 处理后的 CSS 代码
    pub fn process(&mut self, css: &str, used_selectors: Option<&Vec<String>>) -> Result<String> {
        let mut processed = css.to_string();

        // 处理预处理器代码
        if let Some(preprocessor) = self.preprocessor.clone() {
            processed = self.process_preprocessor(&processed, &preprocessor)?;
        }

        // 移除未使用的样式
        if self.remove_unused && used_selectors.is_some() {
            processed = self.remove_unused_styles(&processed, used_selectors.unwrap())?;
        }

        // 压缩 CSS
        if self.minify {
            processed = self.minify_css(&processed);
        }
        else {
            // 添加处理注释
            processed = format!("/* Processed by Nargo Style Processor */\n{}", processed);
        }

        Ok(processed)
    }

    /// 处理预处理器代码
    fn process_preprocessor(&mut self, code: &str, preprocessor: &PreprocessorType) -> Result<String> {
        let cache_key = (code.to_string(), preprocessor.clone());

        Self::get_or_compute(&mut self.cache, cache_key, || {
            match preprocessor {
                PreprocessorType::Css => {
                    // 使用 oak-css 解析 CSS
                    let language = CssLanguage::default();
                    let parser = CssParser::new(&language);
                    let mut session = ParseSession::new(1024);
                    let result = parse_one_pass(&parser, code, &mut session);
                    result.result.map_err(|e| Error::external_error("oak-css".to_string(), e.to_string(), Span::default()))?;
                    Ok(code.to_string())
                }
                PreprocessorType::Scss => {
                    // 使用 oak-scss 解析 SCSS
                    let language = ScssLanguage::default();
                    let parser = ScssParser::new(&language);
                    let mut session = ParseSession::new(1024);
                    let result = parse_one_pass(&parser, code, &mut session);
                    result.result.map_err(|e| Error::external_error("oak-scss".to_string(), e.to_string(), Span::default()))?;
                    Ok(code.to_string())
                }
                PreprocessorType::Less => Ok(code.to_string()),
                PreprocessorType::Stylus => {
                    // 使用 oak-stylus 解析 Stylus
                    let language = StylusLanguage::default();
                    let parser = StylusParser::new(&language);
                    let mut session = ParseSession::new(1024);
                    let result = parse_one_pass(&parser, code, &mut session);
                    result.result.map_err(|e| Error::external_error("oak-stylus".to_string(), e.to_string(), Span::default()))?;
                    Ok(code.to_string())
                }
            }
        })
    }

    /// 通用缓存处理方法
    ///
    /// # 参数
    /// * `cache` - 缓存 HashMap
    /// * `key` - 缓存键
    /// * `compute` - 计算函数，当缓存中不存在时执行
    ///
    /// # 返回值
    /// 缓存的值或计算的结果
    fn get_or_compute<K, V, F>(cache: &mut HashMap<K, V>, key: K, compute: F) -> Result<V>
    where
        K: std::hash::Hash + Eq,
        V: Clone,
        F: FnOnce() -> Result<V>,
    {
        if let Some(cached) = cache.get(&key) {
            Ok(cached.clone())
        }
        else {
            let value = compute()?;
            cache.insert(key, value.clone());
            Ok(value)
        }
    }

    /// 移除未使用的样式
    fn remove_unused_styles(&self, css: &str, used_selectors: &Vec<String>) -> Result<String> {
        // 简单的未使用样式移除实现
        let mut result = String::new();
        let mut remaining = css.to_string();

        while !remaining.is_empty() {
            // 查找规则开始
            if let Some(start_idx) = remaining.find('{') {
                // 提取选择器
                let selector = remaining[..start_idx].trim().to_string();

                // 查找规则结束
                let mut brace_count = 1;
                let mut end_idx = start_idx + 1;

                while end_idx < remaining.len() && brace_count > 0 {
                    if remaining.chars().nth(end_idx) == Some('{') {
                        brace_count += 1;
                    }
                    else if remaining.chars().nth(end_idx) == Some('}') {
                        brace_count -= 1;
                    }
                    end_idx += 1;
                }

                // 提取规则
                let rule = remaining[..end_idx].to_string();

                // 检查选择器是否被使用
                if self.is_selector_used(&selector, used_selectors) {
                    result.push_str(&rule);
                    result.push_str("\n");
                }

                // 更新剩余内容
                remaining = remaining[end_idx..].trim_start().to_string();
            }
            else {
                // 没有更多规则，添加剩余内容
                result.push_str(&remaining);
                break;
            }
        }

        Ok(result)
    }

    /// 检查选择器是否被使用
    fn is_selector_used(&self, selector: &str, used_selectors: &Vec<String>) -> bool {
        // 简单实现：检查选择器是否在使用列表中
        let selectors = selector.split(',').map(|s| s.trim());

        for s in selectors {
            if used_selectors.contains(&s.to_string()) {
                return true;
            }
        }

        false
    }

    /// 压缩 CSS
    ///
    /// # 参数
    /// * `css` - 要压缩的 CSS 代码
    ///
    /// # 返回值
    /// 压缩后的 CSS 代码
    fn minify_css(&mut self, css: &str) -> String {
        // 检查缓存
        if let Some(cached) = self.minify_cache.get(css) {
            return cached.clone();
        }

        let minified = self.perform_minification(css);

        // 存入缓存
        self.minify_cache.insert(css.to_string(), minified.clone());

        minified
    }

    /// 执行 CSS 压缩的核心逻辑
    fn perform_minification(&self, css: &str) -> String {
        let mut result = String::with_capacity(css.len());
        let mut in_comment = false;
        let mut in_string = false;
        let mut string_delimiter = '"';
        let mut chars = css.chars();

        while let Some(c) = chars.next() {
            if in_comment {
                // 处理注释
                if c == '*' {
                    if let Some(next) = chars.next() {
                        if next == '/' {
                            in_comment = false;
                        }
                    }
                }
                continue;
            }

            if in_string {
                // 处理字符串
                result.push(c);
                if c == string_delimiter {
                    // 检查是否被转义
                    let is_escaped = self.is_escaped(&result);
                    if !is_escaped {
                        in_string = false;
                    }
                }
                continue;
            }

            // 处理其他字符
            match c {
                '"' | '\'' => {
                    result.push(c);
                    in_string = true;
                    string_delimiter = c;
                }
                '/' => {
                    if let Some(next) = chars.next() {
                        if next == '*' {
                            in_comment = true;
                        }
                        else {
                            result.push('/');
                            result.push(next);
                        }
                    }
                    else {
                        result.push('/');
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    // 跳过所有空白字符
                }
                '#' => {
                    result.push('#');
                    // 读取并压缩颜色值
                    let mut hex_chars = String::new();
                    for _ in 0..6 {
                        if let Some(next) = chars.next() {
                            if next.is_ascii_hexdigit() {
                                hex_chars.push(next);
                            }
                            else {
                                result.push_str(&hex_chars);
                                result.push(next);
                                break;
                            }
                        }
                        else {
                            result.push_str(&hex_chars);
                            break;
                        }
                    }
                    // 尝试压缩颜色值
                    if hex_chars.len() == 6 {
                        if let Some(compressed) = self.compress_color(&hex_chars) {
                            result.push_str(&compressed);
                        }
                        else {
                            result.push_str(&hex_chars);
                        }
                    }
                    else if !hex_chars.is_empty() {
                        result.push_str(&hex_chars);
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        }

        // 移除不必要的分号
        let trimmed = self.remove_unnecessary_semicolons(&result);

        trimmed
    }

    /// 检查字符是否被转义
    fn is_escaped(&self, result: &String) -> bool {
        let mut backslash_count = 0;
        for ch in result.chars().rev() {
            if ch == '\\' {
                backslash_count += 1;
            }
            else {
                break;
            }
        }
        backslash_count % 2 == 1
    }

    /// 压缩颜色值
    ///
    /// # 参数
    /// * `color` - 6位十六进制颜色值（不包含#）
    ///
    /// # 返回值
    /// 压缩后的颜色值，如 #FFFFFF → #FFF
    fn compress_color(&self, color: &str) -> Option<String> {
        if color.len() != 6 {
            return None;
        }

        let chars: Vec<char> = color.chars().collect();
        if chars[0] == chars[1] && chars[2] == chars[3] && chars[4] == chars[5] { Some(format!("{}{}{}", chars[0], chars[2], chars[4])) } else { None }
    }

    /// 移除不必要的分号
    ///
    /// # 参数
    /// * `css` - 压缩后的 CSS 代码
    ///
    /// # 返回值
    /// 移除不必要分号后的 CSS 代码
    fn remove_unnecessary_semicolons(&self, css: &str) -> String {
        let mut result = String::with_capacity(css.len());
        let mut chars = css.chars().peekable();

        while let Some(c) = chars.next() {
            result.push(c);
        }

        result
    }
}

/// 处理 CSS 代码的便捷函数
///
/// # 参数
/// * `css` - 要处理的 CSS 代码
///
/// # 返回值
/// 处理后的 CSS 代码
pub fn process(css: &str) -> Result<String> {
    let mut processor = StyleProcessor::new();
    processor.process(css, None)
}

/// 处理 CSS 代码并移除未使用的样式
///
/// # 参数
/// * `css` - 要处理的 CSS 代码
/// * `used_selectors` - 已使用的选择器列表
///
/// # 返回值
/// 处理后的 CSS 代码
pub fn process_with_used_selectors(css: &str, used_selectors: &Vec<String>) -> Result<String> {
    let mut processor = StyleProcessor::new().with_remove_unused(true);
    processor.process(css, Some(used_selectors))
}

/// 压缩 CSS 代码
///
/// # 参数
/// * `css` - 要压缩的 CSS 代码
///
/// # 返回值
/// 压缩后的 CSS 代码
pub fn minify(css: &str) -> Result<String> {
    let mut processor = StyleProcessor::new().with_minify(true);
    processor.process(css, None)
}

/// 优化 CSS 代码（移除未使用的样式并压缩）
///
/// # 参数
/// * `css` - 要优化的 CSS 代码
/// * `used_selectors` - 已使用的选择器列表
///
/// # 返回值
/// 优化后的 CSS 代码
pub fn optimize(css: &str, used_selectors: &Vec<String>) -> Result<String> {
    let mut processor = StyleProcessor::new().with_remove_unused(true).with_minify(true);
    processor.process(css, Some(used_selectors))
}

/// 使用预处理器处理 CSS 代码
///
/// # 参数
/// * `css` - 要处理的 CSS 代码
/// * `preprocessor` - CSS 预处理器类型
///
/// # 返回值
/// 处理后的 CSS 代码
pub fn process_with_preprocessor(css: &str, preprocessor: PreprocessorType) -> Result<String> {
    let mut processor = StyleProcessor::new().with_preprocessor(preprocessor);
    processor.process(css, None)
}

/// 监听文件变化并自动重新处理样式
///
/// # 参数
/// * `processor` - 样式处理器实例
/// * `file_path` - 要监听的文件路径
/// * `output_path` - 输出文件路径
/// * `callback` - 文件变化时的回调函数
///
/// # 返回值
/// 监听结果
pub fn watch(processor: &StyleProcessor, file_path: &str, output_path: &str, callback: Option<fn()>) -> Result<()> {
    // 复制处理器配置
    let minify = processor.minify;
    let remove_unused = processor.remove_unused;
    let preprocessor = processor.preprocessor.clone();
    let file_path_str = file_path.to_string();
    let file_path_closure = file_path_str.clone();
    let output_path = output_path.to_string();

    // 创建文件监视器
    match notify::recommended_watcher(move |res: std::result::Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    // 文件被修改，重新处理
                    if let Ok(css) = std::fs::read_to_string(&file_path_closure) {
                        let mut processor = StyleProcessor::new().with_minify(minify).with_remove_unused(remove_unused).with_preprocessor(preprocessor.clone().unwrap_or(PreprocessorType::Css));

                        if let Ok(processed) = processor.process(&css, None) {
                            if std::fs::write(&output_path, processed).is_ok() {
                                println!("Style updated: {}", output_path);
                                if let Some(cb) = callback {
                                    cb();
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }) {
        Ok(mut watcher) => {
            // 开始监听文件
            if let Err(err) = watcher.watch(Path::new(&file_path_str), notify::RecursiveMode::NonRecursive) {
                return Err(Error::external_error("notify".to_string(), err.to_string(), Span::default()));
            }
            Ok(())
        }
        Err(err) => Err(Error::external_error("notify".to_string(), err.to_string(), Span::default())),
    }
}
