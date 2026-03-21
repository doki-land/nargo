//! Nargo CLI - A high-performance frontend tool inspired by Vite and Rolldown.
//!
//! This crate provides the command-line interface for Nargo, including:
//! - Development server with hot reload
//! - Production builds
//! - Testing and coverage
//! - Linting and formatting
//! - Package management
//! - Git hooks
//! - LSP and MCP servers

#![warn(missing_docs)]

pub mod cli;
pub mod commands;
pub mod installer;

pub use cli::Cli;
pub use commands::Commands;
pub use installer::{InstallDiff, InstallOptions, InstallPhase, InstallProgress, InstallResult, Installer};

use color_eyre::eyre::Result;

/// Run the Nargo CLI with the given arguments.
///
/// # Arguments
///
/// * `args` - Command line arguments (typically `std::env::args_os()`)
///
/// # Returns
///
/// Result indicating success or failure.
pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    cli.execute()
}

/// Run the Nargo CLI asynchronously.
///
/// # Arguments
///
/// * `args` - Command line arguments (typically `std::env::args_os()`)
///
/// # Returns
///
/// Result indicating success or failure.
pub async fn run_async<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    cli.execute_async().await
}
