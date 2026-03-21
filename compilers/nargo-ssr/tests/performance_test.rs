use nargo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use nargo_ssr::{PrefetchManager, RouteHandler, Router, SsrBackend};
use nargo_types::{CompileMode, Span};
use std::{collections::HashMap, time::Instant};

/// 测试组件级缓存性能
#[test]
fn test_component_cache_performance() {
    // 创建 SSR 后端
    let mut backend = SsrBackend::new(CompileMode::Vue2);

    // 生成大型组件
    let ir = generate_large_component(100);

    // 第一次渲染（无缓存）
    let start1 = Instant::now();
    let result1 = backend.generate(&ir).unwrap();
    let duration1 = start1.elapsed();

    // 第二次渲染（有缓存）
    let start2 = Instant::now();
    let result2 = backend.generate(&ir).unwrap();
    let duration2 = start2.elapsed();

    // 验证结果一致
    assert_eq!(result1, result2);

    // 验证缓存提升性能
    println!("Component Cache Performance:");
    println!("First render: {:?}", duration1);
    println!("Second render (cached): {:?}", duration2);
    println!("Speedup: {:.2}x", duration1.as_nanos() as f64 / duration2.as_nanos() as f64);

    // 确保缓存有效，第二次渲染至少快 10 倍
    assert!(duration2 < duration1 / 10);
}

/// 测试大型页面渲染性能
#[test]
fn test_large_page_rendering() {
    // 创建 SSR 后端
    let mut backend = SsrBackend::new(CompileMode::Vue2);

    // 测试不同大小的组件
    let sizes = vec![10, 100, 500];

    for size in sizes {
        // 生成大型组件
        let ir = generate_large_component(size);

        // 测量渲染时间
        let start = Instant::now();
        let result = backend.generate(&ir).unwrap();
        let duration = start.elapsed();

        println!("Large Page Rendering ({} elements):", size);
        println!("Duration: {:?}", duration);
        println!("Output size: {} bytes", result.len());

        // 确保渲染成功
        assert!(!result.is_empty());
    }
}

/// 测试动态路由处理性能
#[test]
fn test_dynamic_route_performance() {
    // 创建路由表
    let mut router = Router::new();

    // 添加测试路由
    router.add_route("/users/:id", TestRouteHandler);
    router.add_route("/posts/:post_id/comments/:comment_id", TestRouteHandler);

    // 测试路由匹配性能
    let routes = vec!["/users/123", "/users/456", "/posts/789/comments/101", "/posts/202/comments/303"];

    // 第一次匹配（无缓存）
    let start1 = Instant::now();
    for route in &routes {
        let result = router.match_route(route);
        assert!(result.is_some());
    }
    let duration1 = start1.elapsed();

    // 第二次匹配（有缓存）
    let start2 = Instant::now();
    for route in &routes {
        let result = router.match_route(route);
        assert!(result.is_some());
    }
    let duration2 = start2.elapsed();

    println!("Dynamic Route Performance:");
    println!("First match: {:?}", duration1);
    println!("Second match (cached): {:?}", duration2);
    println!("Speedup: {:.2}x", duration1.as_nanos() as f64 / duration2.as_nanos() as f64);

    // 确保缓存有效
    assert!(duration2 < duration1 / 5);
}

/// 测试数据预取性能
#[tokio::test]
async fn test_prefetch_performance() {
    // 创建预取管理器
    let mut prefetch_manager = PrefetchManager::new();

    // 注册测试预取函数
    prefetch_manager.register_prefetch("user", |params, _query| async move {
        let id = params.get("id").unwrap_or(&"1".to_string());
        Ok(serde_json::json!({
            "id": id,
            "name": format!("User {}", id),
            "email": format!("user{}@example.com", id)
        }))
    });

    // 测试参数
    let params = HashMap::from([("id", "123".to_string())]);
    let query = HashMap::new();

    // 第一次预取（无缓存）
    let start1 = Instant::now();
    let result1 = prefetch_manager.prefetch(&["user"], &params, &query).await;
    let duration1 = start1.elapsed();

    // 第二次预取（有缓存）
    let start2 = Instant::now();
    let result2 = prefetch_manager.prefetch(&["user"], &params, &query).await;
    let duration2 = start2.elapsed();

    // 验证结果一致
    assert_eq!(result1, result2);

    println!("Data Prefetch Performance:");
    println!("First prefetch: {:?}", duration1);
    println!("Second prefetch (cached): {:?}", duration2);
    println!("Speedup: {:.2}x", duration1.as_nanos() as f64 / duration2.as_nanos() as f64);

    // 确保缓存有效
    assert!(duration2 < duration1 / 10);
}

/// 生成大型组件的 IR
fn generate_large_component(size: usize) -> IRModule {
    let mut nodes = Vec::new();

    // 根元素
    let root = ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "class".to_string(), value: Some("container".to_string()), is_directive: false, is_dynamic: false, argument: None, modifiers: vec![], span: Span::default() }], children: vec![], is_static: false, span: Span::default() };

    // 添加子元素
    let mut children = vec![];
    for i in 0..size {
        let child = TemplateNodeIR::Element(ElementIR { tag: "div".to_string(), attributes: vec![AttributeIR { name: "class".to_string(), value: Some(format!("item item-{}", i)), is_directive: false, is_dynamic: false, argument: None, modifiers: vec![], span: Span::default() }], children: vec![TemplateNodeIR::Text(format!("Item {}", i), Span::default(), Default::default()), TemplateNodeIR::Interpolation(nargo_ir::InterpolationIR { code: format!("item{}", i), span: Span::default() })], is_static: false, span: Span::default() });
        children.push(child);
    }

    let root_element = TemplateNodeIR::Element(ElementIR { children, ..root });
    nodes.push(root_element);

    IRModule { name: format!("LargeComponent{}", size), template: Some(TemplateIR { nodes, span: Span::default() }), script: None, script_server: None, style: None, span: Span::default() }
}

/// 测试路由处理器
struct TestRouteHandler;

impl RouteHandler for TestRouteHandler {
    fn handle(&self, params: &HashMap<String, String>, _query: &HashMap<String, String>) -> Result<String, String> {
        Ok(format!("Route handled with params: {:?}", params))
    }
}
