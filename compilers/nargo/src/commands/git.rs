//! Git related commands.
//!
//! This module provides git operations and hook management.

use clap::Subcommand;
use color_eyre::eyre::Result;
use std::path::PathBuf;

/// Git subcommands.
#[derive(Debug, Subcommand)]
pub enum GitCommands {
    /// Show git status.
    Status {
        /// Repository path.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Show current branch.
    Branch {
        /// Repository path.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Clone a repository.
    Clone {
        /// Repository URL.
        url: String,

        /// Target directory.
        target: Option<PathBuf>,
    },

    /// Initialize a new git repository.
    Init {
        /// Repository path.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

/// Hook subcommands.
#[derive(Debug, Subcommand)]
pub enum HookCommands {
    /// Install nargo hooks into .git/hooks.
    Install {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// Uninstall nargo hooks from .git/hooks.
    Uninstall {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// Run a specific hook (used internally by git).
    Run {
        /// Name of the hook to run.
        name: String,

        /// Arguments passed by git.
        args: Vec<String>,
    },
}

/// Execute git subcommand.
pub async fn execute_git(subcommand: &GitCommands) -> Result<()> {
    match subcommand {
        GitCommands::Status { path } => {
            let status = nargo_git::get_status(path).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("📊 Git Status for {}:", path.display());
            if status.is_empty() {
                println!("  Clean");
            }
            else {
                for (file, stat) in status {
                    println!("  {} -> {}", file, stat);
                }
            }
        }
        GitCommands::Branch { path } => {
            let branch = nargo_git::get_current_branch(path).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("🌿 Current branch: {}", branch);
        }
        GitCommands::Clone { url, target } => {
            println!("📥 Cloning {}...", url);
            nargo_git::clone(url, target.as_ref().map(|v| &**v)).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("✅ Clone completed!");
        }
        GitCommands::Init { path } => {
            nargo_git::init(path).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("✅ Git repository initialized at {}", path.display());
        }
    }
    Ok(())
}

/// Execute hooks subcommand.
pub async fn execute_hooks(subcommand: &HookCommands) -> Result<()> {
    match subcommand {
        HookCommands::Install { root } => {
            nargo_hooks::NargoHooks::install(root).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("✅ Git hooks installed successfully.");
        }
        HookCommands::Uninstall { root } => {
            nargo_hooks::NargoHooks::uninstall(root).map_err(|e| color_eyre::eyre::eyre!(e))?;
            println!("✅ Git hooks uninstalled successfully.");
        }
        HookCommands::Run { name, args } => {
            nargo_hooks::NargoHooks::run(name, args.clone(), None).await.map_err(|e| color_eyre::eyre::eyre!(e))?;
        }
    }
    Ok(())
}
