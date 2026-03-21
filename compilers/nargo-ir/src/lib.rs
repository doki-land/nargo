#![warn(missing_docs)]

//! HXO IR 模块
//!
//! 提供 HXO 编译器的中间表示（IR）系统，支持 JavaScript 语言特性的表示、优化和验证。
//!
//! ## 主要功能
//! - 支持多种 JavaScript 表达式和语句的表示
//! - 提供访问者模式，方便遍历和处理 IR 结构
//! - 支持常量折叠、死代码消除等优化规则
//! - 提供类型检查、语义验证等验证功能
//! - 支持模块级别的清理和优化

// 模块声明
pub mod expr;
pub mod optimizer;
pub mod program;
pub mod stmt;
pub mod template;
pub mod types;
pub mod validator;
pub mod visitor;

// 重导出公共接口
pub use expr::{JsExpr, JsExprVisitor, TseAttribute};
pub use optimizer::{ExprOptimizer, IROptimizer, ProgramOptimizer, StmtOptimizer};
pub use program::{IRModule, JsProgram};
pub use stmt::{JsStmt, JsStmtVisitor};
pub use template::{AttributeIR, CustomBlockIR, ElementIR, ExpressionIR, ForIteratorIR, ForNodeIR, IfNodeIR, StyleIR, TemplateIR, TemplateNodeIR, TemplateNodeVisitor, TestIR};
pub use types::{Comment, IRError, MAX_ARRAY_LENGTH, MAX_OBJECT_SIZE, MAX_RECURSION_DEPTH, MAX_STRING_LENGTH, Trivia};
pub use validator::{ExprValidator, IRValidator, ProgramValidator, StmtValidator, TypeEnvironment, TypeInfo};
pub use visitor::DefaultVisitor;

#[cfg(test)]
mod tests {
    use super::*;
    use nargo_types::{NargoValue, Span};
    use std::collections::HashMap;

    #[test]
    fn test_ir_module_creation() {
        let module = IRModule::new("test".to_string());
        assert_eq!(module.name, "test");
        assert!(module.script.is_none());
        assert!(module.template.is_none());
    }

    #[test]
    fn test_ir_module_validation() {
        let module = IRModule::new("test".to_string());
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_ir_module_cleanup() {
        let mut module = IRModule::new("test".to_string());
        module.script = Some(JsProgram::default());
        module.script_meta = Some(NargoValue::Null);
        module.template = Some(TemplateIR::default());
        module.i18n = Some(HashMap::new());
        module.cleanup();
        assert!(module.script.is_none());
        assert!(module.script_meta.is_none());
        assert!(module.template.is_none());
        assert!(module.i18n.is_none());
    }

    #[test]
    fn test_ir_module_optimize() {
        let mut module = IRModule::new("test".to_string());
        module.optimize();
        // 优化后模块应该仍然有效
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_js_program_validation() {
        let program = JsProgram::default();
        assert!(program.validate().is_ok());
    }

    #[test]
    fn test_js_program_optimize() {
        let mut program = JsProgram::default();
        program.optimize();
        // 优化后程序应该仍然有效
        assert!(program.validate().is_ok());
    }

    #[test]
    fn test_js_program_is_empty() {
        let program = JsProgram::default();
        assert!(program.is_empty());
    }

    #[test]
    fn test_ir_module_fields() {
        let mut module = IRModule::new("test".to_string());

        // 测试设置和获取 script
        let script = JsProgram::default();
        module.script = Some(script.clone());
        assert!(module.script.is_some());

        // 测试设置和获取 template
        let template = TemplateIR::default();
        module.template = Some(template.clone());
        assert!(module.template.is_some());

        // 测试设置和获取 styles
        let style = StyleIR { code: "body { color: red; }".to_string(), lang: "css".to_string(), scoped: false, span: Span::default(), trivia: Trivia::default() };
        module.styles.push(style);
        assert!(!module.styles.is_empty());

        // 测试设置和获取 metadata
        module.metadata.insert("key".to_string(), NargoValue::String("value".to_string()));
        assert!(module.metadata.contains_key("key"));

        // 测试设置和获取 dependencies
        module.dependencies.push("dep1".to_string());
        assert!(module.dependencies.contains(&"dep1".to_string()));

        // 测试设置和获取 dependents
        module.dependents.push("dep1".to_string());
        assert!(module.dependents.contains(&"dep1".to_string()));
    }

    #[test]
    fn test_js_program_fields() {
        let mut program = JsProgram::default();

        // 测试设置和获取 body
        let stmt = JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::default(), Trivia::default()), Span::default(), Trivia::default());
        program.body.push(stmt);
        assert!(!program.body.is_empty());
    }

    #[test]
    fn test_js_program_not_empty() {
        let mut program = JsProgram::default();
        let stmt = JsStmt::Expr(JsExpr::Literal(NargoValue::Null, Span::default(), Trivia::default()), Span::default(), Trivia::default());
        program.body.push(stmt);
        assert!(!program.is_empty());
    }
}
