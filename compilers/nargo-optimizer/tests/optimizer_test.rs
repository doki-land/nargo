use nargo_ir::{ElementIR, ExpressionIR, IRModule, JsExpr, TemplateIR, TemplateNodeIR, Trivia};
use nargo_optimizer::Optimizer;
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

#[test]
fn test_optimize_i18n() {
    let mut messages = HashMap::new();
    messages.insert("hello".to_string(), "你好".to_string());

    let mut i18n_data = HashMap::new();
    i18n_data.insert("zh-CN".to_string(), messages);

    let mut ir = IRModule { name: "Test".to_string(), metadata: HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Interpolation(ExpressionIR { code: "$t('hello')".to_string(), ast: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("$t".to_string(), Span::unknown(), Trivia::default())), args: vec![JsExpr::Literal(NargoValue::String("hello".to_string()), Span::unknown(), Trivia::default())], span: Span::unknown(), trivia: Trivia::default() }), is_static: false, span: Span::unknown(), trivia: Trivia::default() })], span: Span::unknown() }), hoisted_nodes: HashMap::new(), styles: vec![], i18n: Some(i18n_data), wasm: vec![], custom_blocks: vec![], tests: vec![], span: Span::unknown() };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, Some("zh-CN"), false);

    if let Some(template) = &ir.template {
        match &template.nodes[0] {
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(JsExpr::Literal(NargoValue::String(s), _, _)) = &expr.ast {
                    assert_eq!(s, "你好");
                }
                else {
                    panic!("Expected literal string in AST, got {:?}", expr.ast);
                }
            }
            TemplateNodeIR::Hoisted(id) => {
                let hoisted = ir.hoisted_nodes.get(id).expect("Hoisted node not found");
                if let TemplateNodeIR::Interpolation(expr) = hoisted {
                    if let Some(JsExpr::Literal(NargoValue::String(s), _, _)) = &expr.ast {
                        assert_eq!(s, "你好");
                    }
                    else {
                        panic!("Expected literal string in AST, got {:?}", expr.ast);
                    }
                }
                else {
                    panic!("Expected interpolation node in hoisted, got {:?}", hoisted);
                }
            }
            _ => panic!("Expected interpolation or hoisted node, got {:?}", template.nodes[0]),
        }
    }
    else {
        panic!("Expected template");
    }
}

#[test]
fn test_optimize_static_element() {
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_server: None,
        script_client: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "div".to_string(),
                attributes: vec![],
                children: vec![TemplateNodeIR::Text("Static".to_string(), Span::unknown(), Trivia::default())],
                is_static: false, // Initially false
                span: Span::unknown(),
                trivia: Trivia::default(),
            })],
            span: Span::unknown(),
        }),
        hoisted_nodes: HashMap::new(),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        tests: vec![],
        span: Span::unknown(),
    };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, None, false);

    if let Some(template) = &ir.template {
        // StaticHoistingPass might have hoisted it
        match &template.nodes[0] {
            TemplateNodeIR::Element(el) => {
                assert!(el.is_static, "Div with static text should be optimized to is_static: true");
            }
            TemplateNodeIR::Hoisted(id) => {
                let hoisted = ir.hoisted_nodes.get(id).expect("Hoisted node not found");
                if let TemplateNodeIR::Element(el) = hoisted {
                    assert!(el.is_static, "Hoisted div should be static");
                }
                else {
                    panic!("Hoisted node is not an element");
                }
            }
            _ => panic!("Expected element or hoisted node, got {:?}", template.nodes[0]),
        }
    }
    else {
        panic!("Expected template");
    }
}

#[test]
fn test_constant_folding() {
    let mut ir = IRModule { name: "Test".to_string(), metadata: HashMap::new(), script: Some(nargo_ir::JsProgram { body: vec![nargo_ir::JsStmt::VariableDecl { kind: "const".to_string(), id: "x".to_string(), init: Some(JsExpr::Binary { left: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::unknown(), Trivia::default())), op: "+".to_string(), right: Box::new(JsExpr::Literal(NargoValue::Number(2.0), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() }), span: Span::unknown(), trivia: Trivia::default() }], span: Span::unknown(), trivia: Trivia::default() }), script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::new(), styles: vec![], i18n: None, wasm: vec![], custom_blocks: vec![], tests: vec![], span: Span::unknown() };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, None, false);

    if let Some(script) = &ir.script {
        if let nargo_ir::JsStmt::VariableDecl { init: Some(JsExpr::Literal(NargoValue::Number(n), _, _)), .. } = &script.body[0] {
            assert_eq!(*n, 3.0);
        }
        else {
            panic!("Expected folded literal 3.0, got {:?}", script.body[0]);
        }
    }
}

#[test]
fn test_dead_code_elimination() {
    let mut ir = IRModule { name: "Test".to_string(), metadata: HashMap::new(), script: Some(nargo_ir::JsProgram { body: vec![nargo_ir::JsStmt::Return(Some(JsExpr::Literal(NargoValue::Number(1.0), Span::unknown(), Trivia::default())), Span::unknown(), Trivia::default()), nargo_ir::JsStmt::Expr(JsExpr::Identifier("unreachable".to_string(), Span::unknown(), Trivia::default()), Span::unknown(), Trivia::default())], span: Span::unknown(), trivia: Trivia::default() }), script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::new(), styles: vec![], i18n: None, wasm: vec![], custom_blocks: vec![], tests: vec![], span: Span::unknown() };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, None, false);

    if let Some(script) = &ir.script {
        assert_eq!(script.body.len(), 1, "Unreachable code should be removed");
    }
}

#[test]
fn test_reference_hoisting() {
    use nargo_ir::AttributeIR;
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: Some(nargo_ir::JsProgram { body: vec![nargo_ir::JsStmt::VariableDecl { kind: "const".to_string(), id: "STATIC_CLASS".to_string(), init: Some(JsExpr::Literal(NargoValue::String("container".to_string()), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() }], span: Span::unknown(), trivia: Trivia::default() }),
        script_server: None,
        script_client: None,
        script_meta: None,
        template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "class".to_string(), value: None, value_ast: Some(JsExpr::Identifier("STATIC_CLASS".to_string(), Span::unknown(), Trivia::default())), argument: None, modifiers: Vec::new(), is_directive: false, is_dynamic: true, span: Span::unknown(), trivia: Trivia::default() }], children: vec![], is_static: false, span: Span::unknown(), trivia: Trivia::default() })], span: Span::unknown() }),
        hoisted_nodes: HashMap::new(),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        tests: vec![],
        span: Span::unknown(),
    };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, None, false);

    if let Some(template) = &ir.template {
        match &template.nodes[0] {
            TemplateNodeIR::Hoisted(id) => {
                let hoisted = ir.hoisted_nodes.get(id).expect("Node should be hoisted");
                if let TemplateNodeIR::Element(el) = hoisted {
                    assert!(el.is_static, "Element with constant reference should be static");
                }
            }
            _ => panic!("Expected node to be hoisted, got {:?}", template.nodes[0]),
        }
    }
}
