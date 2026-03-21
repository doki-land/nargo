#![warn(missing_docs)]

/// 编译后的模板 IR
///
/// 扁平指令序列，由前端编译生成，由 VM 执行
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateIR {
    /// 指令序列
    pub instructions: Vec<Instruction>,
}

/// VM 指令枚举
///
/// 所有模板操作被编译为扁平指令序列。
/// 控制流通过跳转指令实现，表达式通过栈操作指令实现。
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // === 常量加载 ===
    /// 将 null 压入操作数栈
    PushNull,
    /// 将布尔值压入操作数栈
    PushBool(bool),
    /// 将数字压入操作数栈
    PushNumber(f64),
    /// 将字符串压入操作数栈
    PushString(String),

    // === 变量操作 ===
    /// 从上下文加载变量，压入操作数栈
    LoadVar(String),
    /// 弹出操作数栈顶值，绑定到当前作用域的变量
    StoreVar(String),

    // === 访问操作 ===
    /// 弹出对象，压入对象.字段
    FieldAccess(String),
    /// 弹出索引和对象，压入对象[索引]
    IndexAccess,

    // === 运算 ===
    /// 弹出右操作数和左操作数，执行二元运算，压入结果
    BinaryOp(BinaryOp),
    /// 弹出操作数，执行一元运算，压入结果
    UnaryOp(UnaryOp),

    // === 输出 ===
    /// 弹出操作数栈顶值，追加到输出缓冲区
    Output,
    /// 弹出操作数栈顶值，追加到输出缓冲区（不转义）
    OutputRaw,
    /// 将文本直接追加到输出缓冲区（不操作栈）
    OutputText(String),

    // === 控制流 ===
    /// 无条件跳转到指定指令位置
    Jump(usize),
    /// 弹出条件值，若为假则跳转到指定指令位置
    JumpIfFalse(usize),
    /// 弹出条件值，若为真则跳转到指定指令位置
    JumpIfTrue(usize),

    // === 循环 ===
    /// 弹出可迭代对象，创建迭代器；若为空跳转到 loop_end；否则压入新作用域并绑定第一个元素
    ForInit {
        /// 循环变量模式
        pattern: ForPattern,
        /// 循环结束时的跳转目标
        loop_end: usize,
    },
    /// 推进迭代器；若有下一个元素则绑定并跳转到 loop_start；否则弹出作用域继续执行
    ForNext {
        /// 循环起始跳转目标
        loop_start: usize,
    },

    // === 过滤器 ===
    /// 弹出 arg_count 个参数和输入值，应用过滤器，压入结果
    CallFilter {
        /// 过滤器名称
        name: String,
        /// 过滤器参数数量
        arg_count: u8,
    },

    // === 模板 ===
    /// 弹出模板名称，包含并执行模板
    Include {
        /// 是否将上下文传递给被包含的模板
        with_context: bool,
    },

    // === 作用域 ===
    /// 压入新的变量作用域
    ScopeBegin,
    /// 弹出当前变量作用域
    ScopeEnd,
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// 加法
    Add,
    /// 减法
    Sub,
    /// 乘法
    Mul,
    /// 除法
    Div,
    /// 取模
    Mod,
    /// 等于
    Eq,
    /// 不等于
    Ne,
    /// 小于
    Lt,
    /// 小于等于
    Le,
    /// 大于
    Gt,
    /// 大于等于
    Ge,
    /// 逻辑与
    And,
    /// 逻辑或
    Or,
}

/// 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// 逻辑非
    Not,
    /// 负号
    Neg,
}

/// for 循环变量模式
#[derive(Debug, Clone, PartialEq)]
pub enum ForPattern {
    /// 单个标识符
    Identifier(String),
    /// 元组解构
    Tuple(Vec<String>),
}

/// IR 编译辅助器
///
/// 提供便捷的指令发射和跳转标签修补 API
pub struct IRBuilder {
    instructions: Vec<Instruction>,
}

impl IRBuilder {
    /// 创建新的 IRBuilder
    pub fn new() -> Self {
        Self { instructions: Vec::new() }
    }

    /// 将 null 压入操作数栈
    pub fn push_null(&mut self) {
        self.instructions.push(Instruction::PushNull);
    }

    /// 将布尔值压入操作数栈
    pub fn push_bool(&mut self, b: bool) {
        self.instructions.push(Instruction::PushBool(b));
    }

    /// 将数字压入操作数栈
    pub fn push_number(&mut self, n: f64) {
        self.instructions.push(Instruction::PushNumber(n));
    }

    /// 将字符串压入操作数栈
    pub fn push_string(&mut self, s: String) {
        self.instructions.push(Instruction::PushString(s));
    }

    /// 将字符串引用压入操作数栈
    pub fn push_string_ref(&mut self, s: &str) {
        self.push_string(s.to_string());
    }

    /// 从上下文加载变量
    pub fn load_var(&mut self, name: String) {
        self.instructions.push(Instruction::LoadVar(name));
    }

    /// 弹出栈顶值绑定到变量
    pub fn store_var(&mut self, name: String) {
        self.instructions.push(Instruction::StoreVar(name));
    }

    /// 弹出对象，访问字段
    pub fn field_access(&mut self, field: String) {
        self.instructions.push(Instruction::FieldAccess(field));
    }

    /// 弹出索引和对象，执行索引访问
    pub fn index_access(&mut self) {
        self.instructions.push(Instruction::IndexAccess);
    }

    /// 执行二元运算
    pub fn binary_op(&mut self, op: BinaryOp) {
        self.instructions.push(Instruction::BinaryOp(op));
    }

    /// 执行一元运算
    pub fn unary_op(&mut self, op: UnaryOp) {
        self.instructions.push(Instruction::UnaryOp(op));
    }

    /// 弹出栈顶值输出
    pub fn output(&mut self) {
        self.instructions.push(Instruction::Output);
    }

    /// 弹出栈顶值原样输出
    pub fn output_raw(&mut self) {
        self.instructions.push(Instruction::OutputRaw);
    }

    /// 直接输出文本
    pub fn output_text(&mut self, text: String) {
        self.instructions.push(Instruction::OutputText(text));
    }

    /// 直接输出文本引用
    pub fn output_text_ref(&mut self, text: &str) {
        self.output_text(text.to_string());
    }

    /// 发射 Jump 指令（占位目标 0），返回指令位置用于后续修补
    pub fn emit_jump(&mut self) -> usize {
        let ip = self.instructions.len();
        self.instructions.push(Instruction::Jump(0));
        ip
    }

    /// 发射 JumpIfFalse 指令（占位目标 0），返回指令位置用于后续修补
    pub fn emit_jump_if_false(&mut self) -> usize {
        let ip = self.instructions.len();
        self.instructions.push(Instruction::JumpIfFalse(0));
        ip
    }

    /// 发射 JumpIfTrue 指令（占位目标 0），返回指令位置用于后续修补
    pub fn emit_jump_if_true(&mut self) -> usize {
        let ip = self.instructions.len();
        self.instructions.push(Instruction::JumpIfTrue(0));
        ip
    }

    /// 发射 ForInit 指令（占位 loop_end = 0），返回指令位置用于后续修补
    pub fn emit_for_init(&mut self, pattern: ForPattern) -> usize {
        let ip = self.instructions.len();
        self.instructions.push(Instruction::ForInit { pattern, loop_end: 0 });
        ip
    }

    /// 发射 ForNext 指令
    pub fn emit_for_next(&mut self, loop_start: usize) {
        self.instructions.push(Instruction::ForNext { loop_start });
    }

    /// 调用过滤器
    pub fn call_filter(&mut self, name: String, arg_count: u8) {
        self.instructions.push(Instruction::CallFilter { name, arg_count });
    }

    /// 包含模板
    pub fn include(&mut self, with_context: bool) {
        self.instructions.push(Instruction::Include { with_context });
    }

    /// 压入新的变量作用域
    pub fn scope_begin(&mut self) {
        self.instructions.push(Instruction::ScopeBegin);
    }

    /// 弹出当前变量作用域
    pub fn scope_end(&mut self) {
        self.instructions.push(Instruction::ScopeEnd);
    }

    /// 修补指定位置的跳转指令目标为当前指令位置
    pub fn patch_jump(&mut self, ip: usize) {
        let target = self.instructions.len();
        self.patch_jump_to(ip, target);
    }

    /// 修补指定位置的跳转指令目标为指定指令位置
    pub fn patch_jump_to(&mut self, ip: usize, target: usize) {
        match &mut self.instructions[ip] {
            Instruction::Jump(t) => *t = target,
            Instruction::JumpIfFalse(t) => *t = target,
            Instruction::JumpIfTrue(t) => *t = target,
            Instruction::ForInit { loop_end, .. } => *loop_end = target,
            _ => panic!("Not a jump instruction at ip {}", ip),
        }
    }

    /// 获取当前指令位置
    pub fn current_ip(&self) -> usize {
        self.instructions.len()
    }

    /// 消费 builder，生成 TemplateIR
    pub fn build(self) -> TemplateIR {
        TemplateIR { instructions: self.instructions }
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
