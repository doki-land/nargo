//! Command definitions for Nargo CLI.
//!
//! This module defines all available commands and their execution logic.

mod bridge;
mod build;
mod dev;
mod git;
mod init;
mod lsp;
mod package;
mod run;
mod test;

use clap::Subcommand;
use color_eyre::eyre::Result;
use std::path::PathBuf;

pub use git::{GitCommands, HookCommands};

/// Available Nargo commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start development server with hot reload.
    Dev {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Port to listen on.
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Enable integrated mock server.
        #[arg(short, long)]
        mock: bool,

        /// Mock server port.
        #[arg(long, default_value_t = 4000)]
        mock_port: u16,

        /// Mock directory.
        #[arg(long, default_value = "mocks")]
        mock_dir: PathBuf,
    },

    /// Build for production.
    Build {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Output directory.
        #[arg(short, long, default_value = "dist")]
        out_dir: PathBuf,
    },

    /// Compile project (CaaS - Compiler as a Service).
    Compile {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Enable watch mode.
        #[arg(short, long)]
        watch: bool,
    },

    /// Run monorepo tasks.
    Run {
        /// Task name to run.
        task: String,

        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Enable hybrid hot reload (Rust + TS).
        #[arg(short = 'H', long)]
        hybrid: bool,

        /// Enable watch mode.
        #[arg(short, long)]
        watch: bool,
    },

    /// Git related operations (Native).
    Git {
        #[command(subcommand)]
        subcommand: GitCommands,
    },

    /// Native Git Hooks management.
    Hooks {
        #[command(subcommand)]
        subcommand: HookCommands,
    },

    /// Serve static files.
    Serve {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Port to listen on.
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Directory to serve.
        #[arg(default_value = "dist")]
        dir: PathBuf,
    },

    /// Generate TS types from Rust structs.
    Bridge {
        /// Output directory for TS files.
        #[arg(short, long, default_value = "packages/nargo/types")]
        out_dir: PathBuf,
    },

    /// Initialize a new project.
    Init(init::InitArgs),

    /// Run tests.
    Test {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Test file filter (glob pattern or file path).
        filter: Option<String>,

        /// Enable coverage collection.
        #[arg(short, long)]
        coverage: bool,

        /// Generate report.
        #[arg(long)]
        report: bool,

        /// Run tests in parallel.
        #[arg(long, default_value_t = true)]
        parallel: bool,

        /// Test timeout in milliseconds.
        #[arg(long, default_value_t = 5000)]
        timeout: u64,

        /// Watch mode.
        #[arg(short, long)]
        watch: bool,
    },

    /// Lint source files.
    Lint {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Generate report.
        #[arg(long)]
        report: bool,
    },

    /// Format source files.
    Format {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Write changes to files.
        #[arg(short, long)]
        write: bool,

        /// Generate report.
        #[arg(long)]
        report: bool,
    },

    /// Audit project for security issues.
    Audit {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Generate report.
        #[arg(long)]
        report: bool,
    },

    /// Benchmark performance.
    Bench {
        /// Sample file to benchmark.
        #[arg(default_value = "src/App.nargo")]
        file: PathBuf,

        /// Number of iterations.
        #[arg(short, long, default_value_t = 100)]
        iterations: u32,
    },

    /// Run mock server (RaaS - Router as a Service).
    Mock {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Port to listen on.
        #[arg(short, long, default_value_t = 4000)]
        port: u16,

        /// Directory containing mock JSON files.
        #[arg(short, long, default_value = "mocks")]
        dir: PathBuf,
    },

    /// Start Language Server (LSP).
    Lsp {
        /// Listen on a specific port (HTTP/WebSocket).
        #[arg(short, long, default_value_t = 5005)]
        port: u16,

        /// Use stdio for communication.
        #[arg(long)]
        stdio: bool,
    },

    /// Start MCP (Model Context Protocol) server.
    Mcp {
        /// Listen on a specific port.
        #[arg(short, long, default_value_t = 5006)]
        port: u16,

        /// Use stdio for communication.
        #[arg(long)]
        stdio: bool,
    },

    /// Run all checks and generate comprehensive report.
    Check {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// Add a dependency to the project.
    Add {
        /// Package name (with optional version, e.g., lodash@4.17.21).
        package: String,

        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Add as dev dependency.
        #[arg(short = 'D', long)]
        dev: bool,

        /// Add as optional dependency.
        #[arg(short, long)]
        optional: bool,

        /// Enable specific features.
        #[arg(short = 'F', long)]
        features: Vec<String>,
    },

    /// Remove a dependency from the project.
    Remove {
        /// Package name.
        package: String,

        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// Update dependencies.
    Update {
        /// Package name (optional, updates all if not specified).
        package: Option<String>,

        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Update to latest versions (ignore lock file).
        #[arg(short, long)]
        latest: bool,
    },

    /// Install dependencies from Nargo.toml.
    Install {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Force reinstall all dependencies.
        #[arg(short, long)]
        force: bool,

        /// Production only (skip dev dependencies).
        #[arg(short = 'P', long)]
        production: bool,
    },

    /// Migrate from package.json to Nargo.toml.
    Migrate {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Keep original package.json.
        #[arg(long)]
        keep_original: bool,
    },

    /// Clean cache and build artifacts.
    Clean {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Clean cache only.
        #[arg(long)]
        cache: bool,

        /// Clean target directory only.
        #[arg(long)]
        target: bool,
    },

    /// Search for packages in the registry.
    Search {
        /// Search query.
        query: String,

        /// Maximum number of results.
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// Publish package to registry.
    Publish {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Target registry URL.
        #[arg(short, long)]
        registry: Option<String>,

        /// Dry run (don't actually publish).
        #[arg(long)]
        dry_run: bool,

        /// Access level (public or restricted).
        #[arg(long, default_value = "public")]
        access: String,
    },

    /// Show dependency tree.
    Tree {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Maximum depth to display.
        #[arg(short, long, default_value_t = 10)]
        depth: usize,

        /// Show duplicate dependencies.
        #[arg(long)]
        duplicates: bool,
    },

    /// Display project information.
    Info {
        /// Project root directory.
        #[arg(default_value = ".")]
        root: PathBuf,

        /// Output format (text or json).
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

impl Commands {
    /// Execute the command asynchronously.
    pub async fn execute(&self) -> Result<()> {
        match self {
            Commands::Dev { root, port, mock, mock_port, mock_dir } => dev::execute_dev(root, *port, *mock, *mock_port, mock_dir).await,
            Commands::Build { root, out_dir } => build::execute_build(root, out_dir).await,
            Commands::Compile { root, watch } => build::execute_compile(root, *watch).await,
            Commands::Run { task, root, hybrid, watch } => run::execute_run(task, root, *hybrid, *watch).await,
            Commands::Git { subcommand } => git::execute_git(subcommand).await,
            Commands::Hooks { subcommand } => git::execute_hooks(subcommand).await,
            Commands::Serve { root, port, dir } => dev::execute_serve(root, *port, dir).await,
            Commands::Bridge { out_dir } => bridge::execute_bridge(out_dir).await,
            Commands::Init(args) => init::execute_init(args.clone()).await,
            Commands::Test { root, filter, coverage, report, parallel, timeout, watch } => test::execute_test(root, filter.as_ref(), *coverage, *report, *parallel, *timeout, *watch).await,
            Commands::Lint { root, report } => test::execute_lint(root, *report).await,
            Commands::Format { root, write, report } => test::execute_format(root, *write, *report).await,
            Commands::Audit { root, report } => test::execute_audit(root, *report).await,
            Commands::Bench { file, iterations } => test::execute_bench(file, *iterations).await,
            Commands::Mock { root, port, dir } => dev::execute_mock(root, *port, dir).await,
            Commands::Lsp { port, stdio } => lsp::execute_lsp(*port, *stdio).await,
            Commands::Mcp { port, stdio } => lsp::execute_mcp(*port, *stdio).await,
            Commands::Check { root } => test::execute_check(root).await,
            Commands::Add { package, root, dev, optional, features } => package::execute_add(package, root, *dev, *optional, features).await,
            Commands::Remove { package, root } => package::execute_remove(package, root).await,
            Commands::Update { package, root, latest } => package::execute_update(package, root, *latest).await,
            Commands::Install { root, force, production } => package::execute_install(root, *force, *production).await,
            Commands::Migrate { root, keep_original } => package::execute_migrate(root, *keep_original).await,
            Commands::Clean { root, cache, target } => package::execute_clean(root, *cache, *target).await,
            Commands::Search { query, limit } => package::execute_search(query, *limit).await,
            Commands::Publish { root, registry, dry_run, access } => package::execute_publish(root, registry, *dry_run, access).await,
            Commands::Tree { root, depth, duplicates } => package::execute_tree(root, *depth, *duplicates).await,
            Commands::Info { root, format } => package::execute_info(root, format).await,
        }
    }
}
