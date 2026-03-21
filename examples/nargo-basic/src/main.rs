use nargo_config::NargoConfig;
use nargo_types::Result;
use std::path::Path;

fn main() -> Result<()> {
    println!("=== Nargo 基础使用示例 ===\n");

    example_config_loader()?;
    example_path_operations()?;

    println!("\n=== 示例运行完成 ===");
    Ok(())
}

fn example_config_loader() -> Result<()> {
    println!("1. 配置加载器示例");

    let mut config = NargoConfig::default();
    config.name = Some("my-app".to_string());
    config.version = Some("0.1.0".to_string());

    println!("   包名: {:?}", config.name);
    println!("   版本: {:?}", config.version);
    println!("   配置构建完成!\n");

    Ok(())
}

fn example_path_operations() -> Result<()> {
    println!("2. 路径操作示例");

    let current_dir = std::env::current_dir()?;
    println!("   当前目录: {}", current_dir.display());

    let src_path = Path::new("src");
    if src_path.exists() {
        println!("   src 目录存在");
    }
    else {
        println!("   src 目录不存在");
    }

    Ok(())
}
