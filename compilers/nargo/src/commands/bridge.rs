//! Bridge command for TypeScript type generation.
//!
//! This module provides functionality to generate TypeScript types from Rust structs.

use color_eyre::eyre::Result;
use std::path::PathBuf;

/// Execute bridge command.
pub async fn execute_bridge(out_dir: &PathBuf) -> Result<()> {
    println!("🌉 Generating Type Bridge to {}...", out_dir.display());
    let mut bridge = nargo_bridge::NargoBridge::new();
    // 简化实现，不注册具体类型
    bridge.generate(&out_dir.join("index.ts")).map_err(|e| color_eyre::eyre::eyre!(e))?;
    println!("✅ Type Bridge generated successfully!");
    Ok(())
}
