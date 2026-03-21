use nargo_ir::{JsExpr, JsProgram, JsStmt, Trivia};
use nargo_script_analyzer::{IssueLevel, ScriptAnalyzer};
use nargo_types::{NargoValue, Span};
use std::collections::HashMap;

#[test]
fn test_analyze_signals() {
    let program = JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "[count, setCount]".to_string(), init: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("createSignal".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() }, JsStmt::VariableDecl { kind: "const".to_string(), id: "doubleCount".to_string(), init: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("createComputed".to_string(), Span::default(), Trivia::default())), args: vec![], span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() }], span: Span::default(), trivia: Trivia::default() };

    let analyzer = ScriptAnalyzer::new();
    let meta = analyzer.analyze(&program).unwrap();

    assert!(meta.signals.contains("count"));
    assert!(meta.computed.contains("doubleCount"));
    assert_eq!(meta.signals.len(), 1);
    assert_eq!(meta.computed.len(), 1);
}

#[test]
fn test_analyze_props_and_emits() {
    let mut props_map = HashMap::new();
    props_map.insert("msg".to_string(), JsExpr::Identifier("String".to_string(), Span::default(), Trivia::default()));
    props_map.insert("count".to_string(), JsExpr::Identifier("Number".to_string(), Span::default(), Trivia::default()));

    let program = JsProgram {
        body: vec![
            // const props = defineProps({ msg: String, count: Number })
            JsStmt::VariableDecl { kind: "const".to_string(), id: "props".to_string(), init: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("defineProps".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::Object(props_map, Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() },
            // defineEmits(['update', 'delete'])
            JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Identifier("defineEmits".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::Array(vec![JsExpr::Literal(NargoValue::String("update".to_string()), Span::default(), Trivia::default()), JsExpr::Literal(NargoValue::String("delete".to_string()), Span::default(), Trivia::default())], Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default()),
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let meta = analyzer.analyze(&program).unwrap();

    assert!(meta.props.contains("msg"));
    assert!(meta.props.contains("count"));
    assert_eq!(meta.props.len(), 2);

    assert!(meta.emits.contains("update"));
    assert!(meta.emits.contains("delete"));
    assert_eq!(meta.emits.len(), 2);

    let nargo_val = meta.to_nargo_value();
    if let NargoValue::Object(map) = nargo_val {
        assert!(map.contains_key("props"));
        assert!(map.contains_key("emits"));
        assert!(map.contains_key("signals"));
    }
    else {
        panic!("Should be an object");
    }
}

#[test]
fn test_analyze_with_rules_unused_variable() {
    let program = JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "unusedVar".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }, JsStmt::VariableDecl { kind: "const".to_string(), id: "usedVar".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(20.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }, JsStmt::Expr(JsExpr::Binary { left: Box::new(JsExpr::Identifier("usedVar".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(5.0), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到未使用的变量
    let unused_var_issues = report.issues.iter().filter(|i| i.code == "unused-variable" && i.message == "Unused variable: unusedVar").count();
    assert_eq!(unused_var_issues, 1);

    // 应该没有检测到 usedVar 是未使用的
    let used_var_issues = report.issues.iter().filter(|i| i.code == "unused-variable" && i.message == "Unused variable: usedVar").count();
    assert_eq!(used_var_issues, 0);
}

#[test]
fn test_analyze_with_rules_undefined_variable() {
    let program = JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "definedVar".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }, JsStmt::Expr(JsExpr::Binary { left: Box::new(JsExpr::Identifier("definedVar".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Identifier("undefinedVar".to_string(), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到未定义的变量
    let undefined_var_issues = report.issues.iter().filter(|i| i.code == "undefined-variable" && i.message == "Undefined variable: undefinedVar").count();
    assert_eq!(undefined_var_issues, 1);

    // 应该没有检测到 definedVar 是未定义的
    let defined_var_issues = report.issues.iter().filter(|i| i.code == "undefined-variable" && i.message == "Undefined variable: definedVar").count();
    assert_eq!(defined_var_issues, 0);
}

#[test]
fn test_analyze_with_rules_unsafe_operation() {
    let program = JsProgram {
        body: vec![
            // 除以零
            JsStmt::VariableDecl { kind: "const".to_string(), id: "result".to_string(), init: Some(JsExpr::Binary { left: Box::new(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Trivia::default())), op: "/".to_string(), span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() },
            // 使用 eval
            JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Identifier("eval".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::Literal(NargoValue::String("console.log('test')".to_string()), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default()),
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到除以零
    let division_by_zero_issues = report.issues.iter().filter(|i| i.code == "unsafe-operation" && i.message == "Division by zero").count();
    assert_eq!(division_by_zero_issues, 1);

    // 应该检测到使用 eval
    let eval_issues = report.issues.iter().filter(|i| i.code == "unsafe-operation" && i.message == "Use of eval is potentially unsafe").count();
    assert_eq!(eval_issues, 1);
}

#[test]
fn test_analysis_report_generation() {
    let program = JsProgram { body: vec![JsStmt::VariableDecl { kind: "const".to_string(), id: "unusedVar".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }], span: Span::default(), trivia: Trivia::default() };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 测试控制台报告生成
    let console_report = report.generate_console_report();
    assert!(console_report.contains("ANALYSIS REPORT FOR: test.js"));
    assert!(console_report.contains("Total issues: 1"));
    assert!(console_report.contains("Unused variable: unusedVar"));

    // 测试 JSON 报告生成
    let json_report = report.generate_json_report().unwrap();
    if let NargoValue::Object(map) = json_report {
        assert!(map.contains_key("file_path"));
        assert!(map.contains_key("issues"));
        assert!(map.contains_key("duration_ms"));
        assert!(map.contains_key("total_issues"));
    }
    else {
        panic!("JSON report should be an object");
    }
}

#[test]
fn test_analyze_with_rules_unused_import() {
    let program = JsProgram { body: vec![JsStmt::Import { specifiers: vec!["unusedImport".to_string(), "usedImport".to_string()], source: "module".to_string(), span: Span::default(), trivia: Trivia::default() }, JsStmt::Expr(JsExpr::Identifier("usedImport".to_string(), Span::default(), Trivia::default()), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到未使用的导入
    let unused_import_issues = report.issues.iter().filter(|i| i.code == "unused-import" && i.message == "Unused import: unusedImport").count();
    assert_eq!(unused_import_issues, 1);

    // 应该没有检测到 usedImport 是未使用的
    let used_import_issues = report.issues.iter().filter(|i| i.code == "unused-import" && i.message == "Unused import: usedImport").count();
    assert_eq!(used_import_issues, 0);
}

#[test]
fn test_analyze_with_rules_memory_leak() {
    let program = JsProgram {
        body: vec![
            // 未清理的 setInterval
            JsStmt::VariableDecl { kind: "const".to_string(), id: "intervalId".to_string(), init: Some(JsExpr::Call { callee: Box::new(JsExpr::Identifier("setInterval".to_string(), Span::default(), Trivia::default())), args: vec![JsExpr::ArrowFunction { params: vec![], body: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }, JsExpr::Literal(NargoValue::Number(1000.0), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() },
            // 事件监听器
            JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Member { object: Box::new(JsExpr::Identifier("document".to_string(), Span::default(), Trivia::default())), property: Box::new(JsExpr::Identifier("addEventListener".to_string(), Span::default(), Trivia::default())), computed: false, span: Span::default(), trivia: Trivia::default() }), args: vec![JsExpr::Literal(NargoValue::String("click".to_string()), Span::default(), Trivia::default()), JsExpr::ArrowFunction { params: vec![], body: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() }], span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default()),
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到内存泄漏问题
    let memory_leak_issues = report.issues.iter().filter(|i| i.code == "memory-leak").count();
    assert!(memory_leak_issues > 0);
}

#[test]
fn test_analyze_with_rules_performance_bottleneck() {
    let program = JsProgram {
        body: vec![
            // 嵌套循环
            JsStmt::For {
                init: Some(Box::new(JsStmt::VariableDecl { kind: "let".to_string(), id: "i".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() })),
                test: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("i".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), op: "<".to_string(), span: Span::default(), trivia: Trivia::default() }),
                update: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("i".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }),
                body: Box::new(JsStmt::For {
                    init: Some(Box::new(JsStmt::VariableDecl { kind: "let".to_string(), id: "j".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() })),
                    test: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("j".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), op: "<".to_string(), span: Span::default(), trivia: Trivia::default() }),
                    update: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("j".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }),
                    body: Box::new(JsStmt::For {
                        init: Some(Box::new(JsStmt::VariableDecl { kind: "let".to_string(), id: "k".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(0.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() })),
                        test: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("k".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), op: "<".to_string(), span: Span::default(), trivia: Trivia::default() }),
                        update: Some(JsExpr::Binary { left: Box::new(JsExpr::Identifier("k".to_string(), Span::default(), Trivia::default())), right: Box::new(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }),
                        body: Box::new(JsStmt::Expr(JsExpr::Literal(NargoValue::Number(1.0), Span::default(), Trivia::default()), Span::default(), Trivia::default())),
                        span: Span::default(),
                        trivia: Trivia::default(),
                    }),
                    span: Span::default(),
                    trivia: Trivia::default(),
                }),
                span: Span::default(),
                trivia: Trivia::default(),
            },
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到性能瓶颈问题
    let performance_issues = report.issues.iter().filter(|i| i.code == "performance-bottleneck").count();
    assert!(performance_issues > 0);
}

#[test]
fn test_analyze_with_rules_security() {
    let program = JsProgram {
        body: vec![
            // SQL 注入
            JsStmt::VariableDecl { kind: "const".to_string(), id: "sql".to_string(), init: Some(JsExpr::Binary { left: Box::new(JsExpr::Literal(NargoValue::String("SELECT * FROM users WHERE id = ".to_string()), Span::default(), Trivia::default())), right: Box::new(JsExpr::Identifier("userId".to_string(), Span::default(), Trivia::default())), op: "+".to_string(), span: Span::default(), trivia: Trivia::default() }), span: Span::default(), trivia: Trivia::default() },
            // XSS
            JsStmt::Expr(JsExpr::Binary { left: Box::new(JsExpr::Member { object: Box::new(JsExpr::Identifier("element".to_string(), Span::default(), Trivia::default())), property: Box::new(JsExpr::Identifier("innerHTML".to_string(), Span::default(), Trivia::default())), computed: false, span: Span::default(), trivia: Trivia::default() }), right: Box::new(JsExpr::Identifier("userInput".to_string(), Span::default(), Trivia::default())), op: "=".to_string(), span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default()),
            // 不安全的密码存储
            JsStmt::Expr(JsExpr::Call { callee: Box::new(JsExpr::Member { object: Box::new(JsExpr::Identifier("localStorage".to_string(), Span::default(), Trivia::default())), property: Box::new(JsExpr::Identifier("setItem".to_string(), Span::default(), Trivia::default())), computed: false, span: Span::default(), trivia: Trivia::default() }), args: vec![JsExpr::Literal(NargoValue::String("password".to_string()), Span::default(), Trivia::default()), JsExpr::Identifier("userPassword".to_string(), Span::default(), Trivia::default())], span: Span::default(), trivia: Trivia::default() }, Span::default(), Trivia::default()),
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到安全问题
    let security_issues = report.issues.iter().filter(|i| i.code == "security").count();
    assert!(security_issues > 0);
}

#[test]
fn test_analyze_with_rules_code_style() {
    let program = JsProgram {
        body: vec![
            // 变量命名规范
            JsStmt::VariableDecl { kind: "const".to_string(), id: "CamelCaseVar".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(10.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() },
            // 单字符变量名
            JsStmt::VariableDecl { kind: "const".to_string(), id: "a".to_string(), init: Some(JsExpr::Literal(NargoValue::Number(20.0), Span::default(), Trivia::default())), span: Span::default(), trivia: Trivia::default() },
            // 函数命名规范
            JsStmt::FunctionDecl { id: "FunctionName".to_string(), params: vec![], body: vec![], is_async: false, span: Span::default(), trivia: Trivia::default() },
        ],
        span: Span::default(),
        trivia: Trivia::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let (_, report) = analyzer.analyze_with_rules(&program, Some("test.js".to_string())).unwrap();

    // 应该检测到代码风格问题
    let code_style_issues = report.issues.iter().filter(|i| i.code == "code-style").count();
    assert!(code_style_issues > 0);
}
