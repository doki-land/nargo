//! 表达式类型检查模块
//!
//! 包含表达式类型检查相关的代码

use crate::modules::types::{Type, TypeError};

/// 表达式类型检查器
pub trait ExpressionChecker {
    /// 类型检查程序
    ///
    /// # 参数
    ///
    /// * `program` - 程序
    fn check_program(&mut self, program: &nargo_ir::JsProgram);

    /// 类型检查语句
    ///
    /// # 参数
    ///
    /// * `stmt` - 语句
    fn check_statement(&mut self, stmt: &nargo_ir::JsStmt);

    /// 类型检查表达式
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式
    ///
    /// # 返回
    ///
    /// 表达式的类型
    fn check_expression(&mut self, expr: &nargo_ir::JsExpr) -> Type;

    /// 类型检查模板
    ///
    /// # 参数
    ///
    /// * `template` - 模板
    fn check_template(&mut self, template: &nargo_ir::TemplateIR);

    /// 类型检查模板节点
    ///
    /// # 参数
    ///
    /// * `node` - 模板节点
    fn check_template_node(&mut self, node: &nargo_ir::TemplateNodeIR);

    /// 类型检查元素
    ///
    /// # 参数
    ///
    /// * `element` - 元素
    fn check_element(&mut self, element: &nargo_ir::ElementIR);

    /// 类型检查属性
    ///
    /// # 参数
    ///
    /// * `attr` - 属性
    fn check_attribute(&mut self, attr: &nargo_ir::AttributeIR);

    /// 类型检查 if 节点
    ///
    /// # 参数
    ///
    /// * `if_node` - if 节点
    fn check_if_node(&mut self, if_node: &nargo_ir::IfNodeIR);

    /// 类型检查 for 节点
    ///
    /// # 参数
    ///
    /// * `for_node` - for 节点
    fn check_for_node(&mut self, for_node: &nargo_ir::ForNodeIR);

    /// 类型检查插值
    ///
    /// # 参数
    ///
    /// * `interpolation` - 插值
    fn check_interpolation(&mut self, interpolation: &nargo_ir::ExpressionIR);

    /// 类型兼容性检查
    ///
    /// # 参数
    ///
    /// * `actual` - 实际类型
    /// * `expected` - 预期类型
    ///
    /// # 返回
    ///
    /// 是否兼容
    fn is_type_compatible(&self, actual: &Type, expected: &Type) -> bool;

    /// 获取错误位置
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式
    ///
    /// # 返回
    ///
    /// 错误位置
    fn get_error_position(&self, expr: &nargo_ir::JsExpr) -> (usize, usize);

    /// 获取错误位置（行号和列号）
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式
    ///
    /// # 返回
    ///
    /// 错误位置（行号和列号）
    fn get_error_location(&self, expr: &nargo_ir::JsExpr) -> (Option<usize>, Option<usize>);
}
