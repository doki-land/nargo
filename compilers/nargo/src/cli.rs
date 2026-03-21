//! CLI definition for Nargo.
//!
//! This module defines the command-line interface using clap.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::Commands;

/// Nargo - A high-performance frontend tool inspired by Vite and Rolldown, powered by HXO.
#[derive(Parser)]
#[command(name = "nargo")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    /// Parse command line arguments.
    pub fn parse() -> Self {
        Parser::parse()
    }

    /// Parse command line arguments from an iterator.
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Parser::parse_from(args)
    }

    /// Try to parse command line arguments, returning an error on failure.
    pub fn try_parse() -> Result<Self, clap::Error> {
        Parser::try_parse()
    }

    /// Execute the CLI synchronously.
    pub fn execute(&self) -> color_eyre::eyre::Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self.execute_async())
    }

    /// Execute the CLI asynchronously.
    pub async fn execute_async(&self) -> color_eyre::eyre::Result<()> {
        self.command.execute().await
    }
}
