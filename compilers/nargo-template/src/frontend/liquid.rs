#![warn(missing_docs)]

use crate::frontend::Frontend;
use crate::ir::*;
use crate::error::{TemplateError, TemplateResult};
use oak_core::{
    SourceText,
    parser::{Parser, ParseSession},
    tree::RedTree,
};
use oak_liquid::{LiquidElementType, LiquidLanguage, LiquidParser, LiquidTokenType};

/// Liquid 模板语言前端
///
/// 将 Liquid 模板源码编译为扁平 IR 指令序列。
/// 使用 `oak_liquid` 解析器将模板解析为 RedTree，
/// 然后遍历 RedTree 编译为 `TemplateIR`。
pub struct LiquidFrontend {
    /// Liquid 语言配置
    language: LiquidLanguage,
}

impl LiquidFrontend {
    /// 创建新的 Liquid 前端
    pub fn new() -> Self {
        Self {
            language: LiquidLanguage::default(),
        }
    }
}

impl Default for LiquidFrontend {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontend for LiquidFrontend {
    fn compile(&self, source: &str) -> TemplateResult<TemplateIR> {
        let source_text = SourceText::new(source);
        let parser = LiquidParser::new(&self.language);
        let mut cache = ParseSession::<LiquidLanguage>::default();
        let parse_output = parser.parse(&source_text, &[], &mut cache);
        let green_node = parse_output
            .result
            .map_err(|e| TemplateError::Syntax(format!("Liquid parser error: {}", e)))?;
        let red_tree = RedTree::new(green_node);
        let mut compiler = TreeCompiler::new(&source_text);
        compiler.compile_children(&red_tree)?;
        Ok(compiler.builder.build())
    }

    fn name(&self) -> &str {
        "liquid"
    }

    fn default_extension(&self) -> &str {
        "liquid"
    }
}

/// If 语句编译阶段
enum IfPhase {
    /// 尚未遇到条件关键字
    BeforeCondition,
    /// 正在解析条件表达式
    InCondition,
    /// 正在解析 then 体
    InThen,
    /// 正在解析 else 体
    InElse,
    /// 解析完成
    Done,
}

/// For 语句编译阶段
enum ForPhase {
    /// 尚未遇到 for 关键字
    BeforeVar,
    /// 正在解析循环变量
    InVar,
    /// 等待 in 关键字
    BeforeIn,
    /// 正在解析可迭代表达式
    InIterable,
    /// 正在解析循环体
    InBody,
    /// 解析完成
    Done,
}

/// RedTree 到扁平 IR 的编译器
///
/// 遍历 Liquid 解析器生成的 RedTree，将各节点编译为扁平 IR 指令序列。
struct TreeCompiler<'a> {
    /// 模板源文本引用，用于提取节点文本
    source: &'a SourceText,
    /// IR 指令构建器
    builder: IRBuilder,
}

impl<'a> TreeCompiler<'a> {
    /// 创建新的树编译器
    fn new(source: &'a SourceText) -> Self {
        Self {
            source,
            builder: IRBuilder::new(),
        }
    }

    /// 编译 RedTree 的所有子节点
    fn compile_children(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<()> {
        for child in tree.children() {
            self.compile_child(&child)?;
        }
        Ok(())
    }

    /// 编译单个 RedTree 子节点
    fn compile_child(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<()> {
        let kind: LiquidElementType = tree.kind();
        match kind {
            LiquidElementType::Text => {
                let text = tree.text(self.source).into_owned();
                if !text.is_empty() {
                    self.builder.output_text(text);
                }
                Ok(())
            }
            LiquidElementType::Variable => self.compile_variable(tree),
            LiquidElementType::IfStatement => {
                let jumps = self.compile_if(tree)?;
                for jump_ip in jumps {
                    self.builder.patch_jump(jump_ip);
                }
                Ok(())
            }
            LiquidElementType::ForStatement => self.compile_for(tree),
            LiquidElementType::Block => self.compile_block(tree),
            LiquidElementType::Comment => Ok(()),
            LiquidElementType::Tag => Ok(()),
            LiquidElementType::Error => {
                let text = tree.text(self.source);
                Err(TemplateError::Syntax(format!("Parse error at: {}", text)))
            }
            _ => Ok(()),
        }
    }

    /// 编译 Variable 节点
    ///
    /// Variable 节点对应 `{{ expr }}` 语法，编译表达式后输出结果。
    fn compile_variable(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<()> {
        self.compile_expression_from_children(tree)?;
        self.builder.output();
        Ok(())
    }

    /// 编译 IfStatement 节点
    ///
    /// 返回需要修补的跳转到末尾的指令位置列表。
    /// 调用者负责在所有分支编译完成后修补这些跳转。
    ///
    /// 编译策略：
    /// - 条件为假时跳过 then 体（JumpIfFalse）
    /// - then 体末尾跳转到整个 if 语句之后（Jump）
    /// - elsif 通过递归编译嵌套 IfStatement 实现
    /// - else 体直接跟在 JumpIfFalse 目标之后
    fn compile_if(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<Vec<usize>> {
        let mut jump_to_end_list: Vec<usize> = Vec::new();
        let mut current_jump_if_false: Option<usize> = None;
        let mut phase = IfPhase::BeforeCondition;
        let children: Vec<_> = tree.children().collect();

        for child in &children {
            let kind: LiquidElementType = child.kind();
            match phase {
                IfPhase::BeforeCondition => {
                    if self.is_keyword_leaf(child, "if")
                        || self.is_keyword_leaf(child, "elsif")
                    {
                        phase = IfPhase::InCondition;
                        continue;
                    }
                    if matches!(
                        kind,
                        LiquidElementType::Identifier
                            | LiquidElementType::Literal
                            | LiquidElementType::Expression
                            | LiquidElementType::Filter
                            | LiquidElementType::Function
                    ) {
                        self.compile_expression_node(child)?;
                        let jf = self.builder.emit_jump_if_false();
                        current_jump_if_false = Some(jf);
                        phase = IfPhase::InThen;
                        continue;
                    }
                }
                IfPhase::InCondition => {
                    if matches!(
                        kind,
                        LiquidElementType::Identifier
                            | LiquidElementType::Literal
                            | LiquidElementType::Expression
                            | LiquidElementType::Filter
                            | LiquidElementType::Function
                    ) {
                        self.compile_expression_node(child)?;
                        let jf = self.builder.emit_jump_if_false();
                        current_jump_if_false = Some(jf);
                        phase = IfPhase::InThen;
                        continue;
                    }
                }
                IfPhase::InThen => {
                    if kind == LiquidElementType::IfStatement {
                        let first_keyword = self.get_first_keyword(child);
                        if first_keyword == "elsif" {
                            let jump_to_end = self.builder.emit_jump();
                            jump_to_end_list.push(jump_to_end);
                            if let Some(jf) = current_jump_if_false.take() {
                                self.builder.patch_jump(jf);
                            }
                            let mut elsif_jumps = self.compile_if(child)?;
                            jump_to_end_list.append(&mut elsif_jumps);
                            phase = IfPhase::Done;
                            continue;
                        }
                    }
                    if self.is_keyword_leaf(child, "else") {
                        let jump_to_end = self.builder.emit_jump();
                        jump_to_end_list.push(jump_to_end);
                        if let Some(jf) = current_jump_if_false.take() {
                            self.builder.patch_jump(jf);
                        }
                        phase = IfPhase::InElse;
                        continue;
                    }
                    if self.is_keyword_leaf(child, "endif") {
                        if let Some(jf) = current_jump_if_false.take() {
                            self.builder.patch_jump(jf);
                        }
                        phase = IfPhase::Done;
                        continue;
                    }
                    self.compile_child(child)?;
                }
                IfPhase::InElse => {
                    if self.is_keyword_leaf(child, "endif") {
                        phase = IfPhase::Done;
                        continue;
                    }
                    self.compile_child(child)?;
                }
                IfPhase::Done => {}
            }
        }

        Ok(jump_to_end_list)
    }

    /// 编译 ForStatement 节点
    ///
    /// 编译策略：
    /// - 编译可迭代表达式，压入操作数栈
    /// - 发射 ForInit 指令（占位 loop_end）
    /// - 编译循环体
    /// - 发射 ForNext 指令（跳回循环体起始）
    /// - 修补 ForInit 的 loop_end 为 ForNext 之后的指令位置
    fn compile_for(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<()> {
        let mut pattern = ForPattern::Identifier("_".to_string());
        let mut phase = ForPhase::BeforeVar;
        let mut for_init_ip: Option<usize> = None;
        let children: Vec<_> = tree.children().collect();

        for child in &children {
            let kind: LiquidElementType = child.kind();
            match phase {
                ForPhase::BeforeVar => {
                    if self.is_keyword_leaf(child, "for") {
                        phase = ForPhase::InVar;
                        continue;
                    }
                }
                ForPhase::InVar => {
                    if kind == LiquidElementType::Identifier {
                        let text = child.text(self.source).into_owned();
                        pattern = ForPattern::Identifier(text);
                        phase = ForPhase::BeforeIn;
                        continue;
                    }
                }
                ForPhase::BeforeIn => {
                    if self.is_keyword_leaf(child, "in") {
                        phase = ForPhase::InIterable;
                        continue;
                    }
                }
                ForPhase::InIterable => {
                    if matches!(
                        kind,
                        LiquidElementType::Identifier
                            | LiquidElementType::Literal
                            | LiquidElementType::Expression
                            | LiquidElementType::Filter
                            | LiquidElementType::Function
                    ) {
                        self.compile_expression_node(child)?;
                        let ip = self.builder.emit_for_init(pattern.clone());
                        for_init_ip = Some(ip);
                        phase = ForPhase::InBody;
                        continue;
                    }
                }
                ForPhase::InBody => {
                    if self.is_keyword_leaf(child, "endfor") {
                        phase = ForPhase::Done;
                        continue;
                    }
                    self.compile_child(child)?;
                }
                ForPhase::Done => {}
            }
        }

        if let Some(ip) = for_init_ip {
            let loop_start = ip + 1;
            self.builder.emit_for_next(loop_start);
            self.builder.patch_jump(ip);
        }

        Ok(())
    }

    /// 编译 Block 节点
    ///
    /// Block 节点对应 `{% block name %}...{% endblock %}` 语法，
    /// 编译为作用域包裹的指令序列。
    fn compile_block(&mut self, tree: &RedTree<'a, LiquidLanguage>) -> TemplateResult<()> {
        let mut found_name = false;
        let children: Vec<_> = tree.children().collect();

        self.builder.scope_begin();

        for child in &children {
            let kind: LiquidElementType = child.kind();

            if !found_name && kind == LiquidElementType::Identifier {
                let text = child.text(self.source).into_owned();
                if text != "block" && text != "endblock" {
                    found_name = true;
                    continue;
                }
            }

            if self.is_keyword_leaf(child, "endblock") {
                break;
            }

            if found_name {
                self.compile_child(child)?;
            }
        }

        self.builder.scope_end();
        Ok(())
    }

    /// 从子节点中编译表达式
    ///
    /// 遍历子节点，找到第一个表达式类型的节点并编译，
    /// 将求值结果压入操作数栈。若未找到表达式节点则压入 null。
    fn compile_expression_from_children(
        &mut self,
        tree: &RedTree<'a, LiquidLanguage>,
    ) -> TemplateResult<()> {
        for child in tree.children() {
            let kind: LiquidElementType = child.kind();
            if matches!(
                kind,
                LiquidElementType::Identifier
                    | LiquidElementType::Literal
                    | LiquidElementType::Expression
                    | LiquidElementType::Filter
                    | LiquidElementType::Function
            ) {
                return self.compile_expression_node(&child);
            }
        }
        self.builder.push_null();
        Ok(())
    }

    /// 编译表达式类型的 RedTree 节点
    ///
    /// 根据 LiquidElementType 分派到对应的编译逻辑：
    /// - `Identifier` → 加载变量或链式字段访问
    /// - `Literal` → 压入字面量常量
    /// - `Expression` → 二元运算表达式
    /// - `Filter` → 过滤器调用
    /// - `Function` → 函数调用（暂按过滤器处理）
    fn compile_expression_node(
        &mut self,
        tree: &RedTree<'a, LiquidLanguage>,
    ) -> TemplateResult<()> {
        let kind: LiquidElementType = tree.kind();
        match kind {
            LiquidElementType::Identifier => {
                let text = tree.text(self.source).into_owned();
                self.compile_identifier_expr(&text);
                Ok(())
            }
            LiquidElementType::Literal => {
                let text = tree.text(self.source).into_owned();
                self.compile_literal_text(&text);
                Ok(())
            }
            LiquidElementType::Expression => self.compile_binary_expression(tree),
            LiquidElementType::Filter => self.compile_filter_expression(tree),
            LiquidElementType::Function => self.compile_function_expression(tree),
            _ => {
                self.builder.push_null();
                Ok(())
            }
        }
    }

    /// 编译标识符表达式
    ///
    /// 支持简单标识符和点号链式访问（如 `page.title`）。
    /// Liquid 特殊值 `nil`/`null` 映射为 `push_null()`，
    /// `true`/`false` 映射为 `push_bool()`。
    fn compile_identifier_expr(&mut self, text: &str) {
        let text = text.trim();
        if text == "true" {
            self.builder.push_bool(true);
            return;
        }
        if text == "false" {
            self.builder.push_bool(false);
            return;
        }
        if text == "nil" || text == "null" {
            self.builder.push_null();
            return;
        }

        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 1 {
            self.builder.load_var(text.to_string());
            return;
        }

        self.builder.load_var(parts[0].to_string());
        for part in &parts[1..] {
            self.builder.field_access(part.to_string());
        }
    }

    /// 编译字面量文本
    ///
    /// 根据文本内容判断字面量类型并压入操作数栈：
    /// - 布尔值（`true`/`false`）→ `push_bool`
    /// - 空值（`nil`/`null`）→ `push_null`
    /// - 数字 → `push_number`
    /// - 带引号字符串 → `push_string`（去除引号）
    /// - 其他 → `push_string`
    fn compile_literal_text(&mut self, text: &str) {
        let text = text.trim();

        if text == "true" {
            self.builder.push_bool(true);
            return;
        }
        if text == "false" {
            self.builder.push_bool(false);
            return;
        }
        if text == "nil" || text == "null" {
            self.builder.push_null();
            return;
        }

        if let Ok(n) = text.parse::<f64>() {
            self.builder.push_number(n);
            return;
        }

        if (text.starts_with('"') && text.ends_with('"') && text.len() >= 2)
            || (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
        {
            self.builder.push_string(text[1..text.len() - 1].to_string());
            return;
        }

        self.builder.push_string(text.to_string());
    }

    /// 编译二元运算表达式
    ///
    /// Expression 节点的子节点包含左操作数、运算符和右操作数。
    /// 采用左结合顺序依次编译操作数并发射二元运算指令。
    fn compile_binary_expression(
        &mut self,
        tree: &RedTree<'a, LiquidLanguage>,
    ) -> TemplateResult<()> {
        let mut operators: Vec<BinaryOp> = Vec::new();
        let children: Vec<_> = tree.children().collect();

        for child in &children {
            if let Some(op) = self.token_to_binary_op(child) {
                operators.push(op);
            }
        }

        let mut op_iter = operators.into_iter();
        let mut first = true;

        for child in &children {
            let child_kind: LiquidElementType = child.kind();
            match child_kind {
                LiquidElementType::Identifier => {
                    let text = child.text(self.source).into_owned();
                    self.compile_identifier_expr(&text);
                    if !first {
                        if let Some(op) = op_iter.next() {
                            self.builder.binary_op(op);
                        }
                    }
                    first = false;
                }
                LiquidElementType::Literal => {
                    let text = child.text(self.source).into_owned();
                    self.compile_literal_text(&text);
                    if !first {
                        if let Some(op) = op_iter.next() {
                            self.builder.binary_op(op);
                        }
                    }
                    first = false;
                }
                LiquidElementType::Expression => {
                    self.compile_binary_expression(child)?;
                    if !first {
                        if let Some(op) = op_iter.next() {
                            self.builder.binary_op(op);
                        }
                    }
                    first = false;
                }
                LiquidElementType::Filter => {
                    self.compile_filter_expression(child)?;
                    if !first {
                        if let Some(op) = op_iter.next() {
                            self.builder.binary_op(op);
                        }
                    }
                    first = false;
                }
                LiquidElementType::Function => {
                    self.compile_function_expression(child)?;
                    if !first {
                        if let Some(op) = op_iter.next() {
                            self.builder.binary_op(op);
                        }
                    }
                    first = false;
                }
                _ => {}
            }
        }

        if first {
            self.builder.push_null();
        }

        Ok(())
    }

    /// 编译过滤器表达式
    ///
    /// Filter 节点的子节点包含输入值、管道符和过滤器名称及参数。
    /// 编译顺序：输入值 → 参数 → `call_filter`。
    fn compile_filter_expression(
        &mut self,
        tree: &RedTree<'a, LiquidLanguage>,
    ) -> TemplateResult<()> {
        let mut filter_name = String::new();
        let mut filter_args: u8 = 0;
        let mut found_pipe = false;

        for child in tree.children() {
            let child_kind: LiquidElementType = child.kind();

            if !found_pipe {
                if self.is_token_type(&child, LiquidTokenType::Pipe) {
                    found_pipe = true;
                    continue;
                }
                match child_kind {
                    LiquidElementType::Identifier => {
                        let text = child.text(self.source).into_owned();
                        self.compile_identifier_expr(&text);
                    }
                    LiquidElementType::Literal => {
                        let text = child.text(self.source).into_owned();
                        self.compile_literal_text(&text);
                    }
                    LiquidElementType::Expression => {
                        self.compile_binary_expression(&child)?;
                    }
                    LiquidElementType::Filter => {
                        self.compile_filter_expression(&child)?;
                    }
                    LiquidElementType::Function => {
                        self.compile_function_expression(&child)?;
                    }
                    _ => {}
                }
            } else if filter_name.is_empty() {
                if child_kind == LiquidElementType::Identifier {
                    filter_name = child.text(self.source).into_owned();
                }
            } else {
                match child_kind {
                    LiquidElementType::Identifier => {
                        let text = child.text(self.source).into_owned();
                        self.compile_identifier_expr(&text);
                        filter_args += 1;
                    }
                    LiquidElementType::Literal => {
                        let text = child.text(self.source).into_owned();
                        self.compile_literal_text(&text);
                        filter_args += 1;
                    }
                    LiquidElementType::Expression => {
                        self.compile_binary_expression(&child)?;
                        filter_args += 1;
                    }
                    LiquidElementType::Filter => {
                        self.compile_filter_expression(&child)?;
                        filter_args += 1;
                    }
                    _ => {}
                }
            }
        }

        if !filter_name.is_empty() {
            self.builder.call_filter(filter_name, filter_args);
        }

        Ok(())
    }

    /// 编译函数调用表达式
    ///
    /// Function 节点的子节点包含函数名和参数列表。
    /// 暂时按过滤器调用处理：压入 null 作为输入，编译参数，调用过滤器。
    fn compile_function_expression(
        &mut self,
        tree: &RedTree<'a, LiquidLanguage>,
    ) -> TemplateResult<()> {
        let mut callee = String::new();
        let mut arg_count: u8 = 0;

        self.builder.push_null();

        for child in tree.children() {
            let child_kind: LiquidElementType = child.kind();
            match child_kind {
                LiquidElementType::Identifier => {
                    let text = child.text(self.source).into_owned();
                    if callee.is_empty() {
                        callee = text;
                    } else {
                        self.compile_identifier_expr(&text);
                        arg_count += 1;
                    }
                }
                LiquidElementType::Literal => {
                    let text = child.text(self.source).into_owned();
                    self.compile_literal_text(&text);
                    arg_count += 1;
                }
                LiquidElementType::Expression => {
                    self.compile_binary_expression(&child)?;
                    arg_count += 1;
                }
                LiquidElementType::Filter => {
                    self.compile_filter_expression(&child)?;
                    arg_count += 1;
                }
                _ => {}
            }
        }

        if !callee.is_empty() {
            self.builder.call_filter(callee, arg_count);
        }

        Ok(())
    }

    /// 判断 RedTree 节点是否为指定文本的关键字叶节点
    ///
    /// 检查节点是否为 Identifier 类型的叶节点，且文本内容与给定关键字匹配。
    fn is_keyword_leaf(&self, tree: &RedTree<'a, LiquidLanguage>, keyword: &str) -> bool {
        if let Some(leaf) = tree.as_token() {
            if leaf.kind == LiquidTokenType::Identifier {
                let text = tree.text(self.source);
                return text == keyword;
            }
        }
        false
    }

    /// 获取 IfStatement 节点的第一个关键字文本
    ///
    /// 用于区分 `if`、`elsif` 等不同类型的条件语句。
    fn get_first_keyword(&self, tree: &RedTree<'a, LiquidLanguage>) -> String {
        for child in tree.children() {
            if let Some(leaf) = child.as_token() {
                if leaf.kind == LiquidTokenType::Identifier {
                    return child.text(self.source).into_owned();
                }
            }
        }
        String::new()
    }

    /// 尝试将 RedTree 叶节点转换为 BinaryOp
    ///
    /// 根据 LiquidTokenType 映射运算符：
    /// - `Plus` → `Add`，`Minus` → `Sub`，`Star` → `Mul`，`Slash` → `Div`，`Percent` → `Mod`
    /// - `Eq` → `Eq`，`Lt` → `Lt`，`Gt` → `Gt`，`Bang` → `Ne`
    /// - `Amp` → `And`，`Pipe` → `Or`
    fn token_to_binary_op(&self, tree: &RedTree<'a, LiquidLanguage>) -> Option<BinaryOp> {
        if let Some(leaf) = tree.as_token() {
            match leaf.kind {
                LiquidTokenType::Plus => Some(BinaryOp::Add),
                LiquidTokenType::Minus => Some(BinaryOp::Sub),
                LiquidTokenType::Star => Some(BinaryOp::Mul),
                LiquidTokenType::Slash => Some(BinaryOp::Div),
                LiquidTokenType::Percent => Some(BinaryOp::Mod),
                LiquidTokenType::Eq => Some(BinaryOp::Eq),
                LiquidTokenType::Lt => Some(BinaryOp::Lt),
                LiquidTokenType::Gt => Some(BinaryOp::Gt),
                LiquidTokenType::Bang => Some(BinaryOp::Ne),
                LiquidTokenType::Amp => Some(BinaryOp::And),
                LiquidTokenType::Pipe => Some(BinaryOp::Or),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 判断 RedTree 节点是否为指定类型的叶节点
    fn is_token_type(
        &self,
        tree: &RedTree<'a, LiquidLanguage>,
        token_type: LiquidTokenType,
    ) -> bool {
        if let Some(leaf) = tree.as_token() {
            leaf.kind == token_type
        } else {
            false
        }
    }
}
