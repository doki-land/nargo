#![feature(new_range_api)]
#![warn(missing_docs)]
use std::sync::Arc;

use crate::language_service::NargoLanguageService;

/// 运行标准输入/输出服务器
pub async fn run_stdio() -> nargo_types::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let nargo_service = Arc::new(NargoLanguageService::new());
    let server = oak_lsp::LspServer::new(nargo_service);
    server.run(stdin, stdout).await.map_err(|e| nargo_types::Error::external_error("oak-lsp".to_string(), format!("{}", e), nargo_types::Span::unknown()))?;
    Ok(())
}

/// 运行HTTP服务器
pub async fn run_http_server(port: u16) -> nargo_types::Result<()> {
    println!("HTTP/WebSocket LSP server on port {} is not yet fully implemented.", port);
    println!("Falling back to stdio for now.");
    run_stdio().await
}
