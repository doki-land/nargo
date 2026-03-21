#![warn(missing_docs)]

//! 验证器模块
//!
//! 提供 IR 结构的验证功能，包括类型检查、作用域分析、语义验证等。

use nargo_types::{NargoValue, Result};
use std::collections::HashMap;

use crate::{
    expr::JsExpr,
    program::{IRModule, JsProgram},
    stmt::JsStmt,
    types::IRError,
};

/// 类型信息
///
/// 表示 JavaScript 表达式的类型。
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    /// 数字类型
    Number,
    /// 字符串类型
    String,
    /// 布尔类型
    Boolean,
    /// 对象类型
    Object,
    /// 数组类型
    Array,
    /// 函数类型
    Function,
    /// 未定义类型
    Undefined,
    /// 空类型
    Null,
    /// 任意类型
    Any,
}

/// 类型环境
pub struct TypeEnvironment {
    /// 变量类型映射
    variables: HashMap<String, TypeInfo>,
}

impl TypeEnvironment {
    /// 创建新的类型环境
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    /// 添加变量类型
    pub fn add_variable(&mut self, name: String, type_info: TypeInfo) {
        self.variables.insert(name, type_info);
    }

    /// 获取变量类型
    pub fn get_variable(&self, name: &String) -> Option<&TypeInfo> {
        self.variables.get(name)
    }
}

/// 表达式验证器
pub struct ExprValidator {
    /// 类型环境
    env: TypeEnvironment,
}

impl ExprValidator {
    /// 创建新的表达式验证器
    pub fn new() -> Self {
        Self { env: TypeEnvironment::new() }
    }

    /// 验证表达式
    pub fn validate(&mut self, expr: &JsExpr) -> Result<TypeInfo> {
        match expr {
            JsExpr::Identifier(id, _span, _) => {
                if let Some(type_info) = self.env.get_variable(id) {
                    Ok(type_info.clone())
                }
                else {
                    Err(IRError::InvalidExpression(format!("Undefined variable: {}", id)).into())
                }
            }
            JsExpr::Literal(value, _, _) => Ok(match value {
                NargoValue::Number(_) => TypeInfo::Number,
                NargoValue::String(_) => TypeInfo::String,
                NargoValue::Bool(_) => TypeInfo::Boolean,
                NargoValue::Object(_) => TypeInfo::Object,
                NargoValue::Array(_) => TypeInfo::Array,
                NargoValue::Null => TypeInfo::Null,
                _ => TypeInfo::Any,
            }),
            JsExpr::Unary { op, argument, span: _, .. } => {
                let arg_type = self.validate(argument)?;
                match op.as_str() {
                    "!" => Ok(TypeInfo::Boolean),
                    "-" | "+" | "~" => {
                        if matches!(arg_type, TypeInfo::Number) {
                            Ok(TypeInfo::Number)
                        }
                        else {
                            Err(IRError::InvalidExpression(format!("Unary operator {} requires number type", op)).into())
                        }
                    }
                    _ => Ok(TypeInfo::Any),
                }
            }
            JsExpr::Binary { left, op, right, span: _, .. } => {
                let left_type = self.validate(left)?;
                let right_type = self.validate(right)?;
                match op.as_str() {
                    "+" => {
                        if matches!(left_type, TypeInfo::Number) && matches!(right_type, TypeInfo::Number) {
                            Ok(TypeInfo::Number)
                        }
                        else if matches!(left_type, TypeInfo::String) || matches!(right_type, TypeInfo::String) {
                            Ok(TypeInfo::String)
                        }
                        else {
                            Err(IRError::InvalidExpression(format!("Addition operator requires number or string types").to_string()).into())
                        }
                    }
                    "-" | "*" | "/" | "%" => {
                        if matches!(left_type, TypeInfo::Number) && matches!(right_type, TypeInfo::Number) {
                            Ok(TypeInfo::Number)
                        }
                        else {
                            Err(IRError::InvalidExpression(format!("Arithmetic operator {} requires number types", op)).into())
                        }
                    }
                    "==" | "!=" | "===" | "!==" => Ok(TypeInfo::Boolean),
                    "<" | "<=" | ">" | ">=" => {
                        if matches!(left_type, TypeInfo::Number) && matches!(right_type, TypeInfo::Number) {
                            Ok(TypeInfo::Boolean)
                        }
                        else if matches!(left_type, TypeInfo::String) && matches!(right_type, TypeInfo::String) {
                            Ok(TypeInfo::Boolean)
                        }
                        else {
                            Err(IRError::InvalidExpression(format!("Comparison operator {} requires number or string types", op)).into())
                        }
                    }
                    "&&" | "||" => Ok(TypeInfo::Boolean),
                    "&" | "|" | "^" | "<<" | ">>" | ">>>" => {
                        if matches!(left_type, TypeInfo::Number) && matches!(right_type, TypeInfo::Number) {
                            Ok(TypeInfo::Number)
                        }
                        else {
                            Err(IRError::InvalidExpression(format!("Bitwise operator {} requires number types", op)).into())
                        }
                    }
                    "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => Ok(left_type),
                    _ => Ok(TypeInfo::Any),
                }
            }
            JsExpr::Call { callee, args, span: _, .. } => {
                let callee_type = self.validate(callee)?;
                if matches!(callee_type, TypeInfo::Function) {
                    // 验证参数
                    for arg in args {
                        self.validate(arg)?;
                    }
                    Ok(TypeInfo::Any)
                }
                else {
                    Err(IRError::InvalidExpression("Callee must be a function".to_string()).into())
                }
            }
            JsExpr::Member { object, property, .. } => {
                self.validate(object)?;
                self.validate(property)?;
                Ok(TypeInfo::Any)
            }
            JsExpr::OptionalMember { object, property, .. } => {
                self.validate(object)?;
                self.validate(property)?;
                Ok(TypeInfo::Any)
            }
            JsExpr::OptionalCall { callee, args, .. } => {
                self.validate(callee)?;
                for arg in args {
                    self.validate(arg)?;
                }
                Ok(TypeInfo::Any)
            }
            JsExpr::NullishCoalescing { left, right, .. } => {
                self.validate(left)?;
                self.validate(right)?;
                Ok(TypeInfo::Any)
            }
            JsExpr::LogicalAssignment { left, right, .. } => {
                self.validate(left)?;
                self.validate(right)?;
                Ok(TypeInfo::Any)
            }
            JsExpr::Array(items, ..) => {
                for item in items {
                    self.validate(item)?;
                }
                Ok(TypeInfo::Array)
            }
            JsExpr::Object(props, ..) => {
                for (_, value) in props {
                    self.validate(value)?;
                }
                Ok(TypeInfo::Object)
            }
            JsExpr::ArrowFunction { params, body, .. } => {
                // 添加参数到环境
                for param in params {
                    self.env.add_variable(param.clone(), TypeInfo::Any);
                }
                self.validate(body)?;
                Ok(TypeInfo::Function)
            }
            JsExpr::TseElement { children, .. } => {
                for child in children {
                    self.validate(child)?;
                }
                Ok(TypeInfo::Any)
            }
            JsExpr::Conditional { test, consequent, alternate, .. } => {
                let test_type = self.validate(test)?;
                if !matches!(test_type, TypeInfo::Boolean) {
                    return Err(IRError::InvalidExpression("Condition must be a boolean".to_string()).into());
                }
                let _consequent_type = self.validate(consequent)?;
                let _alternate_type = self.validate(alternate)?;
                // 简化处理，返回任意类型
                Ok(TypeInfo::Any)
            }
            JsExpr::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    self.validate(expr)?;
                }
                Ok(TypeInfo::String)
            }
            JsExpr::Spread(expr, ..) => {
                self.validate(expr)?;
                Ok(TypeInfo::Any)
            }
            JsExpr::TypeOf(expr, ..) => {
                self.validate(expr)?;
                Ok(TypeInfo::String)
            }
            JsExpr::InstanceOf { left, right, .. } => {
                self.validate(left)?;
                self.validate(right)?;
                Ok(TypeInfo::Boolean)
            }
            JsExpr::Other(_, _, _) => Ok(TypeInfo::Any),
        }
    }
}

/// 语句验证器
pub struct StmtValidator {
    /// 表达式验证器
    expr_validator: ExprValidator,
}

impl StmtValidator {
    /// 创建新的语句验证器
    pub fn new() -> Self {
        Self { expr_validator: ExprValidator::new() }
    }

    /// 验证语句
    pub fn validate(&mut self, stmt: &JsStmt) -> Result<()> {
        match stmt {
            JsStmt::Expr(expr, ..) => {
                self.expr_validator.validate(expr)?;
                Ok(())
            }
            JsStmt::VariableDecl { kind: _, id, init, .. } => {
                if let Some(expr) = init {
                    let type_info = self.expr_validator.validate(expr)?;
                    self.expr_validator.env.add_variable(id.clone(), type_info);
                }
                else {
                    // 未初始化的变量，默认为任意类型
                    self.expr_validator.env.add_variable(id.clone(), TypeInfo::Any);
                }
                Ok(())
            }
            JsStmt::Import { .. } => Ok(()),
            JsStmt::Export { declaration, .. } => self.validate(declaration),
            JsStmt::ExportAll { .. } => Ok(()),
            JsStmt::ExportNamed { .. } => Ok(()),
            JsStmt::FunctionDecl { id, params, body, .. } => {
                // 添加函数到环境
                self.expr_validator.env.add_variable(id.clone(), TypeInfo::Function);
                // 添加参数到环境
                for param in params {
                    self.expr_validator.env.add_variable(param.clone(), TypeInfo::Any);
                }
                // 验证函数体
                for stmt in body {
                    self.validate(stmt)?;
                }
                Ok(())
            }
            JsStmt::Return(expr, ..) => {
                if let Some(expr) = expr {
                    self.expr_validator.validate(expr)?;
                }
                Ok(())
            }
            JsStmt::If { test, consequent, alternate, .. } => {
                let test_type = self.expr_validator.validate(test)?;
                if !matches!(test_type, TypeInfo::Boolean) {
                    return Err(IRError::InvalidExpression("Condition must be a boolean".to_string()).into());
                }
                self.validate(consequent)?;
                if let Some(alt) = alternate {
                    self.validate(alt)?;
                }
                Ok(())
            }
            JsStmt::While { test, body, .. } => {
                let test_type = self.expr_validator.validate(test)?;
                if !matches!(test_type, TypeInfo::Boolean) {
                    return Err(IRError::InvalidExpression("Condition must be a boolean".to_string()).into());
                }
                self.validate(body)?;
                Ok(())
            }
            JsStmt::For { init, test, update, body, .. } => {
                if let Some(init_stmt) = init {
                    self.validate(init_stmt)?;
                }
                if let Some(test_expr) = test {
                    let test_type = self.expr_validator.validate(test_expr)?;
                    if !matches!(test_type, TypeInfo::Boolean) {
                        return Err(IRError::InvalidExpression("Condition must be a boolean".to_string()).into());
                    }
                }
                if let Some(update_expr) = update {
                    self.expr_validator.validate(update_expr)?;
                }
                self.validate(body)?;
                Ok(())
            }
            JsStmt::ForIn { left, right, body, .. } => {
                self.validate(left)?;
                self.expr_validator.validate(right)?;
                self.validate(body)?;
                Ok(())
            }
            JsStmt::ForOf { left, right, body, .. } => {
                self.validate(left)?;
                self.expr_validator.validate(right)?;
                self.validate(body)?;
                Ok(())
            }
            JsStmt::Try { block, handler, finalizer, .. } => {
                self.validate(block)?;
                if let Some((id, body)) = handler {
                    // 添加错误变量到环境
                    self.expr_validator.env.add_variable(id.clone(), TypeInfo::Any);
                    self.validate(body)?;
                }
                if let Some(body) = finalizer {
                    self.validate(body)?;
                }
                Ok(())
            }
            JsStmt::Switch { discriminant, cases, .. } => {
                self.expr_validator.validate(discriminant)?;
                for (test, stmts) in cases {
                    if let Some(test_expr) = test {
                        self.expr_validator.validate(test_expr)?;
                    }
                    for stmt in stmts {
                        self.validate(stmt)?;
                    }
                }
                Ok(())
            }
            JsStmt::Throw(expr, ..) => {
                self.expr_validator.validate(expr)?;
                Ok(())
            }
            JsStmt::Block(stmts, ..) => {
                for stmt in stmts {
                    self.validate(stmt)?;
                }
                Ok(())
            }
            JsStmt::Break(_, _) => Ok(()),
            JsStmt::Continue(_, _) => Ok(()),
            JsStmt::Other(_, _, _) => Ok(()),
        }
    }
}

/// 程序验证器
pub struct ProgramValidator {
    /// 语句验证器
    stmt_validator: StmtValidator,
}

impl ProgramValidator {
    /// 创建新的程序验证器
    pub fn new() -> Self {
        Self { stmt_validator: StmtValidator::new() }
    }

    /// 验证程序
    pub fn validate(&mut self, program: &JsProgram) -> Result<()> {
        for stmt in &program.body {
            self.stmt_validator.validate(stmt)?;
        }
        Ok(())
    }
}

/// IR 模块验证器
pub struct IRValidator {
    /// 程序验证器
    program_validator: ProgramValidator,
}

impl IRValidator {
    /// 创建新的 IR 模块验证器
    pub fn new() -> Self {
        Self { program_validator: ProgramValidator::new() }
    }

    /// 验证 IR 模块
    pub fn validate(&mut self, module: &IRModule) -> Result<()> {
        // 首先执行基本验证
        module.validate()?;

        // 然后执行类型检查和语义验证
        if let Some(script) = &module.script {
            self.program_validator.validate(script)?;
        }
        if let Some(script_server) = &module.script_server {
            self.program_validator.validate(script_server)?;
        }
        if let Some(script_client) = &module.script_client {
            self.program_validator.validate(script_client)?;
        }

        Ok(())
    }
}
