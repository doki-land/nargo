use nargo_bundler::Bundler;
use nargo_ir::{ElementIR, ExpressionIR, IRModule, JsExpr, JsProgram, JsStmt, TemplateIR, TemplateNodeIR};
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

#[test]
fn test_bundler_basic() {
    let mut bundler = Bundler::new(None);
    let modules = vec![IRModule { name: "Test".to_string(), metadata: HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: Vec::new(), children: vec![TemplateNodeIR::Interpolation(ExpressionIR { code: "count".to_string(), ast: Some(JsExpr::Identifier("count".to_string(), Span::default(), Default::default())), is_static: false, span: Span::default(), trivia: Default::default() })], is_static: false, span: Span::default(), trivia: Default::default() })], span: Span::default() }), hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: None, span: Span::default() }];

    let bundled = bundler.bundle_all_default(&modules).unwrap();

    // Check if it contains the runtime and the component
    let output_code = String::from_utf8_lossy(&bundled.outputs[0].code);
    assert!(output_code.contains("Nargo Custom Runtime"));
    assert!(output_code.contains("Component: Test"));
    // Since it has interpolation, it should have effects
    assert!(output_code.contains("createEffect"));
    // Since it has template, it should have VDOM
    assert!(output_code.contains("function h("));
}

#[test]
fn test_bundler_with_reactivity() {
    let mut bundler = Bundler::new(None);
    let modules = vec![IRModule { name: "Counter".to_string(), metadata: HashMap::new(), script: Some(JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "count".to_string(), init: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("createSignal".to_string(), Span::default(), Default::default())), args: vec![JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Default::default())], span: Span::default(), trivia: Default::default() }), span: Span::default(), trivia: Default::default() }], span: Span::default(), trivia: Default::default() }), script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: None, span: Span::default() }];

    let bundled = bundler.bundle_all_default(&modules).unwrap();

    let output_code = String::from_utf8_lossy(&bundled.outputs[0].code);
    assert!(output_code.contains("createSignal"));
}
