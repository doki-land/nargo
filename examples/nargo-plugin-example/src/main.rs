use nargo_plugin::PluginManager;
use nargo_types::NargoContext;
use std::sync::Arc;

mod plugins;

use plugins::{LoggingPlugin, MinifyPlugin, TransformPlugin};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

    println!("=== Nargo Plugin 插件开发示例 ===\n");

    example_plugin_basics();
    example_plugin_chain().await;
    example_multiple_plugins().await;

    println!("\n=== 示例运行完成 ===");
}

fn example_plugin_basics() {
    println!("1. 插件基础使用示例");

    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let logging_plugin = LoggingPlugin::new();
    manager.register(Box::new(logging_plugin));

    println!("   已注册插件: LoggingPlugin");
    println!("   初始化所有插件...");

    if let Err(e) = manager.init_all() {
        println!("   初始化失败: {}", e);
    }
    else {
        println!("   插件初始化完成!");
    }

    println!();
}

async fn example_plugin_chain() {
    println!("2. 插件链处理示例");

    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    let transform_plugin = TransformPlugin::new();
    manager.register(Box::new(transform_plugin));

    let input_code = r#"
console.log("Hello, World!");
function add(a, b) {
    return a + b;
}
"#;

    println!("   输入代码:\n{}", input_code);
    println!("   应用插件变换...");

    match manager.transform(input_code.to_string()).await {
        Ok(result) => println!("   变换后代码:\n{}", result),
        Err(e) => println!("   变换失败: {}", e),
    }

    println!("   插件链处理完成!\n");
}

async fn example_multiple_plugins() {
    println!("3. 多插件协作示例");

    let ctx = Arc::new(NargoContext::new(serde_json::json!({})));
    let mut manager = PluginManager::new(ctx);

    manager.register(Box::new(LoggingPlugin::new()));
    manager.register(Box::new(TransformPlugin::new()));
    manager.register(Box::new(MinifyPlugin::new()));

    println!("   已注册 3 个插件");
    if let Err(e) = manager.init_all() {
        println!("   初始化失败: {}", e);
        return;
    }

    let input_code = r#"
// 这是一个注释
function   greeting(   name   )   {
    console.log(   "Hello, "   +   name   +   "!"   );
}
"#;

    println!("   输入代码:\n{}", input_code);
    println!("   应用插件链...");

    match manager.transform(input_code.to_string()).await {
        Ok(result) => println!("   最终结果:\n{}", result),
        Err(e) => println!("   处理失败: {}", e),
    }

    println!("   多插件协作完成!\n");
}
