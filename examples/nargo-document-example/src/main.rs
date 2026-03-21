use nargo_document::{config::Config, generator::Generator};
use std::path::Path;

fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

    println!("=== Nargo Document 使用示例 ===\n");

    example_load_config();
    example_generate_documentation();

    println!("\n=== 示例运行完成 ===");
}

fn example_load_config() {
    println!("1. 加载配置示例");

    let config_path = Path::new("nargodoc.config.toml");
    let config = if config_path.exists() {
        match Config::load(config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("   加载配置失败: {}", e);
                Config::default()
            }
        }
    }
    else {
        println!("   配置文件不存在，使用默认配置");
        Config::default()
    };

    println!("   站点标题: {}", config.title);
    println!("   描述: {}", config.description);
    println!("   配置加载完成!\n");
}

fn example_generate_documentation() {
    println!("2. 生成文档示例");

    let config = Config::default();
    let _generator = Generator::new(config);

    let input_dir = Path::new("docs");
    let output_dir = Path::new("dist");

    println!("   输入目录: {}", input_dir.display());
    println!("   输出目录: {}", output_dir.display());

    if !input_dir.exists() {
        println!("   输入目录不存在，跳过文档生成");
        return;
    }

    println!("   文档生成器已准备好!\n");
}
