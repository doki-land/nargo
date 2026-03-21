use nargo_metadata::ApiMetadataExtractor;
use nargo_sdk::SdkGenerator;
use nargo_types::errors::Result;
use std::path::Path;

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

    println!("=== Nargo Metadata + Nargo SDK 使用示例 ===\n");

    example_extract_metadata()?;
    example_generate_sdk()?;

    println!("\n=== 示例运行完成 ===");
    Ok(())
}

const EXAMPLE_SOURCE: &str = r#"
class UserController {
    @http("GET", "/users")
    async listUsers(@Query page: number = 1, @Query limit: number = 10): Promise<User[]> {
    }

    @http("GET", "/users/:id")
    async getUser(@Path id: number): Promise<User> {
    }

    @http("POST", "/users")
    async createUser(@Body user: CreateUserDto): Promise<User> {
    }

    @http("PUT", "/users/:id")
    async updateUser(@Path id: number, @Body user: UpdateUserDto): Promise<User> {
    }

    @http("DELETE", "/users/:id")
    async deleteUser(@Path id: number): Promise<void> {
    }
}

class ProductController {
    @http("GET", "/products")
    async listProducts(@Query category: string): Promise<Product[]> {
    }

    @http("GET", "/products/:id")
    async getProduct(@Path id: number): Promise<Product> {
    }
}

interface User {
    id: number;
    name: string;
    email: string;
    createdAt: Date;
}

interface CreateUserDto {
    name: string;
    email: string;
    password: string;
}

interface UpdateUserDto {
    name?: string;
    email?: string;
}

interface Product {
    id: number;
    name: string;
    price: number;
    category: string;
}
"#;

fn example_extract_metadata() -> Result<()> {
    println!("1. 提取 API 元数据示例");

    let ir = nargo_ir::IRModule::default();
    let metadata = ApiMetadataExtractor::extract(EXAMPLE_SOURCE, &ir)?;

    println!("   找到 {} 个 API 端点", metadata.endpoints.len());
    println!("   找到 {} 个类型定义", metadata.types.len());

    for endpoint in &metadata.endpoints {
        println!("   - {}: {} {} ({})", endpoint.controller, endpoint.method, endpoint.path, endpoint.name);
    }

    for (name, _type_def) in &metadata.types {
        println!("   - 类型: {}", name);
    }

    println!("   元数据提取完成!\n");

    Ok(())
}

fn example_generate_sdk() -> Result<()> {
    println!("2. 生成 TypeScript SDK 示例");

    let ir = nargo_ir::IRModule::default();
    let metadata = ApiMetadataExtractor::extract(EXAMPLE_SOURCE, &ir)?;

    let output_dir = Path::new("./output");
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    println!("   输出目录: {}", output_dir.display());

    SdkGenerator::generate(&metadata, output_dir)?;

    println!("   SDK 生成成功!");
    println!("   请查看 output/ 目录下的生成文件");

    Ok(())
}
