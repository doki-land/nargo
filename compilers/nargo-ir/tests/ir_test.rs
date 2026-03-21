use nargo_ir::*;
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

#[test]
fn test_nargo_file_structure() {
    use nargo_types::{NargoBlock, NargoFile};
    let mut attributes = HashMap::new();
    attributes.insert("lang".to_string(), "ts".to_string());

    let block = NargoBlock { name: "script".to_string(), attributes, content: "const x = 1;".to_string(), span: Default::default(), content_span: Default::default() };

    let file = NargoFile { blocks: vec![block], span: Default::default() };

    assert_eq!(file.blocks.len(), 1);
    assert_eq!(file.blocks[0].name, "script");
    assert_eq!(file.blocks[0].attributes.get("lang").unwrap(), "ts");
}

#[test]
fn test_ir_module_validation() {
    // 创建一个有效的 IR 模块
    let mut module = IRModule { name: "test".to_string(), metadata: HashMap::new(), script: None, script_server: None, script_client: None, script_meta: None, template: None, hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: None, wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), dependencies: Vec::new(), dependents: Vec::new(), span: Span::unknown() };

    // 验证模块有效性
    assert!(module.validate().is_ok());

    // 测试空字符串处理
    module.name = "".to_string();
    assert!(module.validate().is_ok());

    // 测试清理功能
    module.cleanup();
    assert_eq!(module.name, "");
}

#[test]
fn test_js_expr_validation() {
    // 测试标识符表达式
    let expr = JsExpr::Identifier("test".to_string(), Span::unknown(), Trivia::new());
    assert!(expr.validate(0).is_ok());

    // 测试字面量表达式
    let expr = JsExpr::Literal(NargoValue::String("test".to_string()), Span::unknown(), Trivia::new());
    assert!(expr.validate(0).is_ok());

    // 测试二元表达式
    let left = JsExpr::Identifier("a".to_string(), Span::unknown(), Trivia::new());
    let right = JsExpr::Identifier("b".to_string(), Span::unknown(), Trivia::new());
    let expr = JsExpr::Binary { left: Box::new(left), op: "+".to_string(), right: Box::new(right), span: Span::unknown(), trivia: Trivia::new() };
    assert!(expr.validate(0).is_ok());
}

#[test]
fn test_js_stmt_validation() {
    // 测试表达式语句
    let expr = JsExpr::Identifier("test".to_string(), Span::unknown(), Trivia::new());
    let stmt = JsStmt::Expr(expr, Span::unknown(), Trivia::new());
    assert!(stmt.validate(0).is_ok());

    // 测试变量声明语句
    let init = JsExpr::Literal(NargoValue::Number(42.0), Span::unknown(), Trivia::new());
    let stmt = JsStmt::VariableDecl { kind: "const".to_string(), id: "x".to_string(), init: Some(init), span: Span::unknown(), trivia: Trivia::new() };
    assert!(stmt.validate(0).is_ok());

    // 测试块语句
    let stmt = JsStmt::Block(Vec::new(), Span::unknown(), Trivia::new());
    assert!(stmt.validate(0).is_ok());
}

#[test]
fn test_template_node_validation() {
    // 测试文本节点
    let node = TemplateNodeIR::Text("test".to_string(), Span::unknown(), Trivia::new());
    assert!(node.validate(0).is_ok());

    // 测试注释节点
    let node = TemplateNodeIR::Comment("test".to_string(), Span::unknown(), Trivia::new());
    assert!(node.validate(0).is_ok());
}

#[test]
fn test_ir_module_cleanup() {
    // 创建一个包含空元素的 IR 模块
    let mut module = IRModule { name: "test".to_string(), metadata: HashMap::new(), script: Some(JsProgram { body: Vec::new(), span: Span::unknown(), trivia: Trivia::new() }), script_server: None, script_client: None, script_meta: Some(NargoValue::Null), template: Some(TemplateIR { nodes: Vec::new(), span: Span::unknown() }), hoisted_nodes: HashMap::new(), styles: Vec::new(), i18n: Some(HashMap::new()), wasm: Vec::new(), custom_blocks: Vec::new(), tests: Vec::new(), dependencies: Vec::new(), dependents: Vec::new(), span: Span::unknown() };

    // 清理模块
    module.cleanup();

    // 验证空元素被清理
    assert_eq!(module.script, None);
    assert_eq!(module.script_meta, None);
    assert_eq!(module.template, None);
    assert_eq!(module.i18n, None);
}

#[test]
fn test_is_empty_methods() {
    // 测试 Trivia::is_empty
    let trivia = Trivia::new();
    assert!(trivia.is_empty());

    // 测试 Comment::is_empty
    let comment = Comment::new("  ".to_string(), false, Span::unknown());
    assert!(comment.is_empty());

    // 测试 ExpressionIR::is_empty
    let expr = ExpressionIR::default();
    assert!(expr.is_empty());

    // 测试 StyleIR::is_empty
    let style = StyleIR::default();
    assert!(style.is_empty());

    // 测试 CustomBlockIR::is_empty
    let block = CustomBlockIR { name: "".to_string(), content: "".to_string(), attributes: HashMap::new(), span: Span::unknown(), trivia: Trivia::new() };
    assert!(block.is_empty());
}
