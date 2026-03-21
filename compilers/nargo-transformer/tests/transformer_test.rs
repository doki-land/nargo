use nargo_ir::{ElementIR, IRModule, JsExpr, JsStmt, StyleIR, TemplateIR, TemplateNodeIR, Trivia};
use nargo_transformer::{I18nPass, ScopedCssPass, StaticHoistingPass, TransformPass, Transformer};
use nargo_types::{NargoValue, Result, Span};
use std::collections::HashMap;

struct MockPass;

impl TransformPass for MockPass {
    fn name(&self) -> String {
        "MockPass".to_string()
    }

    fn description(&self) -> String {
        "A mock transformation pass for testing".to_string()
    }

    fn transform(&mut self, _ir: &mut IRModule) -> Result<()> {
        // Do nothing
        Ok(())
    }
}

#[test]
fn test_transformer_logging() {
    let mut transformer = Transformer::new();
    let mut ir = IRModule { name: "test".to_string(), metadata: std::collections::HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: std::collections::HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), span: nargo_types::Span::default() };
    let mut pass = MockPass;

    transformer.apply(&mut ir, &mut pass).unwrap();

    let logs = transformer.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].name, "MockPass");
    assert_eq!(logs[0].description, "A mock transformation pass for testing");
}

#[test]
fn test_manual_logging() {
    let mut transformer = Transformer::new();
    transformer.log("Manual", "Manual transformation", None, None, None);

    let logs = transformer.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].name, "Manual");
}

#[test]
fn test_scoped_css_pass() {
    let mut ir = IRModule { name: "test".to_string(), metadata: std::collections::HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: Vec::new(), children: Vec::new(), is_static: false, span: nargo_types::Span::default(), trivia: nargo_ir::Trivia::default() })], span: nargo_types::Span::default() }), hoisted_nodes: std::collections::HashMap::new(), styles: vec![StyleIR { lang: "css".to_string(), code: ".test { color: red; }".to_string(), scoped: true, span: nargo_types::Span::default(), trivia: nargo_ir::Trivia::default() }], i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), span: nargo_types::Span::default() };

    let mut transformer = Transformer::new();
    let mut pass = ScopedCssPass::new("data-h-123".to_string());

    transformer.apply(&mut ir, &mut pass).unwrap();

    let logs = transformer.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].name, "ScopedCssPass");
}

#[test]
fn test_ir_snapshots() {
    let mut ir = IRModule { name: "test".to_string(), metadata: std::collections::HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: Vec::new(), children: Vec::new(), is_static: false, span: nargo_types::Span::default(), trivia: nargo_ir::Trivia::default() })], span: nargo_types::Span::default() }), hoisted_nodes: std::collections::HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), span: nargo_types::Span::default() };

    let mut transformer = Transformer::new();
    transformer.enable_snapshots = true;
    let mut pass = ScopedCssPass::new("data-h-123".to_string());

    transformer.apply(&mut ir, &mut pass).unwrap();

    let logs = transformer.get_logs();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].ir_before.is_some());
    assert!(logs[0].ir_after.is_some());
    assert_ne!(logs[0].ir_before, logs[0].ir_after);

    let ir_before: IRModule = serde_json::from_str(logs[0].ir_before.as_ref().unwrap()).unwrap();
    let ir_after: IRModule = serde_json::from_str(logs[0].ir_after.as_ref().unwrap()).unwrap();

    assert_eq!(ir_before.name, "test");
    assert_eq!(ir_after.name, "test");
}

#[test]
fn test_i18n_pass() {
    let mut messages = HashMap::new();
    messages.insert("hello".to_string(), "你好".to_string());

    let mut ir = IRModule { name: "test".to_string(), metadata: HashMap::new(), script: Some(nargo_ir::JsProgram { body: vec![JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Identifier("$t".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::Literal(NargoValue::String("hello".to_string()), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }), script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), span: Span::default() };

    let mut transformer = Transformer::new();
    let mut pass = I18nPass::new(messages);

    transformer.apply(&mut ir, &mut pass).unwrap();

    if let Some(script) = &ir.script {
        if let JsStmt::Expr(expr, _, _) = &script.body[0] {
            if let JsExpr::Literal(NargoValue::String(s), _, _) = expr {
                assert_eq!(s, "你好");
            }
            else {
                panic!("Expected literal expression");
            }
        }
    }
}

#[test]
fn test_static_hoisting_pass() {
    let mut ir = IRModule { name: "test".to_string(), metadata: HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: Vec::new(), children: vec![TemplateNodeIR::Text("Static text".to_string(), Span::default(), Trivia::default())], is_static: false, span: Span::default(), trivia: Trivia::default() })], span: Span::default() }), hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), span: Span::default() };

    let mut transformer = Transformer::new();
    let mut pass = StaticHoistingPass::new();

    transformer.apply(&mut ir, &mut pass).unwrap();

    if let Some(template) = &ir.template {
        match &template.nodes[0] {
            TemplateNodeIR::Hoisted(id) => {
                assert!(ir.hoisted_nodes.contains_key(id));
                if let Some(TemplateNodeIR::Element(el)) = ir.hoisted_nodes.get(id) {
                    assert_eq!(el.tag, "div");
                    assert!(el.is_static);
                }
                else {
                    panic!("Expected hoisted element");
                }
            }
            _ => panic!("Expected hoisted node reference in template"),
        }
    }
}

#[test]
fn test_source_map_propagation() {
    use nargo_types::Position;

    let test_span = Span { start: Position { line: 10, column: 5, offset: 100 }, end: Position { line: 10, column: 20, offset: 115 } };

    let mut messages = HashMap::new();
    messages.insert("hello".to_string(), "你好".to_string());

    let mut ir = IRModule {
        name: "test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_server: None,
        script_client: None,
        script_meta: None,
        template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![nargo_ir::AttributeIR { name: "title".to_string(), value: None, value_ast: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("$t".to_string(), test_span, Trivia::default())), args: vec![JsExpr::Literal(NargoValue::String("hello".to_string()), test_span, Trivia::default())], span: test_span, trivia: Trivia::default() }), argument: None, modifiers: Vec::new(), is_directive: true, is_dynamic: true, span: test_span, trivia: Trivia::default() }], children: vec![TemplateNodeIR::Text("static".to_string(), test_span, Trivia::default())], is_static: false, span: test_span, trivia: Trivia::default() })], span: test_span }),
        hoisted_nodes: HashMap::new(),
        styles: Vec::new(),
        i18n: None,
        wasm: Vec::new(),
        custom_blocks: Vec::new(),
        tests: Vec::new(),
        span: test_span,
    };

    let mut transformer = Transformer::new();

    // 1. Scoped CSS Pass
    let mut scoped_pass = ScopedCssPass::new("data-h-123".to_string());
    transformer.apply(&mut ir, &mut scoped_pass).unwrap();

    // 2. i18n Pass
    let mut i18n_pass = I18nPass::new(messages);
    transformer.apply(&mut ir, &mut i18n_pass).unwrap();

    // 3. Hoisting Pass
    // 注意：这里的 div 含有动态属性 (i18n 替换后变为了静态，但 hoisting 前需要重新计算 is_static)
    // 我们的 I18nPass 目前只替换 AST，不自动更新 is_dynamic/is_static 标志
    // 所以我们需要先应用 I18nPass，然后再应用 HoistingPass
    let mut hoisting_pass = StaticHoistingPass::new();
    transformer.apply(&mut ir, &mut hoisting_pass).unwrap();

    // 验证：div 节点应该被提升，且其 Span 保持为 test_span
    if let Some(template) = &ir.template {
        match &template.nodes[0] {
            TemplateNodeIR::Hoisted(id) => {
                let hoisted_node = ir.hoisted_nodes.get(id).unwrap();
                if let TemplateNodeIR::Element(el) = hoisted_node {
                    assert_eq!(el.span, test_span);
                    // 验证 i18n 替换后的字面量也保持了 Span
                    if let Some(JsExpr::Literal(_, span, _)) = &el.attributes[0].value_ast {
                        assert_eq!(*span, test_span);
                    }
                }
            }
            _ => panic!("Expected hoisted node"),
        }
    }
}

#[test]
fn test_complex_static_hoisting() {
    let mut ir = IRModule {
        name: "test".to_string(),
        metadata: HashMap::new(),
        script: Some(nargo_ir::JsProgram {
            body: vec![
                // const CONFIG = { theme: 'dark' };
                JsStmt::VariableDecl {
                    kind: "const".to_string(),
                    id: "CONFIG".to_string(),
                    init: Some(JsExpr::Object(
                        {
                            let mut m = HashMap::new();
                            m.insert("theme".to_string(), JsExpr::Literal(NargoValue::String("dark".to_string()), Span::default(), Trivia::default()));
                            m
                        },
                        Span::default(),
                        Trivia::default(),
                    )),
                    span: Span::default(),
                    trivia: Trivia::default(),
                },
                // const THEME = CONFIG.theme;
                JsStmt::VariableDecl { kind: "const".to_string(), id: "THEME".to_string(), init: Some(JsExpr::Member { object: Box::new(JsExpr::Identifier("CONFIG".to_string(), Span::default(), Trivia::default())), property: Box::new(JsExpr::Literal(NargoValue::String("theme".to_string()), Span::default(), Trivia::default())), computed: false, span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() },
            ],
            span: Span::default(),
            trivia: Trivia::default(),
        }),
        script_server: None,
        script_client: None,
        script_meta: None,
        template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![nargo_ir::AttributeIR { name: "class".to_string(), value: None, value_ast: Some(JsExpr::Identifier("THEME".to_string(), Span::default(), Trivia::default())), argument: None, modifiers: Vec::new(), is_directive: true, is_dynamic: true, span: Span::default(), trivia: Trivia::default() }], children: vec![TemplateNodeIR::Text("Content".to_string(), Span::default(), Trivia::default())], is_static: false, span: Span::default(), trivia: Trivia::default() })], span: Span::default() }),
        hoisted_nodes: HashMap::new(),
        styles: Vec::new(),
        i18n: None,
        wasm: Vec::new(),
        custom_blocks: Vec::new(),
        tests: Vec::new(),
        span: Span::default(),
    };

    let mut transformer = Transformer::new();
    let mut pass = StaticHoistingPass::new();

    transformer.apply(&mut ir, &mut pass).unwrap();

    // Verify that the div node is hoisted because THEME is static
    if let Some(template) = &ir.template {
        match &template.nodes[0] {
            TemplateNodeIR::Hoisted(id) => {
                assert!(ir.hoisted_nodes.contains_key(id));
                let hoisted_node = ir.hoisted_nodes.get(id).unwrap();
                if let TemplateNodeIR::Element(el) = hoisted_node {
                    assert_eq!(el.tag, "div");
                    assert!(el.is_static);
                }
            }
            _ => panic!("Expected hoisted node for complex static expression"),
        }
    }
}
