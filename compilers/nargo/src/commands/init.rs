//! Init command for project initialization.
//!
//! This module provides functionality to initialize new Nargo projects.

use clap::Args;
use color_eyre::eyre::Result;

/// Initialize a new Nargo project
#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Project name
    pub name: String,

    /// Use non-interactive mode with default settings
    #[arg(long, short)]
    pub yes: bool,
}

/// Execute init command.
pub async fn execute_init(args: InitArgs) -> Result<()> {
    println!("✨ Initializing new project: {}", args.name);

    if args.yes {
        nargo_scaffolder::init(&args.name).map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    else {
        let config = nargo_scaffolder::interactive_config(&args.name).map_err(|e| color_eyre::eyre::eyre!(e))?;
        nargo_scaffolder::init_with_config(config).map_err(|e| color_eyre::eyre::eyre!(e))?;
    }

    println!("✅ Project initialized successfully!");
    println!("\nNext steps:");
    println!("  cd {}", args.name);
    println!("  pnpm install");
    println!("  nargo dev");

    Ok(())
}
