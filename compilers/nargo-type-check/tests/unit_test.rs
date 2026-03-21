use nargo_ir::{IRModule, JsExpr, JsProgram, JsStmt};
use nargo_type_check::{TypeCheckOptions, TypeChecker};
use nargo_types::{NargoValue, Span};
#[test]
fn test_type_checker() {
    let mut module = IRModule::default();
    module.name = "test".to_string();
    module.script = Some(JsProgram { body: vec![JsStmt::Expr(JsExpr::Literal(NargoValue::String("test".to_string()), Span::default(), Default::default()), Span::default(), Default::default())], span: Span::default(), trivia: Default::default() });
    let options = TypeCheckOptions::default();
    let checker = TypeChecker::new();
    let result = checker.check_module(&module, &options);
    assert!(result.errors == 0);
}
