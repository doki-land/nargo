use nargo_document::generator::markdown::MarkdownRenderer;
use std::fs;

fn main() {
    let renderer = MarkdownRenderer::new();

    // 测试从文件读取并渲染
    match fs::read_to_string("test_markdown.md") {
        Ok(content) => match renderer.render(&content) {
            Ok(html) => {
                println!("渲染成功！");
                println!("\n生成的 HTML:");
                println!("{}", html);
            }
            Err(e) => {
                println!("渲染失败: {}", e);
            }
        },
        Err(e) => {
            println!("读取文件失败: {}", e);
        }
    }

    // 测试直接渲染字符串
    let markdown = "# 测试标题\n\n**粗体文本**\n\n- 列表项 1\n- 列表项 2";
    match renderer.render(markdown) {
        Ok(html) => {
            println!("\n直接渲染字符串结果:");
            println!("{}", html);
        }
        Err(e) => {
            println!("渲染失败: {}", e);
        }
    }
}
