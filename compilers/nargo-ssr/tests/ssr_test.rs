use nargo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use nargo_ssr::SsrBackend;
use nargo_types::{CompileMode, Span};

#[test]
fn test_ssr_generation() {
    let mut backend = SsrBackend::new(CompileMode::Vue2);
    let ir = IRModule { name: "Test".to_string(), template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "class".to_string(), value: Some("container".to_string()), ..Default::default() }], children: vec![TemplateNodeIR::Text("Hello World".to_string(), Span::default(), Default::default())], is_static: true, ..Default::default() })], ..Default::default() }), ..Default::default() };

    let ssr_js = backend.generate(&ir).unwrap();
    assert!(ssr_js.contains("export function render(ctx)"));
    assert!(ssr_js.contains("html += '<div class=\"container\">';"));
    assert!(ssr_js.contains("html += 'Hello World';"));
    assert!(ssr_js.contains("html += '</div>';"));
}

#[test]
fn test_resumable_ssr_generation() {
    let mut backend = SsrBackend::new(CompileMode::Vue2).with_resumable(true);
    let ir = IRModule { name: "Counter".to_string(), template: Some(TemplateIR { nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "button".to_string(), attributes: vec![AttributeIR { name: "@click".to_string(), value: Some("increment".to_string()), is_directive: true, ..Default::default() }], children: vec![TemplateNodeIR::Text("Click me".to_string(), Span::default(), Default::default())], ..Default::default() })], ..Default::default() }), ..Default::default() };

    let ssr_js = backend.generate(&ir).unwrap();
    assert!(ssr_js.contains("html += `<script type=\"nargo/state\">${JSON.stringify(ctx)}</script>`;"));
    assert!(ssr_js.contains("on:click=\"/assets/test.js#increment\""));
}
