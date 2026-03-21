use nargo_hydrate::HydrateBackend;
use nargo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use nargo_types::{CompileMode, Span};
use std::collections::HashMap;

#[test]
fn test_hydrate_generation() {
    let backend = HydrateBackend::new(CompileMode::Vue2);
    let ir = IRModule { name: "Test".to_string(), template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "button".to_string(), attributes: vec![AttributeIR { name: "@click".to_string(), value: Some("handleClick".to_string()), is_directive: true, ..Default::default() }], children: vec![TemplateNodeIR::Text("Click me".to_string(), Span::default(), Default::default())], is_static: false, ..Default::default() })], ..Default::default() }), ..Default::default() };

    let hydrate_js = backend.generate(&ir).unwrap();
    assert!(hydrate_js.contains("export function hydrate(root, ctx, options = {})"));
    assert!(hydrate_js.contains("const nodes = new Map();"));
    assert!(hydrate_js.contains("const eventHandlers = new Map();"));
    assert!(hydrate_js.contains("const el0 = nodes.get('0');"));
    assert!(hydrate_js.contains("eventHandlers.get('click')['0'] = (e) => handleClick;"));
}

#[test]
fn test_hydrate_dynamic_attr_and_nested_index() {
    let backend = HydrateBackend::new(CompileMode::Vue2);
    let ir = IRModule { name: "Test".to_string(), metadata: HashMap::new(), script: None, script_meta: None, template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: ":class".to_string(), value: Some("activeClass".to_string()), value_ast: None, is_dynamic: true, is_directive: false, span: Span::default(), ..Default::default() }], children: vec![TemplateNodeIR::Text("Static".to_string(), Span::default(), Default::default()), TemplateNodeIR::Element(ElementIR { tag: "span".to_string(), attributes: vec![], children: vec![TemplateNodeIR::Interpolation(nargo_ir::ExpressionIR { code: "count".to_string(), ast: None, is_static: false, span: Span::default(), trivia: Default::default() })], is_static: false, span: Span::default(), trivia: Default::default() })], is_static: false, span: Span::default(), trivia: Default::default() })], span: Span::default() }), ..Default::default() };

    let hydrate_js = backend.generate(&ir).unwrap();

    // root div is el0
    assert!(hydrate_js.contains("const el0 = nodes.get('0');"));
    assert!(hydrate_js.contains("el0.setAttribute('class', activeClass);"));

    // Indexing:
    // 0: div (Element)
    // 1: "Static" (Text)
    // 2: span (Element)
    // 3: count (Interpolation)

    assert!(hydrate_js.contains("const el2 = nodes.get('2');"));
    assert!(hydrate_js.contains("const text3 = nodes.get('3');"));
    assert!(hydrate_js.contains("text3.textContent = count;"));
}
