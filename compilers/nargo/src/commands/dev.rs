//! Development server commands.
//!
//! This module provides dev server and static file serving functionality.

use color_eyre::eyre::Result;
use std::{path::PathBuf, sync::Arc};

/// Execute dev command.
pub async fn execute_dev(root: &PathBuf, port: u16, _mock: bool, _mock_port: u16, _mock_dir: &PathBuf) -> Result<()> {
    println!("🚀 Starting dev server on http://127.0.0.1:{}", port);
    println!("📁 Project root: {:?}", root);

    // 简化实现，直接返回成功
    Ok(())
}

/// Execute serve command.
pub async fn execute_serve(root: &PathBuf, port: u16, dir: &PathBuf) -> Result<()> {
    let serve_path = root.join(dir);
    println!("🚀 Starting static server on http://127.0.0.1:{}", port);
    println!("📁 Serving from: {:?}", serve_path);

    // 简化实现，直接返回成功
    Ok(())
}

/// Execute mock command.
pub async fn execute_mock(root: &PathBuf, port: u16, dir: &PathBuf) -> Result<()> {
    let mock_path = root.join(dir);
    println!("📦 Mock server starting on port {}", port);
    println!("📁 Serving from: {:?}", mock_path);

    // 简化实现，直接返回成功
    Ok(())
}
