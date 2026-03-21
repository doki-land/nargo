use nargo_ir::{AttributeIR, ElementIR, IRModule, JsExpr, JsProgram, JsStmt, StyleIR, TemplateIR, TemplateNodeIR};
use nargo_types::{NargoValue, Position, Result, Span};
#[test]
fn test_ir_module_validation() {
    let mut module = IRModule::default();
    module.name = "test".to_string();
    let result = module.validate();
    assert!(result.is_ok());
}
#[test]
fn test_ir_module_cleanup() {
    let mut module = IRModule::default();
    module.name = "test".to_string();
    module.script = Some(JsProgram { body: vec![], span: Span::default(), trivia: Default::default() });
    module.cleanup();
    assert!(module.script.is_none());
}
#[test]
fn test_js_program_validation() {
    let program = JsProgram { body: vec![], span: Span::default(), trivia: Default::default() };
    let result = program.validate();
    assert!(result.is_ok());
}
#[test]
fn test_js_program_optimization() {
    let mut program = JsProgram { body: vec![], span: Span::default(), trivia: Default::default() };
    program.optimize();
    assert!(program.body.is_empty());
}
#[test]
fn test_js_program_is_empty() {
    let program = JsProgram { body: vec![], span: Span::default(), trivia: Default::default() };
    assert!(program.is_empty());
    let program_with_body = JsProgram { body: vec![JsStmt::Expr(JsExpr::Literal(NargoValue::String("test".to_string()), Span::default(), Default::default()), Span::default(), Default::default())], span: Span::default(), trivia: Default::default() };
    assert!(!program_with_body.is_empty());
}
