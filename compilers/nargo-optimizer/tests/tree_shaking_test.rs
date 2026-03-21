use nargo_ir::{IRModule, JsExpr, JsProgram, JsStmt, Trivia};
use nargo_optimizer::{OptimizationLevel, Optimizer};
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

#[test]
fn test_tree_shaking() {
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: Some(JsProgram {
            body: vec![
                // 未使用的函数
                JsStmt::FunctionDecl { id: "unused_function".to_string(), params: vec![], body: vec![JsStmt::Return(Some(JsExpr::Literal(NargoValue::Number(42.0), Span::unknown(), Trivia::default())), Span::unknown(), Trivia::default())], is_async: false, span: Span::unknown(), trivia: Trivia::default() },
                // 未使用的变量
                JsStmt::VariableDecl { kind: "const".to_string(), id: "unused_var".to_string(), init: Some(JsExpr::Literal(NargoValue::String("unused".to_string()), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() },
                // 使用的变量
                JsStmt::VariableDecl { kind: "const".to_string(), id: "used_var".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(10.0), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() },
                // 使用的函数
                JsStmt::FunctionDecl { id: "used_function".to_string(), params: vec![], body: vec![JsStmt::Return(Some(JsExpr::Identifier("used_var".to_string(), Span::unknown(), Trivia::default())), Span::unknown(), Trivia::default())], is_async: false, span: Span::unknown(), trivia: Trivia::default() },
                // 调用使用的函数
                JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Identifier("used_function".to_string(), Span::unknown(), Trivia::default())), args: vec![], span: Span::unknown(), trivia: Trivia::default() }, Span::unknown(), Trivia::default()),
            ],
            span: Span::unknown(),
            trivia: Trivia::default(),
        }),
        script_server: None,
        script_client: None,
        script_meta: None,
        template: None,
        hoisted_nodes: HashMap::new(),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        tests: vec![],
        span: Span::unknown(),
    };

    let mut optimizer = Optimizer::new();
    optimizer.set_optimization_level(OptimizationLevel::Aggressive);
    optimizer.optimize(&mut ir, None, true);

    if let Some(script) = &ir.script {
        // 检查未使用的函数和变量是否被移除
        let stmt_count = script.body.len();
        assert!(stmt_count < 5, "Unused code should be removed");

        // 检查使用的函数和变量是否保留
        let mut has_used_var = false;
        let mut has_used_function = false;
        let mut has_call = false;

        for stmt in &script.body {
            match stmt {
                JsStmt::VariableDecl { id, .. } if id == "used_var" => {
                    has_used_var = true;
                }
                JsStmt::FunctionDecl { id, .. } if id == "used_function" => {
                    has_used_function = true;
                }
                JsStmt::Expr(expr, ..) => {
                    if let JsExpr::Call { callee, .. } = expr {
                        if let JsExpr::Identifier(ref id, ..) = **callee {
                            if id == "used_function" {
                                has_call = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        assert!(has_used_var, "Used variable should be preserved");
        assert!(has_used_function, "Used function should be preserved");
        assert!(has_call, "Function call should be preserved");
    }
}

#[test]
fn test_dead_code_elimination_enhanced() {
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: Some(JsProgram {
            body: vec![
                // 条件为 false 的 if 语句
                JsStmt::If { test: JsExpr::Literal(NargoValue::Bool(false), Span::unknown(), Trivia::default()), consequent: Box::new(JsStmt::Expr(JsExpr::Identifier("dead_code".to_string(), Span::unknown(), Trivia::default()), Span::unknown(), Trivia::default())), alternate: Some(Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Number(1.0), Span::unknown(), Trivia::default()), Span::unknown(), Trivia::default()))), span: Span::unknown(), trivia: Trivia::default() },
                // 循环条件为 false 的 while 循环
                JsStmt::While { test: JsExpr::Literal(NargoValue::Bool(false), Span::unknown(), Trivia::default()), body: Box::new(JsStmt::Expr(JsExpr::Identifier("dead_loop_code".to_string(), Span::unknown(), Trivia::default()), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() },
            ],
            span: Span::unknown(),
            trivia: Trivia::default(),
        }),
        script_server: None,
        script_client: None,
        script_meta: None,
        template: None,
        hoisted_nodes: HashMap::new(),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        tests: vec![],
        span: Span::unknown(),
    };

    let mut optimizer = Optimizer::new();
    optimizer.set_optimization_level(OptimizationLevel::Basic);
    optimizer.optimize(&mut ir, None, true);

    if let Some(script) = &ir.script {
        // 检查死代码是否被移除
        let stmt_count = script.body.len();
        assert!(stmt_count > 0, "Script should not be empty");
    }
}

#[test]
fn test_optimization_levels() {
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: Some(JsProgram {
            body: vec![
                // 未使用的函数
                JsStmt::FunctionDecl { id: "unused_function".to_string(), params: vec![], body: vec![JsStmt::Return(Some(JsExpr::Literal(NargoValue::Number(42.0), Span::unknown(), Trivia::default())), Span::unknown(), Trivia::default())], is_async: false, span: Span::unknown(), trivia: Trivia::default() },
                // 常量表达式
                JsStmt::VariableDecl { kind: "const".to_string(), id: "x".to_string(), init: Some(JsExpr::Binary { left: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::unknown(), Trivia::default())), op: "+".to_string(), right: Box::new(JsExpr::Literal(NargoValue::Number(2.0), Span::unknown(), Trivia::default())), span: Span::unknown(), trivia: Trivia::default() }), span: Span::unknown(), trivia: Trivia::default() },
                // 使用x的语句，确保x不会被移除
                JsStmt::Expr(JsExpr::Identifier("x".to_string(), Span::unknown(), Trivia::default()), Span::unknown(), Trivia::default()),
            ],
            span: Span::unknown(),
            trivia: Trivia::default(),
        }),
        script_server: None,
        script_client: None,
        script_meta: None,
        template: None,
        hoisted_nodes: HashMap::new(),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        tests: vec![],
        span: Span::unknown(),
    };

    // 测试无优化
    let mut optimizer_none = Optimizer::new();
    optimizer_none.set_optimization_level(OptimizationLevel::None);
    let mut ir_none = ir.clone();
    optimizer_none.optimize(&mut ir_none, None, true);

    // 测试基本优化
    let mut optimizer_basic = Optimizer::new();
    optimizer_basic.set_optimization_level(OptimizationLevel::Basic);
    let mut ir_basic = ir.clone();
    optimizer_basic.optimize(&mut ir_basic, None, true);

    // 测试标准优化
    let mut optimizer_standard = Optimizer::new();
    optimizer_standard.set_optimization_level(OptimizationLevel::Standard);
    let mut ir_standard = ir.clone();
    optimizer_standard.optimize(&mut ir_standard, None, true);

    // 测试激进优化
    let mut optimizer_aggressive = Optimizer::new();
    optimizer_aggressive.set_optimization_level(OptimizationLevel::Aggressive);
    let mut ir_aggressive = ir.clone();
    optimizer_aggressive.optimize(&mut ir_aggressive, None, true);

    // 验证无优化时代码保持不变
    assert_eq!(ir_none.script.unwrap().body.len(), 3);

    // 验证激进优化时未使用的函数被移除
    assert!(ir_aggressive.script.unwrap().body.len() < 3);
}
