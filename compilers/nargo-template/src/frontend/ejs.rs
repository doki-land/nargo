#![warn(missing_docs)]

//! EJS 前端模块
//!
//! 将 EJS 模板源码编译为扁平 IR 指令序列，
//! 使用 `oak_ejs` 解析器解析模板，遍历 RedTree 生成指令。

use crate::frontend::Frontend;
use crate::ir::*;
use crate::error::{TemplateError, TemplateResult};
use oak_core::{
    SourceText,
    parser::{Parser, ParseSession},
    source::Source,
    tree::RedTree,
};
use oak_ejs::{EjsElementType, language::EjsLanguage, parser::EjsParser};

/// EJS 模板前端
///
/// 使用 `oak_ejs` 解析器将 EJS 模板解析为语法树，
/// 然后遍历 RedTree 编译为扁平 IR 指令序列。
pub struct EjsFrontend {
    /// EJS 语言配置
    language: EjsLanguage,
}

impl EjsFrontend {
    /// 创建新的 EJS 前端
    ///
    /// # 返回值
    /// 使用默认语言配置的 EJS 前端实例
    pub fn new() -> Self {
        Self {
            language: EjsLanguage::default(),
        }
    }

    /// 使用自定义语言配置创建 EJS 前端
    ///
    /// # 参数
    /// - `language`: EJS 语言配置
    ///
    /// # 返回值
    /// EJS 前端实例
    pub fn with_language(language: EjsLanguage) -> Self {
        Self { language }
    }
}

impl Default for EjsFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for EjsFrontend {
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR> {
        let source_text = SourceText::new(source);
        let parser = EjsParser::new(&self.language);
        let mut cache = ParseSession::<EjsLanguage>::default();
        let parse_output = parser.parse(&source_text, &[], &mut cache);

        let green_node = parse_output
            .result
            .map_err(|e| TemplateError::Syntax(format!("EJS parser error: {}", e)))?;

        let red_tree = RedTree::<EjsLanguage>::new(green_node);
        let mut compiler = EjsCompiler::new(&source_text);
        compiler.compile_tree(&red_tree)?;
        Ok(compiler.builder.build())
    }

    fn name(&self) -> &str {
        "ejs"
    }

    fn default_extension(&self) -> &str {
        "ejs"
    }
}

/// EJS 到 IR 的编译器
///
/// 遍历 EJS RedTree 并使用 IRBuilder 发射扁平指令。
struct EjsCompiler<'a> {
    /// 模板源文本引用
    source: &'a SourceText,
    /// IR 指令构建器
    builder: IRBuilder,
}

impl<'a> EjsCompiler<'a> {
    /// 创建新的 EJS 编译器
    ///
    /// # 参数
    /// - `source`: 模板源文本引用
    ///
    /// # 返回值
    /// 编译器实例
    fn new(source: &'a SourceText) -> Self {
        Self {
            source,
            builder: IRBuilder::new(),
        }
    }

    /// 编译 RedTree 为 IR 指令
    ///
    /// 遍历根节点的子节点，逐一编译为扁平 IR 指令。
    ///
    /// # 参数
    /// - `tree`: RedTree 根节点引用
    ///
    /// # 返回值
    /// 编译成功或错误
    fn compile_tree(&mut self, tree: &RedTree<'a, EjsLanguage>) -> TemplateResult<()> {
        let children: Vec<_> = tree.children().collect();
        let mut i = 0;
        while i < children.len() {
            self.compile_child(&children, &mut i)?;
            i += 1;
        }
        Ok(())
    }

    /// 编译单个子节点
    ///
    /// 根据节点类型分发到对应的编译逻辑。
    ///
    /// # 参数
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 编译成功或错误
    fn compile_child(
        &mut self,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<()> {
        let child = children[*index];
        let kind: EjsElementType = child.kind();

        match kind {
            EjsElementType::Text => {
                let text = self.source.get_text_in(child.span()).into_owned();
                if !text.is_empty() {
                    self.builder.output_text(text);
                }
            }
            EjsElementType::OutputEscape => {
                let expr_text = self.extract_expression_text(&child);
                self.compile_expr(&expr_text);
                self.builder.call_filter("escape".to_string(), 0);
                self.builder.output();
            }
            EjsElementType::OutputRaw => {
                let expr_text = self.extract_expression_text(&child);
                self.compile_expr(&expr_text);
                self.builder.output();
            }
            EjsElementType::Code => {
                let code_text = self.extract_code_text(&child);
                self.compile_code_block(&code_text, children, index)?;
            }
            EjsElementType::EscapedTag => {
                self.builder.output_text("<%".to_string());
            }
            EjsElementType::Comment | EjsElementType::Root => {}
            _ => {}
        }

        Ok(())
    }

    /// 从 EJS 输出标签中提取表达式文本
    ///
    /// 移除 `<%=` / `<%-` / `<%` 和 `%>` 标签边界，
    /// 返回中间的表达式文本。
    ///
    /// # 参数
    /// - `tree`: RedTree 节点引用
    ///
    /// # 返回值
    /// 提取的表达式文本
    fn extract_expression_text(&self, tree: &RedTree<'a, EjsLanguage>) -> String {
        let full = self.source.get_text_in(tree.span()).into_owned();
        strip_ejs_tags(&full)
    }

    /// 从 EJS 代码标签中提取代码文本
    ///
    /// 移除 `<%` 和 `%>` 标签边界，返回中间的代码文本。
    ///
    /// # 参数
    /// - `tree`: RedTree 节点引用
    ///
    /// # 返回值
    /// 提取的代码文本
    fn extract_code_text(&self, tree: &RedTree<'a, EjsLanguage>) -> String {
        let full = self.source.get_text_in(tree.span()).into_owned();
        strip_ejs_tags(&full)
    }

    /// 编译代码块
    ///
    /// 解析 EJS 代码块中的简单 JavaScript 模式：
    /// - `if (condition) {` → 编译条件，发射跳转指令，收集体
    /// - `for (let x of items) {` → 编译可迭代表达式，发射循环指令
    /// - `while (condition) {` → 编译条件，发射循环跳转
    /// - `let/var/const x = value;` → 编译值，存储变量
    ///
    /// # 参数
    /// - `code`: 代码文本
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 编译成功或错误
    fn compile_code_block(
        &mut self,
        code: &str,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<()> {
        let code = code.trim();

        if code == "}" || code == "};}" {
            return Ok(());
        }

        if self.try_compile_if(code, children, index)? {
            return Ok(());
        }

        if self.try_compile_for(code, children, index)? {
            return Ok(());
        }

        if self.try_compile_while(code, children, index)? {
            return Ok(());
        }

        if try_compile_let(code, &mut self.builder) {
            return Ok(());
        }

        Ok(())
    }

    /// 尝试编译 if 语句
    ///
    /// 匹配 `if (condition) {` 模式，编译条件表达式，
    /// 发射 JumpIfFalse 指令，收集体直到遇到对应的 `}` 闭合标签，
    /// 并继续检查 elif 和 else 分支。
    ///
    /// # 参数
    /// - `code`: 代码文本
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 若匹配成功返回 Ok(true)，否则返回 Ok(false)
    fn try_compile_if(
        &mut self,
        code: &str,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<bool> {
        let trimmed = code.trim();
        if !trimmed.starts_with("if") {
            return Ok(false);
        }

        let rest = trimmed[2..].trim();
        if !rest.starts_with('(') {
            return Ok(false);
        }

        let cond_end = match find_matching_paren(rest, 0) {
            Some(pos) => pos,
            None => return Ok(false),
        };
        let condition_text = rest[1..cond_end].trim();

        self.compile_expr(condition_text);
        let jump_if_false_ip = self.builder.emit_jump_if_false();

        self.compile_block_body(children, index)?;

        let mut end_jumps = vec![self.builder.emit_jump()];

        self.builder.patch_jump(jump_if_false_ip);

        while *index + 1 < children.len() {
            let next_idx = *index + 1;
            let next_kind: EjsElementType = children[next_idx].kind();
            if next_kind == EjsElementType::Code {
                let next_code = self.extract_code_text(&children[next_idx]);
                let next_trimmed = next_code.trim();

                if next_trimmed.starts_with("else if") || next_trimmed.starts_with("elif") {
                    *index = next_idx;
                    let elif_rest = if next_trimmed.starts_with("else if") {
                        next_trimmed[7..].trim()
                    } else {
                        next_trimmed[4..].trim()
                    };
                    if let Some(elif_cond_text) = extract_paren_content(elif_rest) {
                        self.compile_expr(elif_cond_text);
                        let elif_jump_ip = self.builder.emit_jump_if_false();
                        self.compile_block_body(children, index)?;
                        end_jumps.push(self.builder.emit_jump());
                        self.builder.patch_jump(elif_jump_ip);
                    }
                    continue;
                } else if next_trimmed.starts_with("else") {
                    *index = next_idx;
                    let else_rest = next_trimmed[4..].trim();
                    if else_rest.starts_with('{') || else_rest.is_empty() {
                        self.compile_block_body(children, index)?;
                    }
                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        for jump_ip in end_jumps {
            self.builder.patch_jump(jump_ip);
        }

        Ok(true)
    }

    /// 尝试编译 for 循环
    ///
    /// 匹配 `for (let x of items) {` 或 `for (var x of items) {` 模式，
    /// 编译可迭代表达式，发射 ForInit 和 ForNext 指令。
    ///
    /// # 参数
    /// - `code`: 代码文本
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 若匹配成功返回 Ok(true)，否则返回 Ok(false)
    fn try_compile_for(
        &mut self,
        code: &str,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<bool> {
        let trimmed = code.trim();
        if !trimmed.starts_with("for") {
            return Ok(false);
        }

        let rest = trimmed[3..].trim();
        if !rest.starts_with('(') {
            return Ok(false);
        }

        let cond_end = match find_matching_paren(rest, 0) {
            Some(pos) => pos,
            None => return Ok(false),
        };
        let header = rest[1..cond_end].trim();

        let (pattern, iterable_text) = match parse_for_header(header) {
            Some(result) => result,
            None => return Ok(false),
        };

        self.compile_expr(iterable_text);
        let for_init_ip = self.builder.emit_for_init(pattern);

        let loop_start = self.builder.current_ip();
        self.compile_block_body(children, index)?;

        self.builder.emit_for_next(loop_start);
        self.builder.patch_jump(for_init_ip);

        Ok(true)
    }

    /// 尝试编译 while 循环
    ///
    /// 匹配 `while (condition) {` 模式，
    /// 编译条件表达式，发射跳转指令实现循环。
    ///
    /// # 参数
    /// - `code`: 代码文本
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 若匹配成功返回 Ok(true)，否则返回 Ok(false)
    fn try_compile_while(
        &mut self,
        code: &str,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<bool> {
        let trimmed = code.trim();
        if !trimmed.starts_with("while") {
            return Ok(false);
        }

        let rest = trimmed[5..].trim();
        if !rest.starts_with('(') {
            return Ok(false);
        }

        let cond_end = match find_matching_paren(rest, 0) {
            Some(pos) => pos,
            None => return Ok(false),
        };
        let condition_text = rest[1..cond_end].trim();

        let loop_start = self.builder.current_ip();
        self.compile_expr(condition_text);
        let jump_if_false_ip = self.builder.emit_jump_if_false();

        self.compile_block_body(children, index)?;

        let back_jump_ip = self.builder.emit_jump();
        self.builder.patch_jump_to(back_jump_ip, loop_start);
        self.builder.patch_jump(jump_if_false_ip);

        Ok(true)
    }

    /// 编译代码块体内容
    ///
    /// 从当前索引之后开始，收集子节点直到遇到对应的 `}` 闭合代码块。
    /// 支持嵌套的 `{}` 块。直接将指令发射到 IRBuilder。
    ///
    /// # 参数
    /// - `children`: 所有子节点切片
    /// - `index`: 当前子节点索引
    ///
    /// # 返回值
    /// 编译成功或错误
    fn compile_block_body(
        &mut self,
        children: &[RedTree<'a, EjsLanguage>],
        index: &mut usize,
    ) -> TemplateResult<()> {
        let mut depth = 1;

        while *index + 1 < children.len() {
            let next_idx = *index + 1;
            let next_kind: EjsElementType = children[next_idx].kind();

            match next_kind {
                EjsElementType::Code => {
                    let next_code = self.extract_code_text(&children[next_idx]);
                    let trimmed = next_code.trim();

                    let open_braces = trimmed.chars().filter(|c| *c == '{').count();
                    let close_braces = trimmed.chars().filter(|c| *c == '}').count();
                    depth = depth + open_braces - close_braces;

                    if depth <= 0 {
                        *index = next_idx;
                        break;
                    }

                    if trimmed == "}" {
                        *index = next_idx;
                        break;
                    }

                    if trimmed.starts_with("else if")
                        || trimmed.starts_with("elif")
                        || trimmed.starts_with("else")
                    {
                        *index = next_idx;
                        break;
                    }

                    *index = next_idx;
                    self.compile_code_block(trimmed, children, index)?;
                }
                EjsElementType::Text => {
                    *index = next_idx;
                    let text = self
                        .source
                        .get_text_in(children[next_idx].span())
                        .into_owned();
                    if !text.is_empty() {
                        self.builder.output_text(text);
                    }
                }
                EjsElementType::OutputEscape => {
                    *index = next_idx;
                    let expr_text = self.extract_expression_text(&children[next_idx]);
                    self.compile_expr(&expr_text);
                    self.builder.call_filter("escape".to_string(), 0);
                    self.builder.output();
                }
                EjsElementType::OutputRaw => {
                    *index = next_idx;
                    let expr_text = self.extract_expression_text(&children[next_idx]);
                    self.compile_expr(&expr_text);
                    self.builder.output();
                }
                EjsElementType::EscapedTag => {
                    *index = next_idx;
                    self.builder.output_text("<%".to_string());
                }
                EjsElementType::Comment => {
                    *index = next_idx;
                }
                _ => {
                    *index = next_idx;
                }
            }
        }

        Ok(())
    }

    /// 编译表达式为栈操作指令
    ///
    /// 将 JavaScript 表达式文本解析并编译为扁平的栈操作指令序列。
    /// 支持三元表达式、二元运算、一元运算、字面量、
    /// 标识符、字段访问、索引访问和函数调用。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    fn compile_expr(&mut self, input: &str) {
        let input = input.trim();
        if input.is_empty() {
            self.builder.push_null();
            return;
        }

        if self.try_compile_ternary(input) {
            return;
        }

        if self.try_compile_binary(input) {
            return;
        }

        if self.try_compile_unary(input) {
            return;
        }

        if self.try_compile_literal(input) {
            return;
        }

        if self.try_compile_call_or_access(input) {
            return;
        }

        self.builder.load_var(input.to_string());
    }

    /// 尝试编译三元表达式
    ///
    /// 匹配 `condition ? then : else` 模式，
    /// 使用跳转指令实现条件分支。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    ///
    /// # 返回值
    /// 若匹配成功返回 true，否则返回 false
    fn try_compile_ternary(&mut self, input: &str) -> bool {
        let question_pos = match find_char_not_in_string(input, '?') {
            Some(pos) => pos,
            None => return false,
        };
        if question_pos == 0 {
            return false;
        }

        let colon_pos = match find_char_not_in_string(&input[question_pos + 1..], ':') {
            Some(pos) => pos,
            None => return false,
        };
        let colon_abs = question_pos + 1 + colon_pos;

        self.compile_expr(&input[..question_pos]);
        let jump_if_false_ip = self.builder.emit_jump_if_false();

        self.compile_expr(&input[question_pos + 1..colon_abs]);
        let jump_end_ip = self.builder.emit_jump();

        self.builder.patch_jump(jump_if_false_ip);
        self.compile_expr(&input[colon_abs + 1..]);
        self.builder.patch_jump(jump_end_ip);

        true
    }

    /// 尝试编译二元运算表达式
    ///
    /// 支持 `||`, `&&`, `==`, `!=`, `===`, `!==`, `>=`, `<=`, `>`, `<`,
    /// `+`, `-`, `*`, `/`, `%` 运算符。
    /// 按运算符优先级从低到高尝试匹配。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    ///
    /// # 返回值
    /// 若匹配成功返回 true，否则返回 false
    fn try_compile_binary(&mut self, input: &str) -> bool {
        let operators_by_precedence: &[(&str, BinaryOp)] = &[
            ("||", BinaryOp::Or),
            ("&&", BinaryOp::And),
            ("===", BinaryOp::Eq),
            ("!==", BinaryOp::Ne),
            ("==", BinaryOp::Eq),
            ("!=", BinaryOp::Ne),
            (">=", BinaryOp::Ge),
            ("<=", BinaryOp::Le),
            (">", BinaryOp::Gt),
            ("<", BinaryOp::Lt),
            ("+", BinaryOp::Add),
            ("-", BinaryOp::Sub),
            ("*", BinaryOp::Mul),
            ("/", BinaryOp::Div),
            ("%", BinaryOp::Mod),
        ];

        for (op_str, op) in operators_by_precedence {
            if let Some(pos) = find_operator_not_in_string(input, op_str) {
                if pos == 0 || pos + op_str.len() >= input.len() {
                    continue;
                }
                self.compile_expr(&input[..pos]);
                self.compile_expr(&input[pos + op_str.len()..]);
                self.builder.binary_op(*op);
                return true;
            }
        }

        false
    }

    /// 尝试编译一元运算表达式
    ///
    /// 支持 `!` (逻辑非) 和 `-` (负号) 运算符。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    ///
    /// # 返回值
    /// 若匹配成功返回 true，否则返回 false
    fn try_compile_unary(&mut self, input: &str) -> bool {
        let trimmed = input.trim();

        if trimmed.starts_with('!') {
            let inner = &trimmed[1..];
            if !inner.is_empty() {
                self.compile_expr(inner);
                self.builder.unary_op(UnaryOp::Not);
                return true;
            }
        }

        if trimmed.starts_with('-') {
            let inner = &trimmed[1..];
            if !inner.is_empty() {
                if let Some(n) = inner.trim().parse::<f64>().ok() {
                    self.builder.push_number(-n);
                    return true;
                }
                self.compile_expr(inner);
                self.builder.unary_op(UnaryOp::Neg);
                return true;
            }
        }

        false
    }

    /// 尝试编译字面量
    ///
    /// 支持 `true`, `false`, `null`, `undefined`, 数字字面量和字符串字面量。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    ///
    /// # 返回值
    /// 若匹配成功返回 true，否则返回 false
    fn try_compile_literal(&mut self, input: &str) -> bool {
        let trimmed = input.trim();

        if trimmed == "true" || trimmed == "True" {
            self.builder.push_bool(true);
            return true;
        }
        if trimmed == "false" || trimmed == "False" {
            self.builder.push_bool(false);
            return true;
        }
        if trimmed == "null" || trimmed == "undefined" {
            self.builder.push_null();
            return true;
        }

        if let Ok(n) = trimmed.parse::<f64>() {
            self.builder.push_number(n);
            return true;
        }

        if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
            || (trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() >= 2)
        {
            self.builder
                .push_string(trimmed[1..trimmed.len() - 1].to_string());
            return true;
        }

        false
    }

    /// 尝试编译函数调用或属性访问表达式
    ///
    /// 解析标识符链（如 `obj.prop`）和函数调用（如 `fn(arg1, arg2)`），
    /// 以及索引访问（如 `arr[0]`）。
    /// 函数调用编译为 `call_filter` 指令。
    ///
    /// # 参数
    /// - `input`: 表达式文本
    ///
    /// # 返回值
    /// 若匹配成功返回 true，否则返回 false
    fn try_compile_call_or_access(&mut self, input: &str) -> bool {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return false;
        }

        if !trimmed
            .chars()
            .next()
            .map_or(false, |c| c.is_alphabetic() || c == '_' || c == '$')
        {
            return false;
        }

        let mut has_complex = false;
        let mut name_buf = String::new();
        let mut i = 0;
        let chars: Vec<char> = trimmed.chars().collect();
        let mut pending_calls: Vec<(String, usize)> = Vec::new();

        while i < chars.len() {
            let ch = chars[i];

            if ch == '.' {
                if name_buf.is_empty() {
                    return false;
                }
                has_complex = true;
                name_buf.push('.');
                i += 1;
                continue;
            }

            if ch == '[' {
                has_complex = true;
                if !name_buf.is_empty() {
                    self.compile_access_chain(&name_buf);
                    name_buf.clear();
                }
                let bracket_content = match extract_bracket_content(&chars, i) {
                    Some(c) => c,
                    None => return false,
                };
                self.compile_expr(&bracket_content);
                self.builder.index_access();
                i = skip_bracket(&chars, i);
                continue;
            }

            if ch == '(' {
                if name_buf.is_empty() {
                    return false;
                }
                has_complex = true;
                let callee = name_buf.clone();
                name_buf.clear();

                let paren_content = match extract_paren_content_from_chars(&chars, i) {
                    Some(c) => c,
                    None => return false,
                };
                let args = split_call_args(&paren_content);
                let arg_count = args.len();

                for arg in &args {
                    self.compile_expr(arg);
                }

                pending_calls.push((callee, arg_count));

                i = skip_paren(&chars, i);
                continue;
            }

            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                name_buf.push(ch);
                i += 1;
            } else {
                break;
            }
        }

        if !has_complex {
            return false;
        }

        if !name_buf.is_empty() {
            self.compile_access_chain(&name_buf);
        }

        for (callee, arg_count) in pending_calls {
            self.builder.call_filter(callee, arg_count as u8);
        }

        true
    }

    /// 编译属性访问链
    ///
    /// 将 `obj.prop.field` 形式的访问链编译为
    /// load_var + field_access 指令序列。
    ///
    /// # 参数
    /// - `chain`: 以 `.` 分隔的访问链字符串
    fn compile_access_chain(&mut self, chain: &str) {
        let parts: Vec<&str> = chain.split('.').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return;
        }

        self.builder.load_var(parts[0].to_string());

        for part in &parts[1..] {
            self.builder.field_access(part.to_string());
        }
    }
}

/// 移除 EJS 标签边界
///
/// 从完整标签内容中移除 `<%=`, `<%-`, `<%` 开头和 `%>` 结尾。
///
/// # 参数
/// - `content`: 完整的 EJS 标签内容
///
/// # 返回值
/// 去除标签后的内容
fn strip_ejs_tags(content: &str) -> String {
    let content = content.trim();

    if content.starts_with("<%=") || content.starts_with("<%-") {
        let inner = &content[3..];
        strip_close_tag(inner)
    } else if content.starts_with("<%") {
        let inner = &content[2..];
        strip_close_tag(inner)
    } else {
        content.to_string()
    }
}

/// 移除 EJS 闭合标签 `%>`
///
/// # 参数
/// - `content`: 可能以 `%>` 结尾的内容
///
/// # 返回值
/// 去除闭合标签后的内容
fn strip_close_tag(content: &str) -> String {
    if content.trim_end().ends_with("%>") {
        let trimmed = content.trim_end();
        trimmed[..trimmed.len() - 2].trim().to_string()
    } else {
        content.trim().to_string()
    }
}

/// 查找匹配的右括号位置
///
/// 从指定起始位置查找与左括号匹配的右括号，支持嵌套。
///
/// # 参数
/// - `s`: 源字符串
/// - `start`: 左括号的位置
///
/// # 返回值
/// 匹配的右括号位置，若未找到则返回 None
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    if start >= chars.len() || chars[start] != '(' {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';

    for i in start..chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == string_char && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
            }
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }

        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                let byte_pos: usize = chars[..i + 1].iter().collect::<String>().len();
                return Some(byte_pos - 1);
            }
        }
    }

    None
}

/// 从以括号开头的字符串中提取括号内的内容
///
/// # 参数
/// - `s`: 以 `(` 开头的字符串
///
/// # 返回值
/// 括号内的文本，若格式不匹配则返回 None
fn extract_paren_content(s: &str) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with('(') {
        return None;
    }
    let end = find_matching_paren(s, 0)?;
    Some(s[1..end].trim())
}

/// 解析 for 循环头部
///
/// 支持 `let/var/const x of iterable` 和 `let/var/const x in iterable` 模式。
///
/// # 参数
/// - `header`: for 循环括号内的文本
///
/// # 返回值
/// 元组（迭代模式，可迭代表达式文本），若无法解析则返回 None
fn parse_for_header(header: &str) -> Option<(ForPattern, &str)> {
    let header = header.trim();

    let after_keyword = if header.starts_with("let ") {
        &header[4..]
    } else if header.starts_with("var ") {
        &header[4..]
    } else if header.starts_with("const ") {
        &header[6..]
    } else {
        header
    };

    let after_keyword = after_keyword.trim();

    let (var_part, iterable_part) = if let Some(pos) = after_keyword.find(" of ") {
        (&after_keyword[..pos], &after_keyword[pos + 4..])
    } else if let Some(pos) = after_keyword.find(" in ") {
        (&after_keyword[..pos], &after_keyword[pos + 4..])
    } else {
        return None;
    };

    let var_names: Vec<&str> = var_part
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let pattern = if var_names.len() == 1 {
        ForPattern::Identifier(var_names[0].to_string())
    } else {
        ForPattern::Tuple(var_names.iter().map(|s| s.to_string()).collect())
    };

    Some((pattern, iterable_part.trim()))
}

/// 尝试编译变量声明语句
///
/// 匹配 `let x = value;`、`var x = value;`、`const x = value;` 模式。
///
/// # 参数
/// - `code`: 代码文本
/// - `builder`: IR 构建器
///
/// # 返回值
/// 若匹配成功返回 true，否则返回 false
fn try_compile_let(code: &str, builder: &mut IRBuilder) -> bool {
    let trimmed = code.trim();

    let after_keyword = if trimmed.starts_with("let ") {
        &trimmed[4..]
    } else if trimmed.starts_with("var ") {
        &trimmed[4..]
    } else if trimmed.starts_with("const ") {
        &trimmed[6..]
    } else {
        return false;
    };

    let after_keyword = after_keyword.trim();

    let eq_pos = match find_first_eq(after_keyword) {
        Some(pos) => pos,
        None => return false,
    };
    let name = after_keyword[..eq_pos].trim().to_string();
    let value_str = after_keyword[eq_pos + 1..].trim().trim_end_matches(';').trim();

    if name.is_empty() {
        return false;
    }

    compile_expr_standalone(builder, value_str);
    builder.store_var(name);

    true
}

/// 独立的表达式编译函数
///
/// 用于在没有 EjsCompiler 实例的场景下编译表达式。
///
/// # 参数
/// - `builder`: IR 构建器
/// - `input`: 表达式文本
fn compile_expr_standalone(builder: &mut IRBuilder, input: &str) {
    let input = input.trim();
    if input.is_empty() {
        builder.push_null();
        return;
    }

    if compile_ternary_standalone(builder, input) {
        return;
    }

    if compile_binary_standalone(builder, input) {
        return;
    }

    if compile_unary_standalone(builder, input) {
        return;
    }

    if compile_literal_standalone(builder, input) {
        return;
    }

    if compile_call_or_access_standalone(builder, input) {
        return;
    }

    builder.load_var(input.to_string());
}

/// 独立编译三元表达式
fn compile_ternary_standalone(builder: &mut IRBuilder, input: &str) -> bool {
    let question_pos = match find_char_not_in_string(input, '?') {
        Some(pos) => pos,
        None => return false,
    };
    if question_pos == 0 {
        return false;
    }

    let colon_pos = match find_char_not_in_string(&input[question_pos + 1..], ':') {
        Some(pos) => pos,
        None => return false,
    };
    let colon_abs = question_pos + 1 + colon_pos;

    compile_expr_standalone(builder, &input[..question_pos]);
    let jump_if_false_ip = builder.emit_jump_if_false();

    compile_expr_standalone(builder, &input[question_pos + 1..colon_abs]);
    let jump_end_ip = builder.emit_jump();

    builder.patch_jump(jump_if_false_ip);
    compile_expr_standalone(builder, &input[colon_abs + 1..]);
    builder.patch_jump(jump_end_ip);

    true
}

/// 独立编译二元运算表达式
fn compile_binary_standalone(builder: &mut IRBuilder, input: &str) -> bool {
    let operators_by_precedence: &[(&str, BinaryOp)] = &[
        ("||", BinaryOp::Or),
        ("&&", BinaryOp::And),
        ("===", BinaryOp::Eq),
        ("!==", BinaryOp::Ne),
        ("==", BinaryOp::Eq),
        ("!=", BinaryOp::Ne),
        (">=", BinaryOp::Ge),
        ("<=", BinaryOp::Le),
        (">", BinaryOp::Gt),
        ("<", BinaryOp::Lt),
        ("+", BinaryOp::Add),
        ("-", BinaryOp::Sub),
        ("*", BinaryOp::Mul),
        ("/", BinaryOp::Div),
        ("%", BinaryOp::Mod),
    ];

    for (op_str, op) in operators_by_precedence {
        if let Some(pos) = find_operator_not_in_string(input, op_str) {
            if pos == 0 || pos + op_str.len() >= input.len() {
                continue;
            }
            compile_expr_standalone(builder, &input[..pos]);
            compile_expr_standalone(builder, &input[pos + op_str.len()..]);
            builder.binary_op(*op);
            return true;
        }
    }

    false
}

/// 独立编译一元运算表达式
fn compile_unary_standalone(builder: &mut IRBuilder, input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.starts_with('!') {
        let inner = &trimmed[1..];
        if !inner.is_empty() {
            compile_expr_standalone(builder, inner);
            builder.unary_op(UnaryOp::Not);
            return true;
        }
    }

    if trimmed.starts_with('-') {
        let inner = &trimmed[1..];
        if !inner.is_empty() {
            if let Some(n) = inner.trim().parse::<f64>().ok() {
                builder.push_number(-n);
                return true;
            }
            compile_expr_standalone(builder, inner);
            builder.unary_op(UnaryOp::Neg);
            return true;
        }
    }

    false
}

/// 独立编译字面量
fn compile_literal_standalone(builder: &mut IRBuilder, input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed == "true" || trimmed == "True" {
        builder.push_bool(true);
        return true;
    }
    if trimmed == "false" || trimmed == "False" {
        builder.push_bool(false);
        return true;
    }
    if trimmed == "null" || trimmed == "undefined" {
        builder.push_null();
        return true;
    }

    if let Ok(n) = trimmed.parse::<f64>() {
        builder.push_number(n);
        return true;
    }

    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
        || (trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() >= 2)
    {
        builder.push_string(trimmed[1..trimmed.len() - 1].to_string());
        return true;
    }

    false
}

/// 独立编译函数调用或属性访问表达式
fn compile_call_or_access_standalone(builder: &mut IRBuilder, input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    if !trimmed
        .chars()
        .next()
        .map_or(false, |c| c.is_alphabetic() || c == '_' || c == '$')
    {
        return false;
    }

    let mut has_complex = false;
    let mut name_buf = String::new();
    let mut i = 0;
    let chars: Vec<char> = trimmed.chars().collect();
    let mut pending_calls: Vec<(String, usize)> = Vec::new();

    while i < chars.len() {
        let ch = chars[i];

        if ch == '.' {
            if name_buf.is_empty() {
                return false;
            }
            has_complex = true;
            name_buf.push('.');
            i += 1;
            continue;
        }

        if ch == '[' {
            has_complex = true;
            if !name_buf.is_empty() {
                emit_access_chain(builder, &name_buf);
                name_buf.clear();
            }
            let bracket_content = match extract_bracket_content(&chars, i) {
                Some(c) => c,
                None => return false,
            };
            compile_expr_standalone(builder, &bracket_content);
            builder.index_access();
            i = skip_bracket(&chars, i);
            continue;
        }

        if ch == '(' {
            if name_buf.is_empty() {
                return false;
            }
            has_complex = true;
            let callee = name_buf.clone();
            name_buf.clear();

            let paren_content = match extract_paren_content_from_chars(&chars, i) {
                Some(c) => c,
                None => return false,
            };
            let args = split_call_args(&paren_content);
            let arg_count = args.len();

            for arg in &args {
                compile_expr_standalone(builder, arg);
            }

            pending_calls.push((callee, arg_count));

            i = skip_paren(&chars, i);
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' || ch == '$' {
            name_buf.push(ch);
            i += 1;
        } else {
            break;
        }
    }

    if !has_complex {
        return false;
    }

    if !name_buf.is_empty() {
        emit_access_chain(builder, &name_buf);
    }

    for (callee, arg_count) in pending_calls {
        builder.call_filter(callee, arg_count as u8);
    }

    true
}

/// 发射属性访问链指令
///
/// 将 `obj.prop.field` 形式的访问链编译为
/// load_var + field_access 指令序列。
///
/// # 参数
/// - `builder`: IR 构建器
/// - `chain`: 以 `.` 分隔的访问链字符串
fn emit_access_chain(builder: &mut IRBuilder, chain: &str) {
    let parts: Vec<&str> = chain.split('.').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return;
    }

    builder.load_var(parts[0].to_string());

    for part in &parts[1..] {
        builder.field_access(part.to_string());
    }
}

/// 查找字符串中第一个不在引号内的指定字符
///
/// # 参数
/// - `s`: 源字符串
/// - `target`: 目标字符
///
/// # 返回值
/// 目标字符的字节位置，若未找到则返回 None
fn find_char_not_in_string(s: &str, target: char) -> Option<usize> {
    let mut in_string = false;
    let mut string_char = ' ';
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == target {
            let byte_pos = chars[..i].iter().collect::<String>().len();
            return Some(byte_pos);
        }
    }

    None
}

/// 在字符串中查找不在引号内的运算符
///
/// 对于 `||` 和 `&&` 从右向左查找以正确处理优先级。
/// 对于其他运算符从左向右查找。
///
/// # 参数
/// - `s`: 源字符串
/// - `op`: 运算符字符串
///
/// # 返回值
/// 运算符的起始字节位置，若未找到则返回 None
fn find_operator_not_in_string(s: &str, op: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let op_chars: Vec<char> = op.chars().collect();
    let op_len = op_chars.len();

    let mut in_string = false;
    let mut string_char = ' ';
    let mut candidates = Vec::new();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            i += 1;
            continue;
        }

        if i + op_len <= chars.len() {
            let slice: String = chars[i..i + op_len].iter().collect();
            if slice == op {
                if op == "==" || op == "!=" {
                    if i + op_len < chars.len() && chars[i + op_len] == '=' {
                        i += op_len + 1;
                        continue;
                    }
                }
                if op == "=" {
                    if i > 0
                        && (chars[i - 1] == '!'
                            || chars[i - 1] == '<'
                            || chars[i - 1] == '>')
                    {
                        i += op_len;
                        continue;
                    }
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        i += op_len;
                        continue;
                    }
                }
                if op == ">" || op == "<" {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        i += op_len + 1;
                        continue;
                    }
                }
                if op == "!" {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        i += op_len + 1;
                        continue;
                    }
                }
                let byte_pos = chars[..i].iter().collect::<String>().len();
                candidates.push(byte_pos);
            }
        }

        i += 1;
    }

    if op == "||" || op == "&&" {
        candidates.last().copied()
    } else {
        candidates.first().copied()
    }
}

/// 查找字符串中第一个不在引号内的赋值等号位置
///
/// 用于区分赋值 `=` 和比较运算符 `==`, `!=`, `<=`, `>=`。
///
/// # 参数
/// - `s`: 源字符串
///
/// # 返回值
/// 第一个赋值等号的位置，若未找到则返回 None
fn find_first_eq(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut in_string = false;
    let mut string_char = ' ';

    for i in 0..chars.len() {
        let ch = chars[i];

        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }

        if ch == '=' {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                continue;
            }
            if i > 0 && (chars[i - 1] == '!' || chars[i - 1] == '<' || chars[i - 1] == '>') {
                continue;
            }
            let byte_pos = chars[..i].iter().collect::<String>().len();
            return Some(byte_pos);
        }
    }

    None
}

/// 从字符数组中提取方括号内的内容
///
/// # 参数
/// - `chars`: 字符数组
/// - `start`: 左方括号的位置
///
/// # 返回值
/// 方括号内的文本，若格式不匹配则返回 None
fn extract_bracket_content(chars: &[char], start: usize) -> Option<String> {
    if start >= chars.len() || chars[start] != '[' {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';

    for i in start..chars.len() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                let content: String = chars[start + 1..i].iter().collect();
                return Some(content.trim().to_string());
            }
        }
    }

    None
}

/// 跳过方括号表达式，返回右方括号之后的位置
///
/// # 参数
/// - `chars`: 字符数组
/// - `start`: 左方括号的位置
///
/// # 返回值
/// 右方括号之后的位置
fn skip_bracket(chars: &[char], start: usize) -> usize {
    let mut depth = 0;
    for i in start..chars.len() {
        if chars[i] == '[' {
            depth += 1;
        } else if chars[i] == ']' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    chars.len()
}

/// 从字符数组中提取圆括号内的内容
///
/// # 参数
/// - `chars`: 字符数组
/// - `start`: 左圆括号的位置
///
/// # 返回值
/// 圆括号内的文本，若格式不匹配则返回 None
fn extract_paren_content_from_chars(chars: &[char], start: usize) -> Option<String> {
    if start >= chars.len() || chars[start] != '(' {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';

    for i in start..chars.len() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                let content: String = chars[start + 1..i].iter().collect();
                return Some(content.trim().to_string());
            }
        }
    }

    None
}

/// 跳过圆括号表达式，返回右圆括号之后的位置
///
/// # 参数
/// - `chars`: 字符数组
/// - `start`: 左圆括号的位置
///
/// # 返回值
/// 右圆括号之后的位置
fn skip_paren(chars: &[char], start: usize) -> usize {
    let mut depth = 0;
    for i in start..chars.len() {
        if chars[i] == '(' {
            depth += 1;
        } else if chars[i] == ')' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    chars.len()
}

/// 分割函数调用参数列表
///
/// 按逗号分隔参数，返回每个参数的文本。
///
/// # 参数
/// - `content`: 逗号分隔的参数文本
///
/// # 返回值
/// 参数文本列表
fn split_call_args(content: &str) -> Vec<String> {
    if content.trim().is_empty() {
        return vec![];
    }

    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let chars: Vec<char> = content.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            in_string = true;
            string_char = ch;
            continue;
        }
        if ch == '(' || ch == '[' || ch == '{' {
            depth += 1;
        } else if ch == ')' || ch == ']' || ch == '}' {
            depth -= 1;
        } else if ch == ',' && depth == 0 {
            let arg_str: String = chars[start..i].iter().collect();
            args.push(arg_str.trim().to_string());
            start = i + 1;
        }
    }

    let remaining: String = chars[start..].iter().collect();
    if !remaining.trim().is_empty() {
        args.push(remaining.trim().to_string());
    }

    args
}
