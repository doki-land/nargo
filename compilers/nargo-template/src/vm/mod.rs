#![warn(missing_docs)]

/// 渲染上下文模块
pub mod context;
/// 内置过滤器模块
pub mod filter;
/// 值操作模块
pub mod value;

use crate::error::{TemplateError, TemplateResult};
use crate::ir::{ForPattern, Instruction, TemplateIR};
use context::RenderContext;
use filter::apply_filter;
use nargo_types::NargoValue;
use std::collections::HashMap;
use value::*;

/// 栈式模板虚拟机
///
/// 执行扁平指令序列，使用操作数栈求值表达式，
/// 使用指令指针和跳转指令执行控制流。
pub struct VM {
    /// 已注册的模板 IR
    templates: HashMap<String, TemplateIR>,
}

/// 迭代器状态
struct IterState {
    /// 循环变量模式
    pattern: ForPattern,
    /// 剩余元素迭代器
    iter: std::vec::IntoIter<NargoValue>,
}

impl VM {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        Self { templates: HashMap::new() }
    }

    /// 注册编译后的 IR 模板
    pub fn register_ir(&mut self, name: &str, ir: TemplateIR) {
        self.templates.insert(name.to_string(), ir);
    }

    /// 渲染指定名称的模板
    pub fn render(&self, name: &str, context: &NargoValue) -> TemplateResult<String> {
        let ir = self.templates.get(name)
            .ok_or_else(|| TemplateError::TemplateNotFound(name.to_string()))?;
        self.execute(ir, context)
    }

    /// 执行 IR，返回渲染结果
    pub fn execute(&self, ir: &TemplateIR, context: &NargoValue) -> TemplateResult<String> {
        let mut stack: Vec<NargoValue> = Vec::new();
        let mut iterators: Vec<IterState> = Vec::new();
        let mut render_ctx = RenderContext::from_value(context);
        let mut output = String::new();
        let mut ip = 0usize;

        while ip < ir.instructions.len() {
            match &ir.instructions[ip] {
                Instruction::PushNull => {
                    stack.push(NargoValue::Null);
                    ip += 1;
                }
                Instruction::PushBool(b) => {
                    stack.push(NargoValue::Bool(*b));
                    ip += 1;
                }
                Instruction::PushNumber(n) => {
                    stack.push(NargoValue::Number(*n));
                    ip += 1;
                }
                Instruction::PushString(s) => {
                    stack.push(NargoValue::String(s.clone()));
                    ip += 1;
                }

                Instruction::LoadVar(name) => {
                    let val = render_ctx.get(name).cloned().unwrap_or(NargoValue::Null);
                    stack.push(val);
                    ip += 1;
                }
                Instruction::StoreVar(name) => {
                    let val = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on StoreVar".to_string())
                    })?;
                    render_ctx.set(name.clone(), val);
                    ip += 1;
                }

                Instruction::FieldAccess(field) => {
                    let obj = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on FieldAccess".to_string())
                    })?;
                    let result = match &obj {
                        NargoValue::Object(map) => map.get(field).cloned().unwrap_or(NargoValue::Null),
                        _ => NargoValue::Null,
                    };
                    stack.push(result);
                    ip += 1;
                }
                Instruction::IndexAccess => {
                    let idx = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on IndexAccess".to_string())
                    })?;
                    let obj = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on IndexAccess".to_string())
                    })?;
                    let result = match (&obj, &idx) {
                        (NargoValue::Array(arr), NargoValue::Number(n)) => {
                            let i = *n as usize;
                            if i < arr.len() { arr[i].clone() } else { NargoValue::Null }
                        }
                        (NargoValue::Object(map), NargoValue::String(key)) => {
                            map.get(key).cloned().unwrap_or(NargoValue::Null)
                        }
                        _ => NargoValue::Null,
                    };
                    stack.push(result);
                    ip += 1;
                }

                Instruction::BinaryOp(op) => {
                    let right = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on BinaryOp".to_string())
                    })?;
                    let left = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on BinaryOp".to_string())
                    })?;
                    let result = evaluate_binary_op(op, &left, &right)?;
                    stack.push(result);
                    ip += 1;
                }
                Instruction::UnaryOp(op) => {
                    let val = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on UnaryOp".to_string())
                    })?;
                    let result = evaluate_unary_op(op, &val)?;
                    stack.push(result);
                    ip += 1;
                }

                Instruction::Output => {
                    let val = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on Output".to_string())
                    })?;
                    output.push_str(&value_to_string(&val));
                    ip += 1;
                }
                Instruction::OutputRaw => {
                    let val = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on OutputRaw".to_string())
                    })?;
                    output.push_str(&value_to_string(&val));
                    ip += 1;
                }
                Instruction::OutputText(text) => {
                    output.push_str(text);
                    ip += 1;
                }

                Instruction::Jump(target) => {
                    ip = *target;
                }
                Instruction::JumpIfFalse(target) => {
                    let cond = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on JumpIfFalse".to_string())
                    })?;
                    if !is_truthy(&cond) { ip = *target; } else { ip += 1; }
                }
                Instruction::JumpIfTrue(target) => {
                    let cond = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on JumpIfTrue".to_string())
                    })?;
                    if is_truthy(&cond) { ip = *target; } else { ip += 1; }
                }

                Instruction::ForInit { pattern, loop_end } => {
                    let iterable = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on ForInit".to_string())
                    })?;
                    let arr = match &iterable {
                        NargoValue::Array(items) => items.clone(),
                        _ => return Err(TemplateError::Render(
                            "For loop iterable is not an array".to_string(),
                        )),
                    };
                    if arr.is_empty() {
                        ip = *loop_end;
                    } else {
                        render_ctx.push_scope();
                        let mut iter = arr.into_iter();
                        if let Some(first) = iter.next() {
                            bind_pattern_to_scope(&mut render_ctx, pattern, first);
                        }
                        iterators.push(IterState { pattern: pattern.clone(), iter });
                        ip += 1;
                    }
                }
                Instruction::ForNext { loop_start } => {
                    if let Some(iter_state) = iterators.last_mut() {
                        if let Some(next_val) = iter_state.iter.next() {
                            bind_pattern_to_scope(&mut render_ctx, &iter_state.pattern, next_val);
                            ip = *loop_start;
                        } else {
                            iterators.pop();
                            render_ctx.pop_scope();
                            ip += 1;
                        }
                    } else {
                        ip += 1;
                    }
                }

                Instruction::CallFilter { name, arg_count } => {
                    let arg_count = *arg_count as usize;
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(stack.pop().ok_or_else(|| {
                            TemplateError::Render("Stack underflow on CallFilter args".to_string())
                        })?);
                    }
                    args.reverse();
                    let input = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on CallFilter input".to_string())
                    })?;
                    let result = apply_filter(name, &input, &args)?;
                    stack.push(result);
                    ip += 1;
                }

                Instruction::Include { with_context } => {
                    let template_name_val = stack.pop().ok_or_else(|| {
                        TemplateError::Render("Stack underflow on Include".to_string())
                    })?;
                    let template_name = match &template_name_val {
                        NargoValue::String(s) => s.clone(),
                        _ => return Err(TemplateError::Render(
                            "Include template name must be a string".to_string(),
                        )),
                    };
                    let included_ir = self.templates.get(&template_name)
                        .ok_or_else(|| TemplateError::TemplateNotFound(template_name))?;
                    if *with_context {
                        let included = self.execute(included_ir, context)?;
                        output.push_str(&included);
                    } else {
                        let empty_ctx = NargoValue::Null;
                        let included = self.execute(included_ir, &empty_ctx)?;
                        output.push_str(&included);
                    }
                    ip += 1;
                }

                Instruction::ScopeBegin => {
                    render_ctx.push_scope();
                    ip += 1;
                }
                Instruction::ScopeEnd => {
                    render_ctx.pop_scope();
                    ip += 1;
                }
            }
        }
        Ok(output)
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

/// 将模式与值绑定到当前作用域
fn bind_pattern_to_scope(ctx: &mut RenderContext, pattern: &ForPattern, value: NargoValue) {
    match pattern {
        ForPattern::Identifier(name) => {
            ctx.set(name.clone(), value);
        }
        ForPattern::Tuple(names) => {
            if let NargoValue::Array(arr) = &value {
                for (i, name) in names.iter().enumerate() {
                    if i < arr.len() {
                        ctx.set(name.clone(), arr[i].clone());
                    }
                }
            }
        }
    }
}
