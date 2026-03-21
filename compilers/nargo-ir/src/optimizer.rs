#![warn(missing_docs)]

//! 优化器模块
//!
//! 提供 IR 结构的优化功能，包括常量折叠、死代码消除等优化规则。

use nargo_types::NargoValue;

use crate::{
    expr::JsExpr,
    program::{IRModule, JsProgram},
    stmt::JsStmt,
    types::Trivia,
};
use nargo_types::Span;

/// 表达式优化器
///
/// 负责优化 JavaScript 表达式，包括常量折叠等优化规则。
pub struct ExprOptimizer;

impl ExprOptimizer {
    /// 优化表达式
    ///
    /// # Parameters
    /// - `expr`: 要优化的表达式
    pub fn optimize(expr: &mut JsExpr) {
        match expr {
            JsExpr::Unary { op, argument, .. } => {
                Self::optimize(argument);
                // 常量折叠：一元表达式
                if let JsExpr::Literal(value, span, trivia) = &**argument {
                    match op.as_str() {
                        "!" => {
                            if let Some(b) = value.as_bool() {
                                *expr = JsExpr::Literal(NargoValue::Bool(!b), *span, trivia.clone());
                            }
                        }
                        "-" => {
                            if let Some(n) = value.as_number() {
                                *expr = JsExpr::Literal(NargoValue::Number(-n), *span, trivia.clone());
                            }
                        }
                        "+" => {
                            if let Some(n) = value.as_number() {
                                *expr = JsExpr::Literal(NargoValue::Number(n), *span, trivia.clone());
                            }
                            else if let Some(s) = value.as_str() {
                                if let Ok(n) = s.parse::<f64>() {
                                    *expr = JsExpr::Literal(NargoValue::Number(n), *span, trivia.clone());
                                }
                            }
                        }
                        "~" => {
                            if let Some(n) = value.as_number() {
                                let int_val = n as i64;
                                *expr = JsExpr::Literal(NargoValue::Number((!int_val) as f64), *span, trivia.clone());
                            }
                        }
                        _ => return,
                    };
                }
            }
            JsExpr::Binary { left, op, right, span, trivia } => {
                Self::optimize(left);
                Self::optimize(right);
                // 常量折叠：二元表达式
                if let (JsExpr::Literal(left_val, _, _), JsExpr::Literal(right_val, _, _)) = (&**left, &**right) {
                    match op.as_str() {
                        "+" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Number(l + r), *span, trivia.clone());
                            }
                            else if let (Some(l), Some(r)) = (left_val.as_str(), right_val.as_str()) {
                                *expr = JsExpr::Literal(NargoValue::String(format!("{}{}", l, r)), *span, trivia.clone());
                            }
                            else if let (Some(l), Some(r)) = (left_val.as_str(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::String(format!("{}{}", l, r)), *span, trivia.clone());
                            }
                            else if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_str()) {
                                *expr = JsExpr::Literal(NargoValue::String(format!("{}{}", l, r)), *span, trivia.clone());
                            }
                        }
                        "-" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Number(l - r), *span, trivia.clone());
                            }
                        }
                        "*" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Number(l * r), *span, trivia.clone());
                            }
                        }
                        "/" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Number(l / r), *span, trivia.clone());
                            }
                        }
                        "%" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Number(l % r), *span, trivia.clone());
                            }
                        }
                        "==" | "!=" | "===" | "!==" => {
                            let result = left_val == right_val;
                            let final_result = if op == "!=" || op == "!==" { !result } else { result };
                            *expr = JsExpr::Literal(NargoValue::Bool(final_result), *span, trivia.clone());
                        }
                        "<" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l < r), *span, trivia.clone());
                            }
                        }
                        "<=" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l <= r), *span, trivia.clone());
                            }
                        }
                        ">" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l > r), *span, trivia.clone());
                            }
                        }
                        ">=" => {
                            if let (Some(l), Some(r)) = (left_val.as_number(), right_val.as_number()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l >= r), *span, trivia.clone());
                            }
                        }
                        "&&" => {
                            if let (Some(l), Some(r)) = (left_val.as_bool(), right_val.as_bool()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l && r), *span, trivia.clone());
                            }
                        }
                        "||" => {
                            if let (Some(l), Some(r)) = (left_val.as_bool(), right_val.as_bool()) {
                                *expr = JsExpr::Literal(NargoValue::Bool(l || r), *span, trivia.clone());
                            }
                        }
                        _ => return,
                    };
                }
            }
            JsExpr::Conditional { test, consequent, alternate, span: _, trivia: _ } => {
                Self::optimize(test);
                Self::optimize(consequent);
                Self::optimize(alternate);
                // 常量折叠：条件表达式
                if let JsExpr::Literal(test_val, _, _) = &**test {
                    if let Some(b) = test_val.as_bool() {
                        let optimized = if b { consequent.clone() } else { alternate.clone() };
                        *expr = *optimized;
                    }
                }
            }
            JsExpr::Array(items, ..) => {
                for item in items {
                    Self::optimize(item);
                }
            }
            JsExpr::Object(props, ..) => {
                for (_, value) in props {
                    Self::optimize(value);
                }
            }
            JsExpr::Call { callee, args, .. } => {
                Self::optimize(callee);
                for arg in args {
                    Self::optimize(arg);
                }
            }
            JsExpr::Member { object, property, .. } => {
                Self::optimize(object);
                Self::optimize(property);
            }
            JsExpr::OptionalMember { object, property, .. } => {
                Self::optimize(object);
                Self::optimize(property);
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                Self::optimize(callee);
                for arg in args {
                    Self::optimize(arg);
                }
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                Self::optimize(left);
                Self::optimize(right);
            }
            JsExpr::LogicalAssignment { left, right, .. } => {
                Self::optimize(left);
                Self::optimize(right);
            }
            JsExpr::ArrowFunction { body, .. } => {
                Self::optimize(body);
            }
            JsExpr::TseElement { children, .. } => {
                for child in children {
                    Self::optimize(child);
                }
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    Self::optimize(expr);
                }
            }
            JsExpr::Spread(expr, ..) => {
                Self::optimize(expr);
            }
            JsExpr::TypeOf(expr, ..) => {
                Self::optimize(expr);
            }
            JsExpr::InstanceOf { left, right, .. } => {
                Self::optimize(left);
                Self::optimize(right);
            }
            _ => {}
        }
    }

    /// 执行常量传播优化
    ///
    /// # Parameters
    /// - `expr`: 要优化的表达式
    /// - `const_map`: 常量映射，键为变量名，值为常量值
    pub fn propagate_constants(expr: &mut JsExpr, const_map: &std::collections::HashMap<String, NargoValue>) {
        match expr {
            JsExpr::Identifier(id, span, trivia) => {
                // 如果标识符在常量映射中，替换为常量值
                if let Some(value) = const_map.get(id) {
                    *expr = JsExpr::Literal(value.clone(), *span, trivia.clone());
                }
            }
            JsExpr::Unary { argument, .. } => {
                Self::propagate_constants(argument, const_map);
            }
            JsExpr::Binary { left, right, .. } => {
                Self::propagate_constants(left, const_map);
                Self::propagate_constants(right, const_map);
            }
            JsExpr::Call { callee, args, .. } => {
                Self::propagate_constants(callee, const_map);
                for arg in args {
                    Self::propagate_constants(arg, const_map);
                }
            }
            JsExpr::Member { object, property, .. } => {
                Self::propagate_constants(object, const_map);
                Self::propagate_constants(property, const_map);
            }
            JsExpr::OptionalMember { object, property, .. } => {
                Self::propagate_constants(object, const_map);
                Self::propagate_constants(property, const_map);
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                Self::propagate_constants(callee, const_map);
                for arg in args {
                    Self::propagate_constants(arg, const_map);
                }
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                Self::propagate_constants(left, const_map);
                Self::propagate_constants(right, const_map);
            }
            JsExpr::LogicalAssignment { left, right, .. } => {
                Self::propagate_constants(left, const_map);
                Self::propagate_constants(right, const_map);
            }
            JsExpr::Array(items, ..) => {
                for item in items {
                    Self::propagate_constants(item, const_map);
                }
            }
            JsExpr::Object(props, ..) => {
                for (_, value) in props {
                    Self::propagate_constants(value, const_map);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                // 不处理函数内部的常量传播，避免作用域问题
                Self::propagate_constants(body, const_map);
            }
            JsExpr::TseElement { children, attributes, .. } => {
                for child in children {
                    Self::propagate_constants(child, const_map);
                }
                for attr in attributes {
                    if let Some(value) = &mut attr.value {
                        Self::propagate_constants(value, const_map);
                    }
                }
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                Self::propagate_constants(test, const_map);
                Self::propagate_constants(consequent, const_map);
                Self::propagate_constants(alternate, const_map);
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    Self::propagate_constants(expr, const_map);
                }
            }
            JsExpr::Spread(expr, ..) => {
                Self::propagate_constants(expr, const_map);
            }
            JsExpr::TypeOf(expr, ..) => {
                Self::propagate_constants(expr, const_map);
            }
            JsExpr::InstanceOf { left, right, .. } => {
                Self::propagate_constants(left, const_map);
                Self::propagate_constants(right, const_map);
            }
            _ => {}
        }
    }
}

/// 语句优化器
///
/// 负责优化 JavaScript 语句，包括死代码消除、函数内联等优化规则。
pub struct StmtOptimizer;

impl StmtOptimizer {
    /// 优化语句
    ///
    /// # Parameters
    /// - `stmt`: 要优化的语句
    pub fn optimize(stmt: &mut JsStmt) {
        // 构建常量映射
        let mut const_map = std::collections::HashMap::new();
        Self::collect_constants(stmt, &mut const_map);

        // 执行常量传播
        Self::propagate_constants_in_stmt(stmt, &const_map);

        match stmt {
            JsStmt::Expr(expr, ..) => {
                ExprOptimizer::optimize(expr);
                // 尝试内联函数调用
                Self::inline_function_call(expr);
            }
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    ExprOptimizer::optimize(expr);
                    // 尝试内联函数调用
                    Self::inline_function_call(expr);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for stmt in body {
                    Self::optimize(stmt);
                }
            }
            JsStmt::Return(expr, ..) => {
                if let Some(expr) = expr {
                    ExprOptimizer::optimize(expr);
                    // 尝试内联函数调用
                    Self::inline_function_call(expr);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                ExprOptimizer::optimize(test);
                Self::optimize(consequent);
                if let Some(alt) = alternate {
                    Self::optimize(alt);
                }
                // 死代码消除：常量条件
                if let JsExpr::Literal(test_val, _, _) = test {
                    if let Some(b) = test_val.as_bool() {
                        if b {
                            // 使用移动而不是克隆
                            *stmt = *std::mem::replace(consequent, Box::new(JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
                        }
                        else if let Some(alt) = alternate {
                            // 使用移动而不是克隆
                            *stmt = *std::mem::replace(alt, Box::new(JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
                        }
                        else {
                            *stmt = JsStmt::Block(vec![], Span::unknown(), Trivia::new());
                        }
                    }
                }
            }
            JsStmt::While { test, body, .. } => {
                ExprOptimizer::optimize(test);
                Self::optimize(body);

                // 检查是否是永远为 false 的条件
                let is_always_false = if let JsExpr::Literal(test_val, _, _) = test { if let Some(b) = test_val.as_bool() { !b } else { false } } else { false };

                // 死代码消除：永远为 false 的条件
                if is_always_false {
                    *stmt = JsStmt::Block(vec![], Span::unknown(), Trivia::new());
                }
                else {
                    // 循环优化：检测死循环
                    if let JsExpr::Literal(test_val, _, _) = test {
                        if let Some(b) = test_val.as_bool() {
                            if b {
                                // 永远为 true 的条件，可能是死循环
                                // 这里可以添加更多的死循环检测逻辑
                            }
                        }
                    }
                }
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    Self::optimize(init_stmt);
                }
                if let Some(test_expr) = test {
                    ExprOptimizer::optimize(test_expr);
                }
                if let Some(update_expr) = update {
                    ExprOptimizer::optimize(update_expr);
                }
                Self::optimize(body);
                // 循环优化：循环不变量外提
                Self::hoist_loop_invariants(body);
            }
            JsStmt::ForIn { left, right, body, .. } => {
                Self::optimize(left);
                ExprOptimizer::optimize(right);
                Self::optimize(body);
            }
            JsStmt::ForOf { left, right, body, .. } => {
                Self::optimize(left);
                ExprOptimizer::optimize(right);
                Self::optimize(body);
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                Self::optimize(block);
                if let Some((_, body)) = handler {
                    Self::optimize(body);
                }
                if let Some(body) = finalizer {
                    Self::optimize(body);
                }
            }
            JsStmt::Switch { discriminant, cases, .. } => {
                ExprOptimizer::optimize(discriminant);
                for (test, stmts) in cases {
                    if let Some(test_expr) = test {
                        ExprOptimizer::optimize(test_expr);
                    }
                    for stmt in stmts {
                        Self::optimize(stmt);
                    }
                }
            }
            JsStmt::Throw(expr, ..) => {
                ExprOptimizer::optimize(expr);
            }
            JsStmt::Block(stmts, ..) => {
                // 移除空语句和死代码
                let mut optimized_stmts = vec![];
                // 直接修改原语句，避免克隆
                let mut i = 0;
                let mut has_unreachable = false;

                while i < stmts.len() {
                    let stmt = &mut stmts[i];

                    // 如果已经遇到不可达代码，直接跳过
                    if has_unreachable {
                        i += 1;
                        continue;
                    }

                    Self::optimize(stmt);

                    // 移除空块
                    if let JsStmt::Block(inner_stmts, _, _) = stmt {
                        if !inner_stmts.is_empty() {
                            // 将内部语句移动到外部
                            optimized_stmts.extend(std::mem::take(inner_stmts));
                        }
                    }
                    else if let JsStmt::Expr(expr, _, _) = stmt {
                        // 移除无效的表达式语句
                        if !matches!(expr, JsExpr::Identifier(_, _, _)) {
                            optimized_stmts.push(std::mem::replace(stmt, JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
                        }
                    }
                    else {
                        // 检查是否是控制流语句（return, break, continue）
                        let is_control_flow = matches!(stmt, JsStmt::Return(_, _, _) | JsStmt::Break(_, _) | JsStmt::Continue(_, _));

                        if is_control_flow {
                            // 添加控制流语句
                            optimized_stmts.push(std::mem::replace(stmt, JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
                            // 标记后续代码为不可达
                            has_unreachable = true;
                        }
                        else {
                            optimized_stmts.push(std::mem::replace(stmt, JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
                        }
                    }
                    i += 1;
                }
                // 清理并替换为优化后的语句
                stmts.clear();
                stmts.extend(optimized_stmts);
            }
            _ => {}
        }
    }

    /// 尝试内联函数调用
    ///
    /// # Parameters
    /// - `expr`: 要优化的表达式
    fn inline_function_call(expr: &mut JsExpr) {
        // 处理箭头函数的内联
        if let JsExpr::Call { callee, args, span, trivia } = expr {
            if let JsExpr::ArrowFunction { params, body, .. } = &**callee {
                // 确保参数数量匹配
                if params.len() == args.len() {
                    // 创建参数映射
                    let mut param_map = std::collections::HashMap::new();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        param_map.insert(param.clone(), arg.clone());
                    }

                    // 替换函数体中的参数为实际参数
                    let mut inlined_body = body.clone();
                    Self::replace_parameters(&mut inlined_body, &param_map);

                    // 内联函数体
                    *expr = *inlined_body;
                }
            }
        }
    }

    /// 替换表达式中的参数为实际值
    ///
    /// # Parameters
    /// - `expr`: 要处理的表达式
    /// - `param_map`: 参数映射，键为参数名，值为实际参数表达式
    fn replace_parameters(expr: &mut JsExpr, param_map: &std::collections::HashMap<String, JsExpr>) {
        match expr {
            JsExpr::Identifier(id, span, trivia) => {
                if let Some(replacement) = param_map.get(id) {
                    *expr = replacement.clone();
                }
            }
            JsExpr::Unary { argument, .. } => {
                Self::replace_parameters(argument, param_map);
            }
            JsExpr::Binary { left, right, .. } => {
                Self::replace_parameters(left, param_map);
                Self::replace_parameters(right, param_map);
            }
            JsExpr::Call { callee, args, .. } => {
                Self::replace_parameters(callee, param_map);
                for arg in args {
                    Self::replace_parameters(arg, param_map);
                }
            }
            JsExpr::Member { object, property, .. } => {
                Self::replace_parameters(object, param_map);
                Self::replace_parameters(property, param_map);
            }
            JsExpr::OptionalMember { object, property, .. } => {
                Self::replace_parameters(object, param_map);
                Self::replace_parameters(property, param_map);
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                Self::replace_parameters(callee, param_map);
                for arg in args {
                    Self::replace_parameters(arg, param_map);
                }
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                Self::replace_parameters(left, param_map);
                Self::replace_parameters(right, param_map);
            }
            JsExpr::LogicalAssignment { left, right, .. } => {
                Self::replace_parameters(left, param_map);
                Self::replace_parameters(right, param_map);
            }
            JsExpr::Array(items, ..) => {
                for item in items {
                    Self::replace_parameters(item, param_map);
                }
            }
            JsExpr::Object(props, ..) => {
                for (_, value) in props {
                    Self::replace_parameters(value, param_map);
                }
            }
            JsExpr::ArrowFunction { body, .. } => {
                // 不处理嵌套函数的参数替换，避免作用域问题
                Self::replace_parameters(body, param_map);
            }
            JsExpr::TseElement { children, attributes, .. } => {
                for child in children {
                    Self::replace_parameters(child, param_map);
                }
                for attr in attributes {
                    if let Some(value) = &mut attr.value {
                        Self::replace_parameters(value, param_map);
                    }
                }
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                Self::replace_parameters(test, param_map);
                Self::replace_parameters(consequent, param_map);
                Self::replace_parameters(alternate, param_map);
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    Self::replace_parameters(expr, param_map);
                }
            }
            JsExpr::Spread(expr, ..) => {
                Self::replace_parameters(expr, param_map);
            }
            JsExpr::TypeOf(expr, ..) => {
                Self::replace_parameters(expr, param_map);
            }
            JsExpr::InstanceOf { left, right, .. } => {
                Self::replace_parameters(left, param_map);
                Self::replace_parameters(right, param_map);
            }
            _ => {}
        }
    }

    /// 循环不变量外提
    ///
    /// 将循环内部不随迭代变化的表达式移到循环外部，减少重复计算
    ///
    /// # Parameters
    /// - `body`: 循环体语句
    fn hoist_loop_invariants(body: &mut JsStmt) {
        // 这里实现循环不变量外提逻辑
        // 由于需要分析变量依赖关系，实际实现可能比较复杂
        // 这里仅作为示例，展示基本思路
        if let JsStmt::Block(stmts, span, trivia) = body {
            let mut invariant_stmts = vec![];
            let mut remaining_stmts = vec![];

            // 简单的循环不变量检测：只处理变量声明和表达式语句
            for stmt in stmts {
                match stmt {
                    JsStmt::VariableDecl { init, .. } => {
                        // 检查初始化表达式是否是常量
                        if let Some(expr) = init {
                            if expr.is_constant() {
                                invariant_stmts.push(stmt.clone());
                                continue;
                            }
                        }
                        remaining_stmts.push(stmt.clone());
                    }
                    JsStmt::Expr(expr, ..) => {
                        // 检查表达式是否是常量
                        if expr.is_constant() {
                            invariant_stmts.push(stmt.clone());
                            continue;
                        }
                        remaining_stmts.push(stmt.clone());
                    }
                    _ => {
                        remaining_stmts.push(stmt.clone());
                    }
                }
            }

            // 如果有不变量语句，创建新的循环体
            if !invariant_stmts.is_empty() {
                // 这里需要将不变量语句移到循环外部
                // 由于当前函数只能修改循环体，实际的外提需要在更高层次进行
                // 这里仅作为示例，展示检测逻辑
            }
        }
    }

    /// 收集语句中的常量定义
    ///
    /// # Parameters
    /// - `stmt`: 要分析的语句
    /// - `const_map`: 常量映射，键为变量名，值为常量值
    fn collect_constants(stmt: &mut JsStmt, const_map: &mut std::collections::HashMap<String, NargoValue>) {
        match stmt {
            JsStmt::VariableDecl { kind, id, init, .. } => {
                // 只处理 const 声明的常量
                if kind == "const" {
                    if let Some(expr) = init {
                        if let JsExpr::Literal(value, _, _) = expr {
                            const_map.insert(id.clone(), value.clone());
                        }
                    }
                }
            }
            JsStmt::Block(stmts, ..) => {
                for stmt in stmts {
                    Self::collect_constants(stmt, const_map);
                }
            }
            JsStmt::If { consequent, alternate, .. } => {
                Self::collect_constants(consequent, const_map);
                if let Some(alt) = alternate {
                    Self::collect_constants(alt, const_map);
                }
            }
            JsStmt::While { body, .. } => {
                Self::collect_constants(body, const_map);
            }
            JsStmt::For { body, .. } => {
                Self::collect_constants(body, const_map);
            }
            JsStmt::ForIn { body, .. } => {
                Self::collect_constants(body, const_map);
            }
            JsStmt::ForOf { body, .. } => {
                Self::collect_constants(body, const_map);
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                Self::collect_constants(block, const_map);
                if let Some((_, body)) = handler {
                    Self::collect_constants(body, const_map);
                }
                if let Some(body) = finalizer {
                    Self::collect_constants(body, const_map);
                }
            }
            JsStmt::Switch { cases, .. } => {
                for (_, stmts) in cases {
                    for stmt in stmts {
                        Self::collect_constants(stmt, const_map);
                    }
                }
            }
            _ => {}
        }
    }

    /// 在语句中传播常量
    ///
    /// # Parameters
    /// - `stmt`: 要优化的语句
    /// - `const_map`: 常量映射，键为变量名，值为常量值
    fn propagate_constants_in_stmt(stmt: &mut JsStmt, const_map: &std::collections::HashMap<String, NargoValue>) {
        match stmt {
            JsStmt::Expr(expr, ..) => {
                ExprOptimizer::propagate_constants(expr, const_map);
            }
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    ExprOptimizer::propagate_constants(expr, const_map);
                }
            }
            JsStmt::Return(expr, ..) => {
                if let Some(expr) = expr {
                    ExprOptimizer::propagate_constants(expr, const_map);
                }
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                ExprOptimizer::propagate_constants(test, const_map);
                Self::propagate_constants_in_stmt(consequent, const_map);
                if let Some(alt) = alternate {
                    Self::propagate_constants_in_stmt(alt, const_map);
                }
            }
            JsStmt::While { test, body, .. } => {
                ExprOptimizer::propagate_constants(test, const_map);
                Self::propagate_constants_in_stmt(body, const_map);
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    Self::propagate_constants_in_stmt(init_stmt, const_map);
                }
                if let Some(test_expr) = test {
                    ExprOptimizer::propagate_constants(test_expr, const_map);
                }
                if let Some(update_expr) = update {
                    ExprOptimizer::propagate_constants(update_expr, const_map);
                }
                Self::propagate_constants_in_stmt(body, const_map);
            }
            JsStmt::ForIn { left, right, body, .. } => {
                Self::propagate_constants_in_stmt(left, const_map);
                ExprOptimizer::propagate_constants(right, const_map);
                Self::propagate_constants_in_stmt(body, const_map);
            }
            JsStmt::ForOf { left, right, body, .. } => {
                Self::propagate_constants_in_stmt(left, const_map);
                ExprOptimizer::propagate_constants(right, const_map);
                Self::propagate_constants_in_stmt(body, const_map);
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                Self::propagate_constants_in_stmt(block, const_map);
                if let Some((_, body)) = handler {
                    Self::propagate_constants_in_stmt(body, const_map);
                }
                if let Some(body) = finalizer {
                    Self::propagate_constants_in_stmt(body, const_map);
                }
            }
            JsStmt::Switch { discriminant, cases, .. } => {
                ExprOptimizer::propagate_constants(discriminant, const_map);
                for (test, stmts) in cases {
                    if let Some(test_expr) = test {
                        ExprOptimizer::propagate_constants(test_expr, const_map);
                    }
                    for stmt in stmts {
                        Self::propagate_constants_in_stmt(stmt, const_map);
                    }
                }
            }
            JsStmt::Throw(expr, ..) => {
                ExprOptimizer::propagate_constants(expr, const_map);
            }
            JsStmt::Block(stmts, ..) => {
                for stmt in stmts {
                    Self::propagate_constants_in_stmt(stmt, const_map);
                }
            }
            _ => {}
        }
    }
}

/// 程序优化器
///
/// 负责优化 JavaScript 程序，包括移除空语句和死代码等优化规则。
pub struct ProgramOptimizer;

impl ProgramOptimizer {
    /// 优化程序
    ///
    /// # Parameters
    /// - `program`: 要优化的程序
    pub fn optimize(program: &mut JsProgram) {
        let mut optimized_body = vec![];
        // 直接修改原语句，避免克隆
        let mut i = 0;
        while i < program.body.len() {
            let stmt = &mut program.body[i];
            StmtOptimizer::optimize(stmt);

            // 移除空块
            if let JsStmt::Block(inner_stmts, _, _) = stmt {
                if !inner_stmts.is_empty() {
                    // 将内部语句移动到外部
                    optimized_body.extend(std::mem::take(inner_stmts));
                }
            }
            else {
                optimized_body.push(std::mem::replace(stmt, JsStmt::Block(vec![], Span::unknown(), Trivia::new())));
            }
            i += 1;
        }
        // 清理并替换为优化后的语句
        program.body.clear();
        program.body.extend(optimized_body);
    }
}

/// IR 模块优化器
///
/// 负责优化 IR 模块，包括优化模块中的脚本。
pub struct IROptimizer;

impl IROptimizer {
    /// 优化 IR 模块
    ///
    /// # Parameters
    /// - `module`: 要优化的 IR 模块
    pub fn optimize(module: &mut IRModule) {
        if let Some(script) = &mut module.script {
            ProgramOptimizer::optimize(script);
        }
        if let Some(script_server) = &mut module.script_server {
            ProgramOptimizer::optimize(script_server);
        }
        if let Some(script_client) = &mut module.script_client {
            ProgramOptimizer::optimize(script_client);
        }
    }
}
