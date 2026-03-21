use nargo_coverage::CoverageInstrumenter;
use nargo_ir::{IRModule, JsExpr, JsProgram, JsStmt, Trivia};
use nargo_types::Span;
use std::collections::HashMap;

#[test]
fn test_basic_instrumentation() {
    let mut module = IRModule { name: "test.nargo".to_string(), metadata: HashMap::new(), script: Some(JsProgram { body: vec![JsStmt::FunctionDecl { id: "add".to_string(), params: vec!["a".to_string(), "b".to_string()], body: vec![JsStmt::Expr(JsExpr::Binary { left: Box::new(JsExpr::Identifier("a".to_string(), Span::default(), Trivia::default())), op: "+".to_string(), right: Box::new(JsExpr::Identifier("b".to_string(), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }], ..Default::default() }), ..Default::default() };

    let mut instrumenter = CoverageInstrumenter::new("file1".to_string(), "test.nargo".to_string());
    let metadata = instrumenter.instrument(&mut module).unwrap();

    assert!(metadata.statement_map.len() > 0);
    assert!(metadata.function_map.len() > 0);

    let program = module.script.unwrap();
    // 1 original function decl -> 1 counter + 1 function decl
    assert_eq!(program.body.len(), 2);

    if let JsStmt::FunctionDecl { body, .. } = &program.body[1] {
        // 1 original expr -> 1 counter + 1 expr
        assert_eq!(body.len(), 2);
        assert!(matches!(body[0], JsStmt::Other(_, _, _)));
    }
    else {
        panic!("Expected FunctionDecl");
    }
}

#[test]
fn test_export_instrumentation() {
    let mut module = IRModule { name: "test_export.nargo".to_string(), metadata: HashMap::new(), script: Some(JsProgram { body: vec![JsStmt::Export { declaration: Box::new(JsStmt::FunctionDecl { id: "sub".to_string(), params: vec!["a".to_string(), "b".to_string()], body: vec![JsStmt::Expr(JsExpr::Identifier("a".to_string(), Span::default(), Trivia::default()), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() }], ..Default::default() }), ..Default::default() };

    let mut instrumenter = CoverageInstrumenter::new("file2".to_string(), "test_export.nargo".to_string());
    instrumenter.instrument(&mut module).unwrap();

    let program = module.script.unwrap();
    // 1 counter + 1 export
    assert_eq!(program.body.len(), 2);

    if let JsStmt::Export { declaration, .. } = &program.body[1] {
        if let JsStmt::FunctionDecl { body, .. } = declaration.as_ref() {
            // 1 counter + 1 expr
            assert_eq!(body.len(), 2);
        }
        else {
            panic!("Expected FunctionDecl in Export");
        }
    }
    else {
        panic!("Expected Export");
    }
}

#[test]
fn test_conditional_instrumentation() {
    let mut module = IRModule { name: "test_cond.nargo".to_string(), metadata: HashMap::new(), script: Some(JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "x".to_string(), init: Some(JsExpr::Conditional { test: Box::new(JsExpr::Identifier("a".to_string(), Span::default(), Trivia::default())), consequent: Box::new(JsExpr::Literal(nargo_types::NargoValue::Number(1.0), Span::default(), Trivia::default())), alternate: Box::new(JsExpr::Literal(nargo_types::NargoValue::Number(2.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() }], ..Default::default() }), ..Default::default() };

    let mut instrumenter = CoverageInstrumenter::new("file3".to_string(), "test_cond.nargo".to_string());
    let metadata = instrumenter.instrument(&mut module).unwrap();

    assert_eq!(metadata.branch_map.len(), 1);

    let program = module.script.unwrap();
    if let JsStmt::VariableDecl { init: Some(JsExpr::Conditional { test, .. }), .. } = &program.body[1] {
        // Test should be wrapped in Binary (comma operator)
        assert!(matches!(test.as_ref(), JsExpr::Binary { op, .. } if op == ","));
    }
    else {
        panic!("Expected Conditional in VariableDecl");
    }
}

#[test]
fn test_template_instrumentation() {
    use nargo_ir::{AttributeIR, ElementIR, TemplateIR, TemplateNodeIR};

    let mut module =
        IRModule { name: "test_template.nargo".to_string(), metadata: HashMap::new(), script: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "class".to_string(), value: None, value_ast: Some(JsExpr::Conditional { test: Box::new(JsExpr::Identifier("active".to_string(), Span::default(), Trivia::default())), consequent: Box::new(JsExpr::Literal(nargo_types::NargoValue::String("a".to_string()), Span::default(), Trivia::default())), alternate: Box::new(JsExpr::Literal(nargo_types::NargoValue::String("b".to_string()), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }), is_directive: false, is_dynamic: true, span: Span::default(), trivia: Trivia::default() }], children: vec![], is_static: false, span: Span::default(), trivia: Trivia::default() })], span: Span::default() }), styles: vec![], i18n: None, wasm: vec![], custom_blocks: vec![], span: Span::default() };

    let mut instrumenter = CoverageInstrumenter::new("file4".to_string(), "test_template.nargo".to_string());
    let metadata = instrumenter.instrument(&mut module).unwrap();

    assert_eq!(metadata.branch_map.len(), 1);

    let template = module.template.unwrap();
    if let TemplateNodeIR::Element(el) = &template.nodes[0] {
        if let Some(JsExpr::Conditional { test, .. }) = &el.attributes[0].value_ast {
            assert!(matches!(test.as_ref(), JsExpr::Binary { op, .. } if op == ","));
        }
        else {
            panic!("Expected Conditional in template attribute");
        }
    }
}
