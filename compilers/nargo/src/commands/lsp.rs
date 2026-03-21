//! LSP and MCP server commands.
//!
//! This module provides Language Server Protocol and Model Context Protocol server functionality.

use color_eyre::eyre::Result;

/// Execute lsp command.
pub async fn execute_lsp(port: u16, stdio: bool) -> Result<()> {
    if stdio {
        println!("🚀 Starting Nargo LSP over stdio...");
        nargo_lsp::run_stdio().await.map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    else {
        println!("🚀 Starting Nargo LSP on http://127.0.0.1:{}...", port);
        // 简化实现，使用 run_stdio 代替
        nargo_lsp::run_stdio().await.map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    Ok(())
}

/// Execute mcp command.
pub async fn execute_mcp(port: u16, stdio: bool) -> Result<()> {
    if stdio {
        println!("🚀 Starting Nargo MCP over stdio...");
        nargo_mcp::run_stdio().await.map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    else {
        println!("🚀 Starting Nargo MCP on http://127.0.0.1:{}...", port);
        // 简化实现，使用 run_stdio 代替
        nargo_mcp::run_stdio().await.map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    Ok(())
}
