use nargo_hydrate::HydrateBackend;
use nargo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use nargo_types::{CompileMode, Span};
use std::time::Instant;

#[test]
fn test_hydrate_performance() {
    // Create a complex IR with many dynamic elements and interpolations
    let mut nodes = Vec::new();

    // Add 1000 dynamic elements
    for i in 0..1000 {
        nodes.push(TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: ":class".to_string(), value: Some(format!("dynamicClass + {}", i)), is_dynamic: true, is_directive: false, span: Span::default(), ..Default::default() }, AttributeIR { name: "@click".to_string(), value: Some(format!("handleClick({})")), is_directive: true, span: Span::default(), ..Default::default() }], children: vec![TemplateNodeIR::Interpolation(nargo_ir::ExpressionIR { code: format!("dynamicValue + {}", i), is_static: false, span: Span::default(), ..Default::default() })], is_static: false, span: Span::default(), ..Default::default() }));
    }

    let ir = IRModule { name: "PerformanceTest".to_string(), template: Some(TemplateIR { nodes, span: Span::default(), ..Default::default() }), ..Default::default() };

    let backend = HydrateBackend::new(CompileMode::Vue2);

    // Measure generation time
    let start = Instant::now();
    let hydrate_js = backend.generate(&ir).unwrap();
    let duration = start.elapsed();

    println!("Hydration code generation time: {:?}", duration);
    println!("Generated code length: {} bytes", hydrate_js.len());

    // Verify the generated code includes our optimizations
    assert!(hydrate_js.contains("eventHandlers"));
    assert!(hydrate_js.contains("partialHydrate"));
    assert!(hydrate_js.contains("if (currentValue !== last"));
}

#[test]
fn test_incremental_hydration() {
    let backend = HydrateBackend::new(CompileMode::Vue2);
    let ir = IRModule {
        name: "IncrementalTest".to_string(),
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "data-nargo-region".to_string(), value: Some("header".to_string()), is_dynamic: false, is_directive: false, span: Span::default(), ..Default::default() }], children: vec![TemplateNodeIR::Interpolation(nargo_ir::ExpressionIR { code: "headerTitle".to_string(), is_static: false, span: Span::default(), ..Default::default() })], is_static: false, span: Span::default(), ..Default::default() }), TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "data-nargo-region".to_string(), value: Some("content".to_string()), is_dynamic: false, is_directive: false, span: Span::default(), ..Default::default() }], children: vec![TemplateNodeIR::Interpolation(nargo_ir::ExpressionIR { code: "contentText".to_string(), is_static: false, span: Span::default(), ..Default::default() })], is_static: false, span: Span::default(), ..Default::default() })],
            span: Span::default(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let hydrate_js = backend.generate(&ir).unwrap();

    // Verify incremental hydration support
    assert!(hydrate_js.contains("partialHydrate"));
    assert!(hydrate_js.contains("options.region"));
    assert!(hydrate_js.contains("data-nargo-region"));
}
